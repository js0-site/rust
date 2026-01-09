#![cfg_attr(docsrs, feature(doc_cfg))]

mod error;

use std::{env::var, fs::File, io::Write, process};

pub use error::Error;

/// Auto daemonize based on environment variables
/// 根据环境变量自动守护进程化
pub fn auto() {
  if let Ok(pid_file_path) = var("PID_FILE")
    && let Err(e) = daemonize(&pid_file_path)
  {
    log::error!("Fork error: {e}");
  }
}

/// Fork the current process and write PID to file
/// 分叉当前进程并将PID写入文件
fn daemonize(pid_file_path: &str) -> Result<(), Error> {
  let pid = unsafe { libc::fork() };

  match pid {
    -1 => {
      // Fork failed
      // 分叉失败
      Err(Error::ForkFailed)
    }
    0 => {
      // Child process continues execution
      // 子进程继续执行
      Ok(())
    }
    child_pid => {
      // Parent process writes child PID and exits
      // 父进程写入子进程PID并退出
      let mut file = File::create(pid_file_path)?;
      writeln!(file, "{child_pid}")?;
      process::exit(0);
    }
  }
}
