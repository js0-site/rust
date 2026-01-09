#![cfg_attr(docsrs, feature(doc_cfg))]

use std::process;

use futures::StreamExt;
use listen_signal::{SIGHUP, wait_all};
use log::info;
use parking_lot::RwLock;
use tokio_util::sync::CancellationToken;

#[static_init::dynamic]
pub static LOCK: RwLock<()> = RwLock::new(());

#[static_init::dynamic]
pub static CANCEL: CancellationToken = CancellationToken::new();

pub async fn graceful_restart() {
  loop {
    if let Some(signal) = wait_all().next().await {
      CANCEL.cancel();
      if signal == SIGHUP {
        #[cfg(target_os = "linux")]
        {
          use std::os::unix::process::CommandExt;
          match self_cmd::get() {
            Ok(mut cmd) => {
              unsafe {
                cmd.pre_exec(|| {
                  // 使用 nix::unistd::setsid 剥离父子关系：
                  match nix::unistd::setsid() {
                    Ok(_) => Ok(()),
                    Err(e) => {
                      use std::{
                        io,
                        os::fd::{BorrowedFd, RawFd},
                      };
                      let fd = BorrowedFd::borrow_raw(2 as RawFd);
                      let _ = nix::unistd::write(fd, b"Error: setsid failed in pre_exec!\n");
                      return Err(io::Error::new(io::ErrorKind::Other, e));
                    }
                  }
                });
              }
              match cmd.spawn() {
                Ok(child) => {
                  let pid = child.id();
                  sys_notify::mainid(pid);
                  info!("[SIGHUP] sys_notify mainid={pid}");
                }
                Err(e) => {
                  log::error!("spawn error: {e}");
                }
              }
            }
            Err(e) => {
              log::error!("SIGHUP error: {e}");
            }
          }
        }
        #[cfg(not(target_os = "linux"))]
        {
          use std::env::consts::{ARCH, OS};
          log::warn!("SIGHUP is not supported on {OS}({ARCH})");
        }
      }
      let _guard = LOCK.write();
      info!("pid={} exit", process::id());
      log::logger().flush();
      process::exit(0);
    }
  }
}

xboot::add!(tokio::spawn(graceful_restart()));
