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

use std::{
  net::{Ipv6Addr, SocketAddr},
  sync::Arc,
};

mod bind;
mod conn;
mod error;
mod mailer;
mod session;

pub use error::{Result, SmtpError};
pub use mailer::Mailer;
use ssl_trait::CertByHost;
use tokio::net::TcpListener;

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
  let mut tasks = vec![];

  let ipv6_addr = (Ipv6Addr::UNSPECIFIED, port).into();
  let ipv6_is_dual_stack = match TcpListener::bind(ipv6_addr).await {
    Ok(listener) => {
      let is_dual = !is_ipv6_only(&listener);
      log::info!(
        "IPv6 {} mode",
        if is_dual {
          "dual-stack (IPv4+IPv6)"
        } else {
          "only"
        }
      );
      tasks.push(spawn_listener(
        listener,
        ipv6_addr,
        auth.clone(),
        mailer.clone(),
        ssl.clone(),
      ));
      is_dual
    }
    Err(e) => {
      log::warn!("Failed to bind IPv6 {ipv6_addr}: {e}");
      false
    }
  };

  if !ipv6_is_dual_stack {
    let ipv4_addr = ([0, 0, 0, 0], port).into();
    match TcpListener::bind(ipv4_addr).await {
      Ok(listener) => {
        tasks.push(spawn_listener(listener, ipv4_addr, auth, mailer, ssl));
      }
      Err(e) => {
        log::warn!("Failed to bind IPv4 {ipv4_addr}: {e}");
      }
    }
  }

  if tasks.is_empty() {
    return Err(SmtpError::Io(std::io::Error::new(
      std::io::ErrorKind::AddrNotAvailable,
      "Failed to bind to any address",
    )));
  }

  for task in tasks {
    task
      .await
      .map_err(|e| SmtpError::Io(std::io::Error::other(format!("Task join error: {e}"))))??;
  }
  Ok(())
}

fn spawn_listener<A: auth_trait::Auth, M: Mailer>(
  listener: TcpListener,
  addr: SocketAddr,
  auth: A,
  mailer: Arc<M>,
  ssl: impl CertByHost,
) -> tokio::task::JoinHandle<Result<()>> {
  tokio::spawn(async move { bind::bind(listener, addr, auth, mailer, ssl).await })
}

/// 检查 IPv6 socket 是否设置了 IPV6_V6ONLY
/// 返回 true 表示只支持 IPv6，false 表示 dual-stack（同时支持 IPv4 和 IPv6）
fn is_ipv6_only(listener: &TcpListener) -> bool {
  use std::os::fd::{AsRawFd, FromRawFd};

  use socket2::Socket;

  let fd = listener.as_raw_fd();
  let socket = unsafe { Socket::from_raw_fd(fd) };
  let result = socket.only_v6().unwrap_or(false);
  std::mem::forget(socket); // 避免关闭 fd
  result
}
