//! LRU cache implementations
//! LRU 缓存实现
//!
//! # Features
//! 特性
//!
//! - `lru`: Standard LRU cache (default)
//! - `no`: NoCache - zero overhead no-op
//!
//! # Complexity Summary
//! 复杂度总结
//!
//! | Cache   | get  | set  | rm   | Space |
//! |---------|------|------|------|-------|
//! | Lru     | O(1) | O(1) | O(1) | O(n)  |
//! | NoCache | O(1) | O(1) | O(1) | O(1)  |

mod cache;

#[cfg(feature = "lru")]
mod lru;

#[cfg(feature = "no")]
mod no;

pub use cache::Cache;
#[cfg(feature = "lru")]
pub use lru::Lru;
#[cfg(feature = "no")]
pub use no::NoCache;
