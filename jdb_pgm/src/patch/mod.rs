use std::fmt::Debug;

use crate::{Key, Segment};

#[cfg(feature = "bitpacking")]
pub mod bitpacked;
mod no;

#[cfg(feature = "bitpacking")]
pub use bitpacked::BitPackedPatch;
pub use no::NoPatch;

/// Trait for PGM patches (hooks for compression/augmentation)
/// PGM 补丁 trait（用于压缩或增强的钩子）
#[cfg(feature = "bitcode")]
pub trait Patch<K: Key>:
  Clone + Debug + bitcode::Encode + for<'a> bitcode::Decode<'a> + 'static
{
  /// Hook: Called after segments are built, but before LUT creation.
  /// 钩子：在段构建完成后，LUT 创建前调用。
  /// This is where you calculate residuals and compress data.
  fn on_segments_built(sorted_data: &[K], segments: &[Segment<K>]) -> Self;
}

#[cfg(not(feature = "bitcode"))]
pub trait Patch<K: Key>: Clone + Debug + 'static {
  fn on_segments_built(sorted_data: &[K], segments: &[Segment<K>]) -> Self;
}
