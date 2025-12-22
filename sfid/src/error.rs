use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
  #[error("Redis error: {0}")]
  Redis(#[from] fred::error::Error),

  #[error("No available machine ID, all {0} slots are occupied")]
  NoAvailableMachineId(u16),

  #[error("Clock moved backwards by {0}ms")]
  ClockMovedBackwards(u64),
}
