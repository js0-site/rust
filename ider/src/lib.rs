//! High-performance ts-based ID generator / 高性能时间戳ID生成器
//!
//! # ID Format / ID格式
//!
//! ```text
//! | 44 bits ts | 20 bits n |
//! |-------------------|------------------|
//! | seconds since epoch | micros within second |
//! ```
//!
//! # Features / 特性
//!
//! - Monotonic increasing IDs / 单调递增ID
//! - ~1M IDs per second / 每秒约100万个ID
//! - Clock backward tolerance / 时钟回拨容错
//! - Restart collision avoidance / 重启冲突避免

#[cfg(feature = "id")]
mod id;
#[cfg(feature = "path")]
pub mod path;

#[cfg(feature = "id")]
pub use id::{id, id_init};

mod ider;
pub use ider::{Ider, id_to_ts, id_to_ts_with_offset};
pub type ID = u64;
