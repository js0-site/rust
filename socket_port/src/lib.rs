#![cfg_attr(docsrs, feature(doc_cfg))]

use std::{
  io::Result,
  net::{Ipv6Addr, SocketAddr, TcpListener},
};

use socket2::{Domain, Socket, Type};

#[cfg(target_os = "linux")]
mod listen_fd;
#[cfg(target_os = "linux")]
pub use listen_fd::listen_fd;

pub fn listen(port: u16) -> Result<TcpListener> {
  // =======================================================================
  // 1. Systemd Socket Activation / 热重启逻辑
  // =======================================================================
  // 优先检查是否存在 Systemd 传递进来的 FD / Check for systemd socket activation first
  #[cfg(target_os = "linux")]
  if let Some(listener) = listen_fd(port)? {
    return Ok(listener);
  }

  // =======================================================================
  // 2. 冷启动 / 本地开发逻辑 (Socket2 手动 Bind)
  // =======================================================================
  // 如果没有继承到 FD，说明是第一次冷启动（非 Systemd 环境）或 Systemd 未配置 Socket。
  // 我们手动创建一个支持双栈 (IPv4 + IPv6) 的 Socket。
  // =======================================================================
  // 创建 IPv6 Socket (Linux 上 IPv6 socket 默认可以兼容 IPv4)

  #[cfg(feature = "kill_port")]
  kill_port::kill_port(port);

  let socket = Socket::new(Domain::IPV6, Type::STREAM, None)?;

  // 设置为双栈模式 (Dual Stack)
  // false = 既处理 IPv6 也处理 IPv4 (映射为 IPv6)
  socket.set_only_v6(false)?;

  // 允许地址和端口重用，防止 TIME_WAIT 导致 Address already in use
  socket.set_reuse_address(true)?;
  #[cfg(not(target_os = "windows"))]
  socket.set_reuse_port(true)?;

  // 设置非阻塞模式 (Tokio 需要)
  socket.set_nonblocking(true)?;

  // 绑定地址 [::]:port (同时覆盖 0.0.0.0:port)
  let addr = SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), port);
  socket.bind(&addr.into())?;

  // 开始监听，backlog 设置为 1024
  socket.listen(1024)?;

  Ok(socket.into())
}
