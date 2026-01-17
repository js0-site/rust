use super::Patch;
use crate::{Key, Segment};

/// Default No-Op Patch (Zero Sized Type)
/// 默认无操作补丁（零大小类型）
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[derive(Clone, Copy, Debug, Default)]
pub struct NoPatch;

impl<K: Key> Patch<K> for NoPatch {
  #[inline(always)]
  fn on_segments_built(_sorted_data: &[K], _segments: &[Segment<K>]) -> Self {
    Self // 没有任何开销
  }
}
