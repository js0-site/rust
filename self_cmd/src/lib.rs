#![cfg_attr(docsrs, feature(doc_cfg))]

use std::{
  env,
  process::{Command, Stdio},
};

mod error;
pub use error::{Error, Result};

#[cfg(target_os = "linux")]
mod fd_mapping;

#[cfg(target_os = "linux")]
pub use fd_mapping::fd_mapping;

pub fn get() -> Result<Command> {
  let program = env::current_exe()?;
  let args: Vec<String> = env::args().skip(1).collect();
  let mut cmd = Command::new(program);
  cmd.args(args);

  // 继承日志输出
  cmd.stdin(Stdio::inherit());
  cmd.stdout(Stdio::inherit());
  cmd.stderr(Stdio::inherit());

  #[cfg(target_os = "linux")]
  {
    use command_fds::CommandFdExt;
    let fd_mappings = crate::fd_mapping()?;

    if !fd_mappings.is_empty() {
      cmd.env("LISTEN_FDS", fd_mappings.len().to_string());

      if let Err(err) = cmd.fd_mappings(fd_mappings) {
        return Err(err.into());
      }
      cmd.env_remove("LISTEN_PID");
    }
  }

  Ok(cmd)
}
