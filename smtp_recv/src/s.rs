pub struct S;

impl S {
  // 成功响应
  pub const OK: &'static str = "250 OK";
  pub const MESSAGE_ACCEPTED: &'static str = "250 Message accepted";
  pub const MESSAGE_ACCEPTED_FORWARDING: &'static str = "250 Message accepted for forwarding";
  pub const AUTH_SUCCESS: &'static str = "235 Authentication successful";
  pub const DATA_START: &'static str = "354 End data with <CR><LF>.<CR><LF>";
  pub const QUIT: &'static str = "221 Bye";
  pub const STARTTLS_READY: &'static str = "220 Ready to start TLS";
  pub const WELCOME: &'static str = "220 SMTP Server Ready";

  // 客户端错误 (4xx)
  pub const AUTH_REQUIRED: &'static str = "530 Authentication required";
  pub const AUTH_FAILED: &'static str = "535 Authentication failed";
  pub const INVALID_MAIL_SYNTAX: &'static str = "501 Invalid MAIL FROM syntax";
  pub const SENDER_REJECTED: &'static str = "550 Sender address rejected: not owned by auth user";
  pub const INVALID_RCPT_SYNTAX: &'static str = "501 Invalid RCPT TO syntax";
  pub const INVALID_AUTH_PLAIN: &'static str = "501 Invalid AUTH PLAIN encoding";
  pub const BAD_SEQUENCE_MAIL_RCPT: &'static str = "503 Bad sequence: need MAIL and RCPT first";
  pub const BAD_SEQUENCE_MAIL: &'static str = "503 Bad sequence: need MAIL first";
  pub const TLS_ALREADY_ACTIVE: &'static str = "503 TLS already active";
  pub const AUTH_MECHANISM_NOT_SUPPORTED: &'static str =
    "504 Authentication mechanism not supported";

  // 服务器错误 (5xx)
  pub const COMMAND_NOT_RECOGNIZED: &'static str = "500 Command not recognized";
  pub const NO_SUCH_USER: &'static str = "550 No such user";
  pub const LOCAL_ERROR: &'static str = "451 Requested action aborted: local error in processing";
  /// RFC 7208 Section 8.4 - SPF fail 应返回 550
  pub const SPF_FAIL: &'static str = "550 5.7.23 SPF validation failed";
  /// RFC 7208 Section 8.6 - SPF temperror 应返回 451
  pub const SPF_TEMP_ERROR: &'static str = "451 4.7.24 SPF validation error";

  // Base64编码的认证提示
  pub const AUTH_USERNAME_PROMPT: &'static str = "334 VXNlcm5hbWU6"; // "Username:"
  pub const AUTH_PASSWORD_PROMPT: &'static str = "334 UGFzc3dvcmQ6"; // "Password:"
  pub const AUTH_PLAIN_PROMPT: &'static str = "334 ";
}
