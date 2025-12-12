#![cfg_attr(docsrs, feature(doc_cfg))]
//! # SMTP服务器实现
//!
//! 本库提供了一个完整的SMTP服务器实现，支持以下特性：
//! - **Implicit TLS (SMTPS)**: 所有连接必须使用TLS加密
//! - **SNI支持**: 通过SNI自动选择对应域名的证书
//! - **SMTP认证**: 支持AUTH PLAIN和AUTH LOGIN
//! - **Pipeline支持**: 实现RFC 2920 SMTP命令流水线
//! - **异步I/O**: 基于tokio的高性能异步处理
//!
//! ## 使用示例
//!
//! ```ignore
//! use smtp_recv::{run, Mailer, Mail, Result};
//!
//! struct MyMailer;
//!
//! impl Mailer for MyMailer {
//!     async fn send(&self, mail: Mail) -> Result<()> {
//!         // 处理邮件
//!         Ok(())
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // 运行SMTP服务器（需要实现认证和SSL）
//!     run(465, my_auth, MyMailer, my_ssl).await
//! }
//! ```

use std::sync::Arc;

mod accept;
pub use accept::accept;
mod bind;
pub use bind::bind;
mod conn;
pub use conn::conn;
mod error;
mod mailer;
mod session;

pub use error::{Result, SmtpError};
pub use mailer::Mailer;
use ssl_trait::CertByHost;

/// 运行SMTP服务器
///
/// # 参数
///
/// - `port`: 监听端口（标准SMTPS端口为465）
/// - `auth`: 认证服务实现，用于验证用户凭证
/// - `mailer`: 邮件发送处理器，用于实际发送邮件
/// - `ssl`: SSL证书提供者，根据SNI提供对应的证书
///
/// # 特性
///
/// - 使用Implicit TLS（连接时立即开始TLS握手）
/// - 支持SNI (Server Name Indication)
/// - 每个连接15分钟超时
/// - 自动并发处理多个客户端连接
///
/// # 错误
///
/// 当监听器无法绑定到指定端口时返回错误
pub async fn run<A: auth_trait::Auth, M: Mailer>(
  port: u16,
  auth: A,
  mailer: impl Into<Arc<M>>,
  ssl: impl CertByHost,
) -> Result<()> {
  let mailer = mailer.into();
  let listener = socket_port::listen(port)?;
  bind(
    tokio::net::TcpListener::from_std(listener)?,
    auth,
    mailer,
    ssl,
  )
  .await
}
