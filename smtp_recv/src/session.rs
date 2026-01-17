use std::str::FromStr;

use email_address::EmailAddress;

use crate::{Result, S, Stream};

/// 从SMTP命令参数中解析邮箱地址
/// 支持格式: FROM:<addr> 或 TO:<addr>
/// RFC 5321: 信封地址只是纯邮箱，不含显示名
pub fn parse_address(args: &str) -> Option<String> {
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

/// 解析SMTP命令行，分离命令和参数
/// 命令自动转换为大写以便匹配
pub fn parse_command(line: &str) -> (String, &str) {
  match line.split_once(' ') {
    Some((cmd, args)) => (cmd.to_uppercase(), args),
    None => (line.to_uppercase(), ""),
  }
}

/// 解码AUTH PLAIN凭证
/// 格式: base64(authzid\0authcid\0passwd)
/// RFC 4616
#[cfg(feature = "send")]
pub fn decode_auth_plain(encoded: &str) -> Option<(String, String)> {
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
#[cfg(feature = "send")]
pub fn decode_base64(encoded: &str) -> Option<String> {
  use base64::Engine;
  let decoded = base64::engine::general_purpose::STANDARD
    .decode(encoded.trim())
    .ok()?;
  String::from_utf8(decoded).ok()
}

/// 读取邮件内容（处理以.结束的多行数据）
/// RFC 5321: 4.5.2 透明性处理
pub async fn read_mail(stream: &mut impl Stream) -> Result<Vec<u8>> {
  let mut body = Vec::new();
  let mut line = String::new();

  // 读取邮件内容直到遇到"."结束标记
  loop {
    line.clear();
    stream.read_line(&mut line).await?;

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
  Ok(body)
}

pub enum Action {
  Reply(String),
  /// 命令已处理（包括可能的IO和响应发送），无需进一步操作
  Handled,
  /// 标准成功响应 "250 OK"
  Ok,
  Quit(String),
  /// DATA命令，包含响应消息
  Data(String),
}

impl From<&str> for Action {
  fn from(msg: &str) -> Self {
    Action::Reply(msg.to_string())
  }
}

impl From<String> for Action {
  fn from(msg: String) -> Self {
    Action::Reply(msg)
  }
}

pub trait Session {
  type Stream: crate::Stream;
  fn stream(&mut self) -> &mut Self::Stream;

  async fn read_line(&mut self, buf: &mut String) -> Result<usize> {
    self.stream().read_line(buf).await
  }

  async fn buf_line(&mut self) -> bool {
    self.stream().buf_line().await
  }

  async fn flush(&mut self, responses: &[String]) -> Result<()> {
    self.stream().flush(responses).await
  }

  async fn send_welcome(&mut self) -> Result<()> {
    self.stream().send(S::WELCOME).await
  }

  /// 处理单个命令
  async fn handle_cmd(
    &mut self,
    cmd: &str,
    args: &str,
    pending_responses: &mut Vec<String>,
  ) -> Result<Action>;

  async fn handle_data(&mut self) -> Result<()>;
}

/// 运行SMTP会话主循环
pub async fn run_loop<T: Session + ?Sized>(session: &mut T) -> Result<()> {
  // 发送欢迎消息
  session.send_welcome().await?;

  let mut line = String::new();
  let mut commands = Vec::new();
  let mut responses = Vec::new();

  loop {
    // 为新的命令批次清空缓冲区
    commands.clear();
    responses.clear();

    // 至少读取一条命令
    line.clear();
    if session.read_line(&mut line).await? == 0 {
      // 客户端关闭连接
      break;
    }

    let trimmed = line.trim();
    if !trimmed.is_empty() {
      commands.push(trimmed.to_string());
    }

    // 尝试读取更多命令以支持Pipeline（非阻塞）
    while session.buf_line().await {
      line.clear();
      if session.read_line(&mut line).await? == 0 {
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
      let (cmd, args) = parse_command(cmd_line);
      let action = session.handle_cmd(&cmd, args, &mut responses).await?;

      match action {
        Action::Reply(s) => responses.push(s),
        Action::Handled => {}
        Action::Ok => responses.push(S::OK.to_string()),
        Action::Quit(s) => {
          responses.push(s);
          should_quit = true;
        }
        Action::Data(s) => {
          responses.push(s);
          flush_before_data = true;
        }
      }

      // 遇到DATA命令，需要立即刷新并处理
      if flush_before_data {
        break;
      }
    }

    // 发送所有响应
    session.flush(&responses).await?;

    // 如果批次中有DATA命令，处理邮件内容
    if flush_before_data {
      session.handle_data().await?;
    }

    if should_quit {
      break;
    }
  }
  Ok(())
}
