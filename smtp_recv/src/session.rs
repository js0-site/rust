use std::{str::FromStr, sync::Arc};

use auth_trait::Auth;
use email_address::EmailAddress;
use mail_struct::UserMail;
use tokio::{
  io::{AsyncBufReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf},
  net::TcpStream,
};

use crate::{error::Result, mailer::Mailer};

/// SMTP会话结构，管理单个客户端连接的完整生命周期
pub struct Session<A: Auth, M: Mailer> {
  /// TLS流的写入端
  writer: WriteHalf<tokio_rustls::TlsStream<TcpStream>>,
  /// TLS流的读取端（带缓冲）
  reader: BufReader<ReadHalf<tokio_rustls::TlsStream<TcpStream>>>,
  /// 认证服务
  auth: A,
  /// 邮件发送服务
  mailer: Arc<M>,
  /// 服务器主机名
  host: String,
  /// 当前会话是否已认证
  authenticated: bool,
  /// 当前认证用户的ID
  user_id: u64,
  to_li: Vec<String>,
  sender: Option<String>,
}

impl<A: Auth, M: Mailer> Session<A, M> {
  /// 创建新的SMTP会话
  pub fn new(
    stream: tokio_rustls::TlsStream<TcpStream>,
    auth: A,
    mailer: Arc<M>,
    host: String,
  ) -> Self {
    let (reader, writer) = tokio::io::split(stream);
    Self {
      writer,
      reader: BufReader::new(reader),
      auth,
      mailer,
      host,
      authenticated: false,
      user_id: 0,
      sender: None,
      to_li: Vec::new(),
    }
  }

  /// 向客户端发送SMTP响应（自动添加CRLF）
  async fn send(&mut self, msg: &str) -> Result<()> {
    self.writer.write_all(msg.as_bytes()).await?;
    self.writer.write_all(b"\r\n").await?;
    Ok(())
  }

  /// 运行SMTP会话主循环，处理客户端命令
  ///
  /// 支持SMTP Pipelining (RFC 2920)，允许客户端在不等待响应的情况下发送多个命令
  pub async fn run(mut self) -> Result<()> {
    // 发送欢迎消息 (RFC 5321: 220 服务就绪)
    self
      .send(&format!("220 {} SMTP Server Ready", self.host))
      .await?;

    let mut line = String::new();
    let mut commands = Vec::new();
    let mut responses = Vec::new();

    loop {
      // 为新的命令批次清空缓冲区
      commands.clear();
      responses.clear();

      // 至少读取一条命令
      line.clear();
      if self.reader.read_line(&mut line).await? == 0 {
        // 客户端关闭连接
        break;
      }

      let trimmed = line.trim();
      if !trimmed.is_empty() {
        commands.push(trimmed.to_string());
      }

      // 尝试读取更多命令以支持Pipeline（非阻塞）
      // 使用更高效的方式检查缓冲区是否有完整命令
      while self.has_buffered_line().await {
        line.clear();
        if self.reader.read_line(&mut line).await? == 0 {
          break;
        }

        let trimmed = line.trim();
        if !trimmed.is_empty() {
          commands.push(trimmed.to_string());
        }
      }

      // 处理所有命令并收集响应
      let mut should_quit = false;
      let mut flush_before_data = false;

      for cmd_line in &commands {
        let (cmd, args) = Self::parse_command(cmd_line);

        let response = match cmd.as_str() {
          "EHLO" | "HELO" => self.handle_ehlo(),
          "AUTH" => {
            // AUTH需要交互，先刷新已有响应
            self.flush_responses(&responses).await?;
            responses.clear();
            self.handle_auth(args).await?;
            continue; // 已发送响应
          }
          "MAIL" => self.handle_mail(args),
          "RCPT" => self.handle_rcpt(args),
          "DATA" => {
            // DATA需要交互，先刷新所有待发响应
            flush_before_data = true;
            "354 End data with <CR><LF>.<CR><LF>".to_string()
          }
          "RSET" => {
            self.reset_state();
            "250 OK".to_string()
          }
          "NOOP" => "250 OK".to_string(),
          "QUIT" => {
            should_quit = true;
            "221 Bye".to_string()
          }
          _ => "500 Command not recognized".to_string(),
        };

        responses.push(response);

        // 遇到DATA命令，需要立即刷新并处理
        if flush_before_data {
          break;
        }
      }

      // 发送所有响应
      self.flush_responses(&responses).await?;

      // 如果批次中有DATA命令，处理邮件内容
      if flush_before_data {
        self.handle_data_content().await?;
      }

      if should_quit {
        break;
      }
    }
    Ok(())
  }

  /// 检查缓冲区中是否有完整的行（非阻塞方式）
  ///
  /// 使用零超时来实现非阻塞检查，如果数据已在缓冲区中则立即返回true
  async fn has_buffered_line(&mut self) -> bool {
    // 使用零超时的fill_buf来非阻塞地检查缓冲区
    // 如果缓冲区中已有数据，会立即返回；否则超时返回
    match tokio::time::timeout(std::time::Duration::from_millis(0), self.reader.fill_buf()).await {
      Ok(Ok(buf)) => buf.contains(&b'\n'),
      _ => false,
    }
  }

  /// 解析SMTP命令行，分离命令和参数
  /// 命令自动转换为大写以便匹配
  fn parse_command(line: &str) -> (String, &str) {
    match line.split_once(' ') {
      Some((cmd, args)) => (cmd.to_uppercase(), args),
      None => (line.to_uppercase(), ""),
    }
  }

  /// 批量发送响应
  async fn flush_responses(&mut self, responses: &[String]) -> Result<()> {
    for response in responses {
      self.send(response).await?;
    }
    Ok(())
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
      _ => {
        self
          .send("504 Authentication mechanism not supported")
          .await
      }
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
      self.send("334 ").await?;
      let mut line = String::new();
      self.reader.read_line(&mut line).await?;
      decode_auth_plain(&line)
    };

    if let Some((username, password)) = credentials {
      self.verify_auth(username, password).await
    } else {
      self.send("501 Invalid AUTH PLAIN encoding").await
    }
  }

  /// 处理AUTH LOGIN认证
  /// 非标准但广泛支持的认证方式
  async fn handle_auth_login(&mut self) -> Result<()> {
    // 请求用户名 (Base64编码的"Username:")
    self.send("334 VXNlcm5hbWU6").await?;
    let username = self.read_base64_response().await?;

    // 请求密码 (Base64编码的"Password:")
    self.send("334 UGFzc3dvcmQ6").await?;
    let password = self.read_base64_response().await?;

    self.verify_auth(username, password).await
  }

  /// 读取并解码Base64响应
  async fn read_base64_response(&mut self) -> Result<String> {
    let mut line = String::new();
    self.reader.read_line(&mut line).await?;
    Ok(decode_base64(&line).unwrap_or_default())
  }

  /// 验证用户凭证
  async fn verify_auth(&mut self, username: String, password: String) -> Result<()> {
    if let Some(user_id) = self.auth.verify(&self.host, &username, &password).await? {
      self.authenticated = true;
      self.user_id = user_id;
      self.send("235 Authentication successful").await
    } else {
      self.send("535 Authentication failed").await
    }
  }

  /// 处理MAIL FROM命令
  /// RFC 5321: 指定邮件发件人
  fn handle_mail(&mut self, args: &str) -> String {
    if !self.authenticated {
      return "530 Authentication required".to_string();
    }

    if let Some(sender) = parse_smtp_address(args) {
      self.sender = Some(sender);
      "250 OK".to_string()
    } else {
      "501 Invalid MAIL FROM syntax".to_string()
    }
  }

  /// 处理RCPT TO命令
  /// RFC 5321: 指定邮件收件人（可多次调用）
  fn handle_rcpt(&mut self, args: &str) -> String {
    if !self.authenticated {
      return "530 Authentication required".to_string();
    }

    if self.sender.is_some() {
      if let Some(rcpt) = parse_smtp_address(args) {
        self.to_li.push(rcpt);
        "250 OK".to_string()
      } else {
        "501 Invalid RCPT TO syntax".to_string()
      }
    } else {
      "503 Bad sequence: need MAIL first".to_string()
    }
  }

  /// 处理DATA命令的内容部分
  /// RFC 5321: 接收邮件内容，以单独的"."行结束
  async fn handle_data_content(&mut self) -> Result<()> {
    if !self.authenticated {
      self.send("530 Authentication required").await?;
      return Ok(());
    }

    if self.sender.is_none() || self.to_li.is_empty() {
      self
        .send("503 Bad sequence: need MAIL and RCPT first")
        .await?;
      return Ok(());
    }

    self.read_and_send_mail().await
  }

  /// 读取邮件内容并发送
  async fn read_and_send_mail(&mut self) -> Result<()> {
    let mut body = Vec::new();
    let mut line = String::new();

    // 读取邮件内容直到遇到"."结束标记
    loop {
      line.clear();
      self.reader.read_line(&mut line).await?;

      // 检查结束标记（RFC 5321: 单独一行的"."）
      if line == ".\r\n" || line == ".\n" {
        break;
      }

      // 处理点号透明转换（RFC 5321: 4.5.2）
      // 如果行首是".."，去掉一个"."
      if line.starts_with("..") {
        body.extend_from_slice(&line.as_bytes()[1..]);
      } else {
        body.extend_from_slice(line.as_bytes());
      }
    }

    if let Some(sender) = self.sender.take() {
      if let Some(mail) = mail_struct::Mail::new(&sender, std::mem::take(&mut self.to_li), body) {
        let user_mail = UserMail {
          user_id: self.user_id,
          mail,
        };
        let mailer = self.mailer.clone();
        if let Err(err) = mailer.send(user_mail).await {
          log::error!("{sender}: {err}");
          self
            .send("451 Requested action aborted: local error in processing")
            .await?;
          return Ok(());
        }
      };
    } else {
      self.to_li.clear();
    }

    // 发送成功响应
    self.send("250 Message accepted").await?;
    Ok(())
  }

  /// 重置会话状态（用于RSET命令或DATA完成后）
  fn reset_state(&mut self) {
    self.to_li.clear();
    self.sender = None;
  }
}

/// 解码AUTH PLAIN凭证
/// 格式: base64(authzid\0authcid\0passwd)
/// RFC 4616
fn decode_auth_plain(encoded: &str) -> Option<(String, String)> {
  let decoded = decode_base64(encoded)?;
  let parts: Vec<&str> = decoded.splitn(3, '\0').collect();
  if parts.len() == 3 {
    // parts[0] = authzid (通常为空)
    // parts[1] = authcid (用户名)
    // parts[2] = passwd (密码)
    Some((parts[1].to_string(), parts[2].to_string()))
  } else {
    None
  }
}

/// 解码Base64字符串
fn decode_base64(encoded: &str) -> Option<String> {
  use base64::Engine;
  let decoded = base64::engine::general_purpose::STANDARD
    .decode(encoded.trim())
    .ok()?;
  String::from_utf8(decoded).ok()
}

/// 从SMTP命令参数中解析邮箱地址
/// 支持格式: FROM:<addr> 或 TO:<addr>
/// RFC 5321: 信封地址只是纯邮箱，不含显示名
fn parse_smtp_address(args: &str) -> Option<String> {
  // 提取"FROM:"或"TO:"后的地址部分
  let addr_part = args
    .trim()
    .split_once(':')
    .map(|(_, addr)| addr.trim_start())
    .unwrap_or(args);

  // 去除尖括号
  let addr_clean = addr_part.trim_matches(|c| c == '<' || c == '>').trim();

  if addr_clean.is_empty() {
    return None;
  }

  // 验证邮箱地址格式
  if let Ok(email) = EmailAddress::from_str(addr_clean) {
    Some(email.to_string())
  } else {
    None
  }
}
