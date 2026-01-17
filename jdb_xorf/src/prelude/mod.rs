//! xor filter 的通称方法
//! Common methods for xor filters.

pub mod bfuse;

/// 创建一个存储输出指纹的块
/// Creates a block to store output fingerprints.
///
/// 这与 `make_block` 不同，因为我们可能想要随机化未使用的指纹，而不是将它们全部设为 0
/// This is distinguished from `make_block`, as we may want to randomize the unused fingerprints
/// rather than making them all 0.
///
/// ## 为什么？
/// ## Why?
///
/// 也不可避免地会有一些指纹条目未被使用。如果所有这些未使用的条目都是 0，
/// 则指纹(x) = 0 的元素 x 的误报率会显著高于未使用条目在均匀随机的情况下
/// Inevitably some fingerprint entries will not be used. If all of these unused entries are 0,
/// then the false-positive rate for a element x where fingerprint(x) = 0 is significantly higher
/// than if the unused entries are uniformly random
///
/// 权衡是生成随机元素比 memset 一堆零更昂贵，因此该选项可通过 `uniform-random` 特性配置
/// Of course, the tradeoff here is that generating random elements is more expensive than
/// memsetting a bunch of zeroes, so the option is configurable with the `uniform-random` feature.
#[doc(hidden)]
#[macro_export]
macro_rules! make_fp_block(
    ($size:expr, $ty:ty) => {
        {
            #[cfg(feature = "uniform-random")] {
                let mut seed = 0x12345678u64;
                core::iter::repeat_with(|| {
                    seed = seed.wrapping_add(1);
                    <$ty as $crate::hash::Fingerprint>::from_hash($crate::hash::mix64(seed))
                })
                    .take($size as usize)
                    .collect::<Box<[_]>>()
            }

            #[cfg(not(feature = "uniform-random"))] {
                vec![<$ty>::default(); $size as usize].into_boxed_slice()
            }
        }
    }
);

/// 检查键集合中的所有值是否完全不同
/// Checks if a collection of keys has all distinct values.
#[cfg(debug_assertions)]
pub fn all_distinct(keys: impl IntoIterator<Item = u64>) -> bool {
  let mut s = alloc::collections::BTreeSet::new();
  keys.into_iter().all(move |x| s.insert(x))
}
