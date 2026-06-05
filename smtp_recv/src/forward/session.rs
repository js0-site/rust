use std::{mem, net::IpAddr};

use idns::{Spf, spf::Status};
use log::info;
use mail_forward::Forward;
use ssl_trait::CertByHost;
use tokio::net::TcpStream;

use crate::{
  Mailer, Result, S, SmtpError, Stream,
  cmd::{handle_cmd, handle_forward_rcpt},
  send_to::SendTo,
  session::{self, Action, parse_address, read_mail},
  stream::{StreamEnum, perform_tls_handshake},
};

/// 检查是否是本地地址（包括 IPv4-mapped IPv6 loopback）
fn is_local(ip: &IpAddr) -> bool {
  match ip {
    IpAddr::V4(v4) => v4.is_loopback(),
    IpAddr::V6(v6) => v6.is_loopback() || v6.to_ipv4_mapped().is_some_and(|v4| v4.is_loopback()),
  }
}

/// SMTP转发会话结构，管理25端口的邮件接收和转发
///
/// 与认证会话不同，转发会话：
/// - 不需要用户认证
/// - 支持STARTTLS升级
/// - 接收邮件后转发给指定的邮箱服务
/// - 支持SPF验证发件人
pub struct Session<F: Forward, M: Mailer, C: CertByHost> {
  /// 流（明文或TLS）
  stream: StreamEnum,
  /// 转发服务
  forward: F,
  /// 邮件发送服务
  mailer: M,
  /// SSL证书提供者
  ssl: C,
  /// 实际的SNI主机名（STARTTLS后设置）
  sni_host: Option<String>,
  /// 邮件发送目标信息
  send_to: SendTo,
  /// 客户端IP地址（用于SPF验证）
  ip: IpAddr,
}

impl<F: Forward, M: Mailer, C: CertByHost> session::Session for Session<F, M, C> {
  type Stream = StreamEnum;

  fn stream(&mut self) -> &mut Self::Stream {
    &mut self.stream
  }

  async fn handle_cmd(
    &mut self,
    cmd: &str,
    args: &str,
    pending_responses: &mut Vec<String>,
  ) -> Result<Action> {
    match cmd {
      "EHLO" | "HELO" => Ok(Action::Reply(self.handle_ehlo()?)),
      "STARTTLS" => {
        if !self.stream.is_tls() {
          // STARTTLS需要立即处理，不能批量
          self.stream.flush(pending_responses).await?;
          pending_responses.clear();
          self.handle_starttls().await?;
          Ok(Action::Handled)
        } else {
          Ok(S::TLS_ALREADY_ACTIVE.into())
        }
      }
      "MAIL" => self.handle_mail(args).await,
      "RCPT" => Ok(handle_forward_rcpt(args, &mut self.send_to)),
      _ => Ok(handle_cmd(cmd, &mut self.send_to, 25)),
    }
  }

  async fn handle_data(&mut self) -> Result<()> {
    self.read_and_forward_mail().await
  }
}

impl<F: Forward, M: Mailer, C: CertByHost> Session<F, M, C> {
  /// 创建新的转发会话
  pub fn new(stream: TcpStream, forward: F, mailer: M, ssl: C, ip: IpAddr) -> Self {
    Self {
      stream: StreamEnum::new_plain(stream),
      forward,
      mailer,
      ssl,
      sni_host: None,
      send_to: SendTo::new(),
      ip,
    }
  }

  /// 向客户端发送SMTP响应（自动添加CRLF）
  async fn send(&mut self, msg: &str) -> Result<()> {
    self.stream.send(msg).await
  }

  /// 运行SMTP转发会话主循环
  pub async fn run(mut self) -> Result<()> {
    session::run_loop(&mut self).await
  }

  /// 获取当前有效的主机名（优先使用SNI主机名）
  fn host(&self) -> Result<&str> {
    match self.sni_host.as_deref() {
      Some(h) => Ok(h),
      None if self.stream.is_tls() => Err(SmtpError::NoSni),
      None => Ok("localhost"),
    }
  }

  /// 处理EHLO/HELO命令
  fn handle_ehlo(&self) -> Result<String> {
    let host = self.host()?;
    Ok(if self.stream.is_tls() {
      format!("250-{host}\r\n250-PIPELINING\r\n250 8BITMIME")
    } else {
      format!("250-{host}\r\n250-STARTTLS\r\n250-PIPELINING\r\n250 8BITMIME")
    })
  }

  /// 处理 MAIL FROM 命令（RFC 7208: SPF 验证应在此阶段进行）
  async fn handle_mail(&mut self, args: &str) -> Result<Action> {
    let Some(sender) = parse_address(args) else {
      return Ok(S::INVALID_MAIL_SYNTAX.into());
    };

    // SPF 验证（跳过本地回环地址）
    if !is_local(&self.ip)
      && let Some((_, host)) = xmail::norm_user_host(&sender)
    {
      let status = Spf::verify(&*idot::DOT, &host, self.ip).await;
      info!("SPF {host} {} = {status:?}", self.ip);
      match status {
        Status::Fail | Status::PermError => return Ok(S::SPF_FAIL.into()),
        Status::TempError => return Ok(S::SPF_TEMP_ERROR.into()),
        _ => {}
      }
    }

    self.send_to.set_sender(sender);
    Ok(Action::Ok)
  }

  /// 处理STARTTLS命令
  async fn handle_starttls(&mut self) -> Result<()> {
    self.send(S::STARTTLS_READY).await?;

    let StreamEnum::Plain(plain) = mem::replace(&mut self.stream, StreamEnum::None) else {
      return Err(SmtpError::StreamNone);
    };

    let tcp = plain.reader.into_inner().unsplit(plain.writer);
    let (tls, sni) = perform_tls_handshake(tcp, &self.ssl, true).await?;

    self.sni_host = Some(sni);
    self.stream = StreamEnum::new_tls(tls);
    Ok(())
  }

  /// 读取邮件内容并转发
  async fn read_and_forward_mail(&mut self) -> Result<()> {
    let body = read_mail(&mut self.stream).await?;

    if let Some(sender) = self.send_to.take_sender() {
      let to_li = self.send_to.take_to_li();

      // 获取转发目标
      let forward_li = match self.forward_li(&to_li).await {
        Ok(li) => li,
        Err(_) => return Ok(()), // 错误已在函数内处理
      };

      if forward_li.is_empty() {
        return Ok(());
      }

      // 发送邮件
      self.send_mail(&sender, forward_li, body).await?;
    }

    Ok(())
  }

  /// 根据收件人列表获取转发目标
  async fn forward_li(&mut self, to_li: &[String]) -> Result<Vec<String>> {
    let li = match to_li.len() {
      0 => return Ok(vec![]),
      1 => match self.forward.forward(&to_li[0]).await {
        Ok(Some(t)) => vec![t],
        Ok(None) => {
          self.send(S::NO_SUCH_USER).await?;
          return Ok(vec![]);
        }
        Err(e) => {
          log::error!("forward: {e}");
          self.send(S::LOCAL_ERROR).await?;
          return Err(e.into());
        }
      },
      _ => match self.forward.forward_set(to_li).await {
        Ok(li) if li.is_empty() => {
          self.send(S::NO_SUCH_USER).await?;
          return Ok(vec![]);
        }
        Ok(li) => li,
        Err(e) => {
          log::error!("forward: {e}");
          self.send(S::LOCAL_ERROR).await?;
          return Err(e.into());
        }
      },
    };
    info!("forward {to_li:?} → {li:?}");
    Ok(li)
  }

  /// 发送邮件到指定目标
  async fn send_mail(&mut self, sender: &str, li: Vec<String>, body: Vec<u8>) -> Result<()> {
    let n = li.len();
    let Some(mail) = mail_struct::Mail::new(sender, li, body) else {
      log::error!("mail construct: {sender}");
      self.send(S::LOCAL_ERROR).await?;
      return Ok(());
    };

    match self.mailer.forward(mail).await {
      Ok(_) => {
        info!("{sender} forwarded to {n}");
        self.send(S::MESSAGE_ACCEPTED_FORWARDING).await?;
      }
      Err(e) => {
        log::error!("mailer {sender}: {e}");
        self.send(S::LOCAL_ERROR).await?;
      }
    }
    Ok(())
  }
}
