use std::{os::unix::process::CommandExt, process::Command};

use nix::unistd;
use tokio::signal::unix::{SignalKind, signal};

/// Wait for SIGHUP signal / 等待 SIGHUP 信号
pub async fn wait_reload() {
  let mut stream = signal(SignalKind::hangup()).expect("Failed to create SIGHUP signal stream");
  stream.recv().await;
}

/// Setup Unix process detachment / 设置 Unix 进程分离
pub fn setup_process_detachment(command: &mut Command) {
  unsafe {
    command.pre_exec(|| {
      // Create new session, completely detach from controlling terminal and parent process
      // 创建新的会话，完全脱离控制终端和父进程
      unistd::setsid().map_err(std::io::Error::other)?;
      Ok(())
    });
  }
}
