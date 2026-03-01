use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  #[error("IO error: {0} / IO 错误: {0}")]
  Io(#[from] std::io::Error),

  #[cfg(target_os = "linux")]
  #[error("FD mapping collision: {0} / FD 映射冲突: {0}")]
  FdMappingCollision(#[from] command_fds::FdMappingCollision),
}

pub type Result<T> = std::result::Result<T, Error>;
