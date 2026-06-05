use std::{io, result};

use fred::error::Error as FredError;
use thiserror::Error;

pub type Result<T, E = Error> = result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
  #[cfg(feature = "auto_pid")]
  #[error("Redis: {0}")]
  Redis(#[from] FredError),

  #[cfg(feature = "auto_pid")]
  #[error("No available PID, all {0} slots occupied")]
  NoAvailablePid(u32),

  #[cfg(feature = "auto_pid")]
  #[error("Machine ID: {0}")]
  OsId(#[from] &'static osid::Error),

  #[cfg(feature = "auto_pid")]
  #[error("Lock file: {0}")]
  LockFile(io::Error),
}
