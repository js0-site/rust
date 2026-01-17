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
//! let pgm = Pgm::new(&data, 32);
//! let range = pgm.predict_range(500);
//! assert!(range.contains(&500));
//!
//! // PgmData: with data ownership
//! // PgmData：持有数据
//! let pgm_data = PgmData::new(&data, 32);
//! assert_eq!(pgm_data.get(500), Some(500));
//! ```

#[cfg(feature = "compress")]
pub mod pc;
pub mod pgm;
pub mod util;

#[cfg(feature = "compress")]
pub use pc::Pc;
#[cfg(feature = "data")]
pub use pgm::data::PgmData;
pub use pgm::{
  Pgm,
  build::{build_lut, build_segments},
  consts::{LUT_BINS_MULTIPLIER, MAX_LUT_BINS, MIN_EPSILON, MIN_LUT_BINS},
  types::{Key, PgmStats, Segment, ToKey},
};

/// Alias for backward compatibility
/// 向后兼容的别名
#[cfg(feature = "data")]
pub type PgmIndex<K> = PgmData<K>;

#[cfg(feature = "key_to_u64")]
#[inline]
pub fn key_to_u64(key: &[u8]) -> u64 {
  pgm::types::bytes_to_u64(key)
}
