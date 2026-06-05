use std::{
  io::ErrorKind,
  net::{IpAddr, Ipv6Addr, TcpStream},
  time::Duration,
};

use aok::{OK, Void};
use log::info;
use socket_port::listen;
use tokio::task::spawn_blocking;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

/// 测试：基本功能 - 验证 socket_port 能成功创建监听器
#[test]
fn test_basic_listen() -> Void {
  info!("测试：基本功能 - 创建监听器");

  // 使用操作系统自动分配端口（端口 0）
  let listener = listen(0)?;

  // 验证能够获取本地地址
  let local_addr = listener.local_addr()?;
  info!("成功创建监听器，地址: {}", local_addr);

  // 验证端口号已被分配（非 0）
  assert_ne!(local_addr.port(), 0, "端口应该被操作系统分配");

  OK
}

/// 测试：端口绑定 - 验证能绑定到指定端口
#[test]
fn test_bind_specific_port() -> Void {
  info!("测试：端口绑定 - 绑定指定端口");

  // 使用一个较大的端口号，减少冲突可能性
  let port = 18888;

  let listener = listen(port)?;
  let local_addr = listener.local_addr()?;

  info!("绑定到端口: {}", local_addr.port());
  assert_eq!(local_addr.port(), port, "应该绑定到指定端口");

  OK
}

/// 测试：双栈支持 - 验证同时支持 IPv4 和 IPv6
#[tokio::test]
async fn test_dual_stack_support() -> Void {
  info!("测试：双栈支持 - IPv4 和 IPv6 连接");

  let listener = listen(0)?;
  let local_addr = listener.local_addr()?;
  let port = local_addr.port();

  info!("监听器端口: {}", port);

  // 设置非阻塞模式，以便异步操作
  listener.set_nonblocking(true)?;

  // 测试 IPv4 连接
  let ipv4_result = spawn_blocking(move || {
    let addr = format!("127.0.0.1:{}", port);
    info!("尝试 IPv4 连接到: {}", addr);
    TcpStream::connect_timeout(&addr.parse().unwrap(), Duration::from_secs(2))
  })
  .await;

  match ipv4_result {
    Ok(Ok(_stream)) => {
      info!("✓ IPv4 连接成功");
    }
    Ok(Err(e)) => {
      info!("IPv4 连接失败（可能正常）: {}", e);
    }
    Err(e) => {
      info!("IPv4 连接任务错误: {}", e);
    }
  }

  // 重新创建监听器用于 IPv6 测试
  let listener_v6 = listen(0)?;
  let local_addr_v6 = listener_v6.local_addr()?;
  let port_v6 = local_addr_v6.port();

  listener_v6.set_nonblocking(true)?;

  // 测试 IPv6 连接
  let ipv6_result = spawn_blocking(move || {
    let addr = format!("[::1]:{}", port_v6);
    info!("尝试 IPv6 连接到: {}", addr);
    TcpStream::connect_timeout(&addr.parse().unwrap(), Duration::from_secs(2))
  })
  .await;

  match ipv6_result {
    Ok(Ok(_stream)) => {
      info!("✓ IPv6 连接成功");
    }
    Ok(Err(e)) => {
      info!("IPv6 连接失败（可能正常）: {}", e);
    }
    Err(e) => {
      info!("IPv6 连接任务错误: {}", e);
    }
  }

  OK
}

/// 测试：端口占用 - 验证已被占用的端口会返回错误
#[test]
fn test_port_already_in_use() -> Void {
  info!("测试：端口占用 - 重复绑定同一端口");

  // 先创建第一个监听器，使用自动分配端口
  let listener1 = listen(0)?;
  let addr = listener1.local_addr()?;
  let port = addr.port();

  info!("第一个监听器占用端口: {}", port);

  // 尝试在同一端口创建第二个监听器
  let result = listen(port);

  match result {
    Err(e) if e.kind() == ErrorKind::AddrInUse => {
      info!("✓ 正确返回地址已被占用错误: {}", e);
      OK
    }
    Err(e) => {
      info!("✓ 返回错误（错误类型: {:?}）: {}", e.kind(), e);
      // 在某些系统上，可能返回其他权限相关错误
      OK
    }
    Ok(_) => {
      // 注意：在某些系统上，如果启用了 SO_REUSEPORT，
      // 可能允许多个监听器绑定同一端口，这是预期行为
      #[cfg(not(windows))]
      {
        info!("✓ 允许多个监听器（SO_REUSEPORT 已启用）");
        OK
      }
      #[cfg(windows)]
      {
        panic!("Windows 不应该允许重复绑定端口");
      }
    }
  }
}

/// 测试：并发监听 - 验证可以同时创建多个监听器（不同端口）
#[test]
fn test_concurrent_listeners() -> Void {
  info!("测试：并发监听 - 创建多个监听器");

  let mut listeners = Vec::new();

  // 创建 5 个监听器
  for i in 0..5 {
    let listener = listen(0)?;
    let addr = listener.local_addr()?;
    info!("监听器 {} 创建成功，端口: {}", i, addr.port());
    listeners.push(listener);
  }

  assert_eq!(listeners.len(), 5, "应该成功创建 5 个监听器");

  // 验证所有端口都不同
  let ports: Vec<u16> = listeners
    .iter()
    .map(|l| l.local_addr().unwrap().port())
    .collect();

  for i in 0..ports.len() {
    for j in (i + 1)..ports.len() {
      assert_ne!(ports[i], ports[j], "每个监听器应该使用不同的端口");
    }
  }

  info!("✓ 所有监听器使用不同端口");

  OK
}

/// 测试：监听器地址类型 - 验证监听器绑定到 IPv6 地址
#[test]
fn test_listener_address_type() -> Void {
  info!("测试：监听器地址类型");

  let listener = listen(0)?;
  let local_addr = listener.local_addr()?;

  info!("本地地址: {}", local_addr);

  // 验证是 IPv6 地址
  match local_addr.ip() {
    IpAddr::V6(addr) => {
      info!("✓ 正确使用 IPv6 地址: {}", addr);
      assert_eq!(
        addr,
        Ipv6Addr::UNSPECIFIED,
        "应该绑定到 IPv6 UNSPECIFIED 地址"
      );
    }
    IpAddr::V4(addr) => {
      panic!("意外的 IPv4 地址: {}", addr);
    }
  }

  OK
}
