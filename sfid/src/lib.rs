#![cfg_attr(docsrs, feature(doc_cfg))]

mod bits;
pub use bits::{DefaultLayout, Layout};

mod error;
pub use error::{Error, Result};

/// Clock backward threshold in seconds
/// 时钟回拨告警阈值（秒）
const CLOCK_BACKWARD_WARN_SEC: u64 = 1;

/// Default epoch: 2025-12-22 00:00:00 UTC (seconds)
/// 默认纪元：2025-12-22 00:00:00 UTC（秒）
pub const EPOCH: u64 = 1766361600;

#[cfg(feature = "auto_pid")]
mod pid;
#[cfg(feature = "auto_pid")]
#[cfg_attr(docsrs, doc(cfg(feature = "auto_pid")))]
pub use pid::{Pid, allocate};

#[cfg(feature = "snowflake")]
mod sfid;
#[cfg(feature = "snowflake")]
#[cfg_attr(docsrs, doc(cfg(feature = "snowflake")))]
pub use sfid::SfId;

#[cfg(feature = "parse")]
mod parse;
#[cfg(feature = "parse")]
#[cfg_attr(docsrs, doc(cfg(feature = "parse")))]
pub use parse::{ParsedId, parse, parse_with};

/// Create SfId with Redis-allocated process ID
/// 使用 Redis 分配的进程号创建 SfId
#[cfg(feature = "auto_pid")]
#[cfg_attr(docsrs, doc(cfg(feature = "auto_pid")))]
pub async fn new(app: impl AsRef<[u8]>) -> Result<SfId> {
  let pid_handle = allocate::<DefaultLayout>(app).await?;
  Ok(SfId::with_pid(EPOCH, pid_handle))
}
