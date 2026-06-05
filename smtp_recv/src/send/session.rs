use auth_trait::Auth;
use mail_struct::UserMail;
use tokio::{
  io::{BufReader, split},
  net::TcpStream,
};
use tokio_rustls::TlsStream as RustlsTlsStream;

use crate::{
  Mailer, Result, S, SmtpError, Stream,
  cmd::{handle_cmd, handle_send_mail, handle_send_rcpt},
  send_to::SendTo,
  session::{self, Action, decode_auth_plain, decode_base64, read_mail},
  stream::TlsStream,
};

/// SMTP会话结构，管理单个客户端连接的完整生命周期
pub struct Session<A: Auth, M: Mailer> {
  /// TLS流
  stream: TlsStream,
  /// 认证服务
  auth: A,
  /// 邮件发送服务
  mailer: M,
  /// 服务器主机名
  host: String,
  /// 当前会话是否已认证
  authenticated: bool,
  /// 当前认证用户的ID
  user_id: u64,
  /// Login email of the current authenticated user
  /// 当前认证用户的登录邮箱
  email: String,
  /// 邮件发送目标信息
  send_to: SendTo,
}

impl<A: Auth, M: Mailer> session::Session for Session<A, M> {
  type Stream = TlsStream;

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
      "EHLO" | "HELO" => Ok(Action::Reply(self.handle_ehlo())),
      "AUTH" => {
        // AUTH需要交互，先刷新已有响应
        self.stream.flush(pending_responses).await?;
        pending_responses.clear();
        self.handle_auth(args).await?;
        Ok(Action::Handled)
      }
      "MAIL" => Ok(handle_send_mail(
        args,
        &mut self.send_to,
        self.authenticated,
        &self.email,
      )),
      "RCPT" => Ok(handle_send_rcpt(
        args,
        &mut self.send_to,
        self.authenticated,
      )),
      _ => Ok(handle_cmd(cmd, &mut self.send_to, 465)),
    }
  }

  async fn handle_data(&mut self) -> Result<()> {
    self.read_and_send_mail().await
  }
}

impl<A: Auth, M: Mailer> Session<A, M> {
  /// 创建新的SMTP会话
  pub fn load(
    stream: RustlsTlsStream<TcpStream>,
    auth: A,
    mailer: M,
    host: String,
  ) -> Result<Self> {
    // 提取服务器端TLS流
    let server_stream = match stream {
      RustlsTlsStream::Server(s) => s,
      RustlsTlsStream::Client(_) => {
        return Err(SmtpError::InvalidTlsStreamType);
      }
    };

    let (reader, writer) = split(server_stream);
    Ok(Self {
      stream: TlsStream {
        reader: BufReader::new(reader),
        writer,
      },
      auth,
      mailer,
      host,
      authenticated: false,
      user_id: 0,
      email: String::new(),
      send_to: SendTo::new(),
    })
  }

  /// 向客户端发送SMTP响应（自动添加CRLF）
  async fn send(&mut self, msg: &str) -> Result<()> {
    self.stream.send(msg).await
  }

  /// 运行SMTP会话主循环，处理客户端命令
  ///
  /// 支持SMTP Pipelining (RFC 2920)，允许客户端在不等待响应的情况下发送多个命令
  pub async fn run(mut self) -> Result<()> {
    session::run_loop(&mut self).await
  }

  /// 处理EHLO/HELO命令
  /// RFC 5321: EHLO用于扩展SMTP，HELO用于标准SMTP
  fn handle_ehlo(&self) -> String {
    format!(
      "250-{}\r\n250-AUTH PLAIN LOGIN\r\n250-PIPELINING\r\n250 8BITMIME",
      self.host
    )
  }

  /// 处理AUTH命令
  /// RFC 4954: SMTP认证扩展
  async fn handle_auth(&mut self, args: &str) -> Result<()> {
    let (mechanism, param) = args.split_once(' ').unwrap_or((args, ""));
    match mechanism.to_uppercase().as_str() {
      "PLAIN" => self.handle_auth_plain(param).await,
      "LOGIN" => self.handle_auth_login().await,
      _ => self.send(S::AUTH_MECHANISM_NOT_SUPPORTED).await,
    }
  }

  /// 处理AUTH PLAIN认证
  /// RFC 4616: PLAIN SASL机制
  async fn handle_auth_plain(&mut self, initial_response: &str) -> Result<()> {
    let credentials = if !initial_response.is_empty() {
      // 客户端在命令中直接提供凭证
      decode_auth_plain(initial_response)
    } else {
      // 请求客户端提供凭证
      self.send(S::AUTH_PLAIN_PROMPT).await?;
      let mut line = String::new();
      self.stream.read_line(&mut line).await?;
      decode_auth_plain(&line)
    };

    if let Some((username, password)) = credentials {
      self.verify_auth(username, password).await
    } else {
      self.send(S::INVALID_AUTH_PLAIN).await
    }
  }

  /// 处理AUTH LOGIN认证
  /// 非标准但广泛支持的认证方式
  async fn handle_auth_login(&mut self) -> Result<()> {
    // 请求用户名 (Base64编码的"Username:")
    self.send(S::AUTH_USERNAME_PROMPT).await?;
    let username = self.read_base64_response().await?;

    // 请求密码 (Base64编码的"Password:")
    self.send(S::AUTH_PASSWORD_PROMPT).await?;
    let password = self.read_base64_response().await?;

    self.verify_auth(username, password).await
  }

  /// 读取并解码Base64响应
  async fn read_base64_response(&mut self) -> Result<String> {
    let mut line = String::new();
    self.stream.read_line(&mut line).await?;
    Ok(decode_base64(&line).unwrap_or_default())
  }

  /// 验证用户凭证
  async fn verify_auth(&mut self, username: String, password: String) -> Result<()> {
    if let Some(user_id) = self.auth.verify(&self.host, &username, &password).await? {
      self.authenticated = true;
      self.user_id = user_id;
      self.email = username;
      self.send(S::AUTH_SUCCESS).await
    } else {
      self.send(S::AUTH_FAILED).await
    }
  }

  /// 读取邮件内容并发送
  async fn read_and_send_mail(&mut self) -> Result<()> {
    // 读取邮件内容直到遇到"."结束标记
    let body = read_mail(&mut self.stream).await?;

    if let Some(sender) = self.send_to.take_sender()
      && let Some(mail) = mail_struct::Mail::new(&sender, self.send_to.take_to_li(), body)
    {
      let user_mail = UserMail {
        user_id: self.user_id,
        mail,
      };
      let mailer = self.mailer.clone();
      if let Err(err) = mailer.send(user_mail).await {
        log::error!("{sender}: {err}");
        self.send(S::LOCAL_ERROR).await?;
        return Ok(());
      }
    };

    // 发送成功响应
    self.send(S::MESSAGE_ACCEPTED).await?;
    Ok(())
  }
}
