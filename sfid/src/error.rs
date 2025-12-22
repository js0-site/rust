use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
  #[cfg(feature = "auto_pid")]
  #[error("Redis error: {0}")]
  Redis(#[from] fred::error::Error),

  #[cfg(feature = "auto_pid")]
  #[error("No available process ID, all {0} slots are occupied")]
  NoAvailablePid(u32),
}
