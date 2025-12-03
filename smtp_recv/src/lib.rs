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

use std::{borrow::Borrow, sync::Arc};

mod error;
mod mailer;
mod session;

pub use error::{Result, SmtpError};
use log::info;
pub use mail_struct::Mail;
pub use mailer::Mailer;
use rustls::ServerConfig;
use session::Session;
use ssl_trait::CertByHost;
use tokio::net::{TcpListener, TcpStream};

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
  let auth = Arc::new(auth);
  let addr = format!("0.0.0.0:{}", port);
  info!("SMTP {addr} with implicit TLS");
  let listener = TcpListener::bind(addr).await?;

  loop {
    // 接受新的客户端连接
    if let Ok((stream, addr)) = xerr::ok!(listener.accept().await) {
      let auth = auth.clone();
      let mailer = mailer.clone();
      let ssl = ssl.clone();

      // 为每个连接创建独立的task
      tokio::spawn(async move {
        // 设置15分钟连接超时
        let duration = std::time::Duration::from_secs(15 * 60);
        let result =
          tokio::time::timeout(duration, handle_connection(addr, stream, auth, mailer, ssl)).await;
        if let Ok(result) = result {
          if let Err(e) = result {
            log::error!("❌ {}: {}", addr, e);
          }
        } else {
          log::error!("❌ {}: connection timed out", addr);
        }
      });
    }
  }
}

/// 处理单个客户端连接
///
/// 工作流程：
/// 1. 进行TLS握手
/// 2. 从ClientHello中提取SNI
/// 3. 根据SNI获取对应的SSL证书
/// 4. 完成TLS握手
/// 5. 创建SMTP会话并开始处理命令
async fn handle_connection<A: auth_trait::Auth, M: Mailer>(
  addr: std::net::SocketAddr,
  stream: TcpStream,
  auth: Arc<A>,
  mailer: Arc<M>,
  ssl: impl CertByHost,
) -> Result<()> {
  // 启动惰性TLS接受器，等待ClientHello
  let acceptor = tokio_rustls::LazyConfigAcceptor::new(Default::default(), stream);

  let start = acceptor.await?;

  let client_hello = start.client_hello();

  // 提取SNI（Server Name Indication）
  let host = client_hello
    .server_name()
    .ok_or(SmtpError::NoSni)?
    .to_string();

  log::info!("→ {host} {addr}");

  // 根据SNI获取对应的SSL证书
  let ssl_config = ssl
    .get(&host)
    .await
    .map_err(|e| SmtpError::Certificate(e.into()))?
    .ok_or_else(|| SmtpError::NoCertificate(host.clone()))?;

  // 构建TLS配置
  let ssl_ref = ssl_config.borrow();
  let config = ServerConfig::builder()
    .with_no_client_auth()
    .with_single_cert(ssl_ref.cert.clone(), ssl_ref.key.clone_key())?;

  // 完成TLS握手
  let tls_stream = start.into_stream(Arc::new(config)).await?;

  // 创建并运行SMTP会话
  Session::new(
    tokio_rustls::TlsStream::Server(tls_stream),
    auth,
    mailer,
    host,
  )
  .run()
  .await
}
