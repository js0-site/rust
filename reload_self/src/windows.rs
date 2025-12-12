use std::{process::Command, sync::OnceLock};

use tokio::sync::Notify;
use winapi::{
  shared::minwindef::{BOOL, DWORD, FALSE, TRUE},
  um::{consoleapi::SetConsoleCtrlHandler, wincon::CTRL_BREAK_EVENT},
};

static SIGNAL_NOTIFY: OnceLock<Notify> = OnceLock::new();

/// Console control handler for Windows / Windows 控制台控制处理程序
unsafe extern "system" fn console_handler(ctrl_type: DWORD) -> BOOL {
  match ctrl_type {
    CTRL_BREAK_EVENT => {
      if let Some(notify) = SIGNAL_NOTIFY.get() {
        notify.notify_one();
      }
      TRUE
    }
    _ => FALSE,
  }
}

/// Wait for CTRL_BREAK_EVENT signal / 等待 CTRL_BREAK_EVENT 信号
pub async fn wait_reload() {
  let notify = SIGNAL_NOTIFY.get_or_init(|| {
    unsafe {
      // Register console control handler / 注册控制台控制处理程序
      SetConsoleCtrlHandler(Some(console_handler), TRUE);
    }
    Notify::new()
  });

  notify.notified().await;
}

/// Setup Windows process detachment / 设置 Windows 进程分离
pub fn setup_process_detachment(_command: &mut Command) {
  // Windows doesn't need special process detachment setup
  // Windows 不需要特殊的进程分离设置
}
