use std::{
  env,
  os::fd::{FromRawFd, OwnedFd},
};

use command_fds::FdMapping;
use log::{error, info, warn};

pub fn fd_mapping() -> std::io::Result<Vec<command_fds::FdMapping>> {
  // Get LISTEN_FDS from environment / 从环境变量获取 LISTEN_FDS
  let listen_fds = env::var("LISTEN_FDS")
    .ok()
    .and_then(|s| s.parse::<i32>().ok())
    .unwrap_or(0);

  if listen_fds <= 0 {
    return Ok(Vec::new());
  }

  info!("LISTEN_FDS={}", listen_fds);

  let mut li = Vec::new();

  // Process each FD starting from 3 / 从文件描述符 3 开始处理每个 FD
  for i in 0..listen_fds {
    let raw_fd = 3 + i;

    if unsafe { libc::fcntl(raw_fd, libc::F_GETFD) } >= 0 {
      match unsafe { libc::dup(raw_fd) } {
        val if val >= 0 => {
          let owned_fd = unsafe { OwnedFd::from_raw_fd(val) };
          info!("FD {raw_fd} duplicated to FD {val}");

          li.push(FdMapping {
            parent_fd: owned_fd,
            child_fd: raw_fd,
          });
        }
        _ => {
          let err = std::io::Error::last_os_error();
          error!("❌ failed to duplicate FD {raw_fd}: {err}");
          return Err(err);
        }
      }
    } else {
      warn!("⚠️ FD {raw_fd} is NOT open");
    }
  }

  Ok(li)
}
