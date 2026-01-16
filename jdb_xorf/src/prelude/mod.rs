//! xor filter 的通称方法
//! Common methods for xor filters.

#[cfg(feature = "binary-fuse")]
pub mod bfuse;

/// Applies a finalization mix to a randomly-seeded key, resulting in an avalanched hash. This
/// helps avoid high false-positive ratios (see Section 4 in the paper).
pub use crate::rand::mix;

/// 计算指纹
/// Computes a fingerprint.
#[doc(hidden)]
#[macro_export]
macro_rules! fingerprint(
    ($hash:expr) => {
        $hash ^ ($hash >> 32)
    };
);

/// 左旋
/// Rotate left
#[doc(hidden)]
#[macro_export]
macro_rules! rotl64(
    ($n:expr, by $c:expr) => {
        ($n << ($c & 63)) | ($n >> ((-$c) & 63))
    };
);

/// [取模归约的快速替代方案](http://lemire.me/blog/2016/06/27/a-fast-alternative-to-the-modulo-reduction/)
/// [A fast alternative to the modulo reduction](http://lemire.me/blog/2016/06/27/a-fast-alternative-to-the-modulo-reduction/)
#[doc(hidden)]
#[macro_export]
macro_rules! reduce(
    ($hash:ident on interval $n:expr) => {
        (($hash as u64 * $n as u64) >> 32) as usize
    };
);

/// 创建一个集合块，每个集合类型为 T
/// Creates a block of sets, each set being of type T.
#[doc(hidden)]
#[macro_export]
macro_rules! make_block(
    (with $size:ident sets) => {
        {
            vec![Default::default(); $size].into_boxed_slice()
        }
    };
);

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
    ($size:ident, $ty:ident) => {
        {
            #[cfg(feature = "uniform-random")] {
                core::iter::repeat_with(|| fastrand::$ty(..))
                    .take($size)
                    .collect::<Box<[_]>>()
            }

            #[cfg(not(feature = "uniform-random"))] {
                make_block!(with $size sets)
            }
        }
    }
);

/// Creates a block of sets, each set being of type T.
#[doc(hidden)]
#[macro_export]
macro_rules! try_enqueue(
    (block $H_block:expr, set $idx:ident; queue block $Q_block:expr, with size $qblock_size:expr) => {
        if $H_block[$idx].count == 1 {
            $Q_block[$qblock_size].index = $idx;
            // If there is only one key, the mask contains it wholly.
            $Q_block[$qblock_size].hash = $H_block[$idx].mask;
            $qblock_size += 1;
        }
    };
);

/// 检查键集合中的所有值是否完全不同
/// Checks if a collection of keys has all distinct values.
#[cfg(debug_assertions)]
pub fn all_distinct(keys: impl IntoIterator<Item = u64>) -> bool {
  let mut s = alloc::collections::BTreeSet::new();
  keys.into_iter().all(move |x| s.insert(x))
}
