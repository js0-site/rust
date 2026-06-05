use std::{
  env,
  io::Result,
  mem::{size_of, zeroed},
  net::TcpListener,
  os::unix::io::FromRawFd,
  sync::Mutex,
};

use log::{info, warn};
use once_cell::sync::Lazy;
use rapidhash::{HashMapExt, RapidHashMap as HashMap};

/// 缓存 systemd 传递的 FD，按端口号索引
/// Cache systemd FDs indexed by port number
static SYSTEMD_FDS: Lazy<Mutex<HashMap<u16, i32>>> = Lazy::new(|| {
  let mut map = HashMap::new();

  if let Ok(listen_fds) = env::var("LISTEN_FDS") {
    if let Ok(fd_count) = listen_fds.parse::<i32>() {
      info!("LISTEN_FDS={fd_count}");

      for i in 0..fd_count {
        let fd = 3 + i;

        // 验证 FD 是否有效
        if unsafe { libc::fcntl(fd, libc::F_GETFD) } < 0 {
          warn!("FD {fd} invalid");
          continue;
        }

        // 获取 socket 地址信息
        let mut addr: libc::sockaddr_storage = unsafe { zeroed() };
        let mut addr_len = size_of::<libc::sockaddr_storage>() as libc::socklen_t;

        if unsafe {
          libc::getsockname(
            fd,
            &mut addr as *mut _ as *mut libc::sockaddr,
            &mut addr_len,
          )
        } != 0
        {
          warn!("failed to get socket name for FD {fd}");
          continue;
        }

        let port = match addr.ss_family as i32 {
          libc::AF_INET => {
            let addr_in = unsafe { &*((&addr) as *const _ as *const libc::sockaddr_in) };
            u16::from_be(addr_in.sin_port)
          }
          libc::AF_INET6 => {
            let addr_in6 = unsafe { &*((&addr) as *const _ as *const libc::sockaddr_in6) };
            u16::from_be(addr_in6.sin6_port)
          }
          _ => continue,
        };

        info!("FD {fd} → port {port}");
        map.insert(port, fd);
      }
    }
  }

  Mutex::new(map)
});

/// 尝试从 systemd socket activation 获取 TcpListener
/// Try to get TcpListener from systemd socket activation
///
/// 每个端口的 FD 只能获取一次，重复调用同一端口会返回 None
/// Each port's FD can only be obtained once, repeated calls return None
pub fn listen_fd(port: u16) -> Result<Option<TcpListener>> {
  let fds = SYSTEMD_FDS.lock().unwrap();

  if let Some(from_fd) = fds.get(&port) {
    // 复制 fd，避免 Rust 关闭原始 fd 影响 reload
    // Duplicate fd to prevent Rust from closing the original fd during reload
    let fd = unsafe { libc::dup(*from_fd) };
    if fd < 0 {
      return Err(std::io::Error::last_os_error());
    }

    info!("systemd socket activation: port {port} → FD {fd} ( dup from {from_fd} )");

    // 设置非阻塞模式 (Tokio 需要)
    // Set non-blocking mode (required by Tokio)
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 {
      return Err(std::io::Error::last_os_error());
    }
    if unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
      return Err(std::io::Error::last_os_error());
    }

    return Ok(Some(unsafe { TcpListener::from_raw_fd(fd) }));
  }

  Ok(None)
}
