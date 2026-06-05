use std::{error, io, result};

use thiserror::Error;

/// SMTP服务器错误类型
///
/// 包含SMTP服务器运行过程中可能遇到的各种错误
#[derive(Error, Debug)]
pub enum SmtpError {
  /// I/O错误（网络读写等）
  #[error("IO error: {0}")]
  Io(#[from] io::Error),

  /// TLS相关错误（握手、加密等）
  #[error("TLS error: {0}")]
  Tls(#[from] rustls::Error),

  /// 证书错误（证书加载、验证等）
  #[error("Certificate error: {0}")]
  Certificate(#[source] Box<dyn error::Error + Send + Sync>),

  /// 认证错误（用户验证失败等）
  #[error("Authentication error: {0}")]
  Auth(#[source] anyhow::Error),

  /// 通用错误
  #[error(transparent)]
  Anyhow(#[from] anyhow::Error),

  /// ClientHello中缺少SNI（Server Name Indication）
  /// 这是SMTPS必需的，用于选择正确的SSL证书
  #[error("No SNI in ClientHello")]
  NoSni,

  /// 指定的主机名没有对应的SSL证书
  #[error("No certificate found for host: {0}")]
  NoCertificate(String),

  /// 邮箱地址格式无效
  #[error("Invalid email address")]
  InvalidEmail,

  /// 未认证就尝试执行需要认证的命令
  #[error("Authentication required")]
  AuthRequired,

  /// MAIL FROM命令语法错误
  #[error("Invalid MAIL FROM syntax")]
  InvalidMailFrom,

  /// RCPT TO命令语法错误
  #[error("Invalid RCPT TO syntax")]
  InvalidRcptTo,

  /// SMTP命令序列错误（如：在MAIL之前执行RCPT）
  #[error("Bad sequence: {0}")]
  BadSequence(String),

  /// AUTH PLAIN编码无效
  #[error("Invalid AUTH PLAIN encoding")]
  InvalidAuthPlain,

  /// 不支持的认证机制
  #[error("Authentication mechanism not supported")]
  AuthMechanismNotSupported,

  /// 无法识别的SMTP命令
  #[error("Command not recognized")]
  CommandNotRecognized,

  /// TLS流类型错误（期望服务器流但得到客户端流）
  #[error("Invalid TLS stream type: expected server stream")]
  InvalidTlsStreamType,

  /// 流已被取走（用于 STARTTLS 升级过程中的临时状态）
  #[error("Stream is None")]
  StreamNone,

  /// SPF验证失败
  #[error("SPF verification failed")]
  SpfFail,
}

/// SMTP操作的Result类型别名
pub type Result<T> = result::Result<T, SmtpError>;
