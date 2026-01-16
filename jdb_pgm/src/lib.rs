//! # Pgm-Index: Ultra-Fast Learned Index
//! Pgm 索引：超快的学习型索引
//!
//! Piecewise Geometric Model (Pgm) Index for fast lookups in sorted arrays.
//! 分段几何模型（Pgm）索引，用于在已排序数组中快速查找。
//!
//! ## Usage / 使用方法
//!
//! ```rust
//! use jdb_pgm::{Pgm, PgmData};
//!
//! // Pgm: no data ownership, for SSTable etc.
//! // Pgm：不持有数据，适用于 SSTable 等场景
//! let data: Vec<u64> = (0..1000).collect();
//! let pgm = Pgm::new(&data, 32, true).unwrap();
//! let (start, end) = pgm.predict_range(500);
//! assert!(start <= 500 && 500 < end);
//!
//! // PgmData: with data ownership
//! // PgmData：持有数据
//! let pgm_data = PgmData::load(data, 32, true).unwrap();
//! assert_eq!(pgm_data.get(500), Some(500));
//! ```

mod build;
mod consts;
#[cfg(feature = "data")]
mod data;
pub mod error;
mod pgm;
mod types;

#[doc(hidden)]
pub mod bench_common;

pub use build::{build_lut, build_segments};
pub use consts::{LUT_BINS_MULTIPLIER, MAX_LUT_BINS, MIN_EPSILON, MIN_LUT_BINS};
#[cfg(feature = "data")]
pub use data::PgmData;
pub use error::{PgmError, Result};
pub use pgm::Pgm;
pub use types::{Key, PgmStats, Segment, ToKey};

/// Alias for backward compatibility
/// 向后兼容的别名
#[cfg(feature = "data")]
pub type PgmIndex<K> = PgmData<K>;

/// Convert key bytes to u64 prefix (big-endian, pad with 0).
/// 将键字节转换为 u64 前缀（大端序，不足补0）。
#[cfg(feature = "key_to_u64")]
#[inline]
pub fn key_to_u64(key: &[u8]) -> u64 {
  let len = key.len().min(8);
  let mut buf = [0u8; 8];
  buf[..len].copy_from_slice(&key[..len]);
  u64::from_be_bytes(buf)
}
