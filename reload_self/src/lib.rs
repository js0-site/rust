#![cfg_attr(docsrs, feature(doc_cfg))]

use std::{env, process::Command};

use log::{error, info};
use tokio::task;
pub use tokio_util::sync::CancellationToken;

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

// Import platform-specific functions / 导入平台特定函数
#[cfg(unix)]
use unix::{setup_process_detachment, wait_reload};
#[cfg(windows)]
use windows::{setup_process_detachment, wait_reload};

/// Spawn new process with same executable and arguments / 使用相同的可执行文件和参数生成新进程
async fn spawn(token: CancellationToken) {
  let current_exe = match env::current_exe() {
    Ok(path) => path,
    Err(e) => {
      error!("NO EXE PATH {e}");
      return;
    }
  };

  // Get arguments passed to current process / 获取传递给当前进程的参数
  let args: Vec<String> = env::args().collect();

  info!("reload_self : {} {:?}", current_exe.display(), &args[1..]);

  let mut command = Command::new(current_exe);
  command.args(&args[1..]);
  // .stdin(Stdio::null())
  // .stdout(Stdio::inherit())
  // .stderr(Stdio::inherit());

  // Apply platform-specific process setup / 应用平台特定的进程设置
  setup_process_detachment(&mut command);

  match command.spawn() {
    Ok(child) => {
      let pid = child.id();
      info!("reload_self : PID {pid} ; 母进程开始关闭");
      token.cancel();
    }
    Err(e) => {
      error!("reload_self : {e}");
    }
  }
}

/// Listen for platform-specific reload signal and return a CancellationToken.
/// On Unix systems: listens for SIGHUP signal
/// On Windows systems: listens for CTRL_BREAK_EVENT signal
///
/// 监听平台特定的重载信号并返回一个 CancellationToken。
/// Unix 系统：监听 SIGHUP 信号
/// Windows 系统：监听 CTRL_BREAK_EVENT 信号
pub fn listen() -> Result<CancellationToken, std::io::Error> {
  let token = CancellationToken::new();
  let token_for_signal = token.clone();

  task::spawn(async move {
    // Wait for platform-specific signal / 等待平台特定信号
    wait_reload().await;
    spawn(token_for_signal).await;
  });

  Ok(token)
}
