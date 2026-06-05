use log::warn;

use crate::{
  S,
  send_to::SendTo,
  session::{Action, parse_address},
};

/// 处理通用SMTP命令
///
/// 这些命令在转发和发送会话中的行为完全相同：
/// - DATA: 检查状态并开始数据传输
/// - RSET: 重置会话状态
/// - NOOP: 无操作
/// - QUIT: 退出会话
///
/// 对于未识别的命令返回500错误
/// 注意：EHLO/HELO/AUTH/STARTTLS/MAIL/RCPT 命令不应该调用此函数
pub fn handle_cmd(cmd: &str, send_to: &mut SendTo, port: u16) -> Action {
  match cmd {
    "DATA" => {
      if !send_to.is_ready() {
        S::BAD_SEQUENCE_MAIL_RCPT.into()
      } else {
        Action::Data(S::DATA_START.to_string())
      }
    }
    "RSET" => {
      send_to.reset();
      Action::Ok
    }
    "NOOP" => Action::Ok,
    "QUIT" => Action::Quit(S::QUIT.to_string()),
    _ => {
      warn!("port {}: Unrecognized Command: {}", port, cmd);
      S::COMMAND_NOT_RECOGNIZED.into()
    }
  }
}

/// 转发会话的RCPT TO处理
pub fn handle_forward_rcpt(args: &str, send_to: &mut SendTo) -> Action {
  if send_to.has_sender() {
    if let Some(rcpt) = parse_address(args) {
      send_to.add_to(rcpt);
      Action::Ok
    } else {
      S::INVALID_RCPT_SYNTAX.into()
    }
  } else {
    S::BAD_SEQUENCE_MAIL.into()
  }
}

/// MAIL FROM command handling for authenticated sending sessions
/// 发送会话的MAIL FROM处理（需要认证）
pub fn handle_send_mail(
  args: &str,
  send_to: &mut SendTo,
  authenticated: bool,
  auth_email: &str,
) -> Action {
  if !authenticated {
    return S::AUTH_REQUIRED.into();
  }

  if let Some(sender) = parse_address(args) {
    if !sender.eq_ignore_ascii_case(auth_email) {
      return S::SENDER_REJECTED.into();
    }
    send_to.set_sender(sender);
    Action::Ok
  } else {
    S::INVALID_MAIL_SYNTAX.into()
  }
}

/// 发送会话的RCPT TO处理（需要认证）
pub fn handle_send_rcpt(args: &str, send_to: &mut SendTo, authenticated: bool) -> Action {
  if !authenticated {
    return S::AUTH_REQUIRED.into();
  }

  if send_to.has_sender() {
    if let Some(rcpt) = parse_address(args) {
      send_to.add_to(rcpt);
      Action::Ok
    } else {
      S::INVALID_RCPT_SYNTAX.into()
    }
  } else {
    S::BAD_SEQUENCE_MAIL.into()
  }
}
