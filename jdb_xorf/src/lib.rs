//! # jdb_xorf
//!
//! Fast and compact Xor and Binary Fuse filters for Rust.
//!
//! 快速、紧凑的 Rust Xor 和 Binary Fuse 过滤器。
//!
//! Please refer to the [README](https://self) for detailed documentation.
//!
//! 详细文档请参阅 [README](https://self)。

#![allow(unexpected_cfgs)]
#![no_std]
#![cfg_attr(feature = "nightly", feature(allocator_internals), needs_allocator)]
#![warn(missing_docs)]
#![allow(clippy::multiple_crate_versions, clippy::fallible_impl_from)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[macro_use]
extern crate alloc;

#[macro_use]
mod prelude;
mod base;
mod hash;

pub use base::{Bf8, Bf8Ref, Bf16, Bf16Ref, Bf32, Bf32Ref, BfRef};
mod bf;


pub use bf::Bf;
pub use hash::{RapidHasher, mix64};

/// Methods common to xor filters.
///
/// Xor 过滤器的通用方法。
pub trait Filter<Type> {
  /// Returns `true` if the filter probably contains the specified key.
  ///
  /// 如果过滤器可能包含指定的键，则返回 `true`。
  ///
  /// There can never be a false negative, but there is a small possibility of false positives.
  /// Refer to individual filters' documentation for false positive rates.
  ///
  /// 绝不会出现假阴性（False Negative），但有极小的假阳性（False Positive）可能性。
  /// 关于假阳性率，请参阅各个过滤器的文档。
  fn contains(&self, key: &Type) -> bool;

  /// Returns the number of fingerprints in the filter.
  ///
  /// 返回过滤器中指纹的数量。
  fn len(&self) -> usize;

  /// Returns `true` if the filter has no fingerprints.
  ///
  /// 如果过滤器没有指纹，则返回 `true`。
  fn is_empty(&self) -> bool {
    self.len() == 0
  }
}

/// Equivalent to Filter except represents a reference to fingerprints stored elsewhere.
///
/// 类似于 Filter，但表示对存储在别处的指纹的引用。
pub trait FilterRef<'a, Type>: Filter<Type> {
  /// The alignment required of the fingerprints slice.
  ///
  /// 指纹切片所需的对齐方式。
  const FINGERPRINT_ALIGNMENT: usize;

  /// Create a filter from memory slices. These slices can be mmap from a file. The desc
  /// is eagerly destructured while the fingerprints reference is retained. If the fingerprints
  /// slice provided doesn't have an alignment of `FINGERPRINT_ALIGNMENT`, this function will
  /// panic.
  ///
  /// 从内存切片创建过滤器。这些切片可以是文件的 mmap。描述符会被立即解构，而指纹引用会被保留。
  /// 如果提供的指纹切片没有 `FINGERPRINT_ALIGNMENT` 的对齐方式，此函数将 panic。
  fn from_dma(desc: &[u8], fingerprints: &'a [u8]) -> Self;
}

/// DMA serializable filters are ones who can be essentially directly accessed into/out of DMA buffers.
///
/// DMA 可序列化过滤器是指本质上可以直接存取 DMA 缓冲区的过滤器。
///
/// This isn't a true 0-copy implementation and instead we make the following simplification.
/// A DMA serializable filter has two components - the "fixed" desc and the variable len fingerprints.
/// The fixed desc is small (a few words at most) and is copied into / out of the serialized form.
/// The variable len fingerprints however are referenced directly.
///
/// 这不是真正的零拷贝实现，我们做了如下简化。
/// DMA 可序列化过滤器有两个组件 - “固定”描述符和可变长度指纹。
/// 固定描述符很小（最多几个字），会被复制进/出序列化形式。
/// 而可变长度指纹则是直接引用的。
pub trait DmaSerializable {
  /// The serialized len of the desc. Very small and safe to allocate on-stack if needed.
  ///
  /// 描述符的序列化长度。非常小，如果需要，可以在栈上安全分配。
  const LEN: usize;

  /// Copies the small fixed-len desc part of the filter to an output buffer.
  ///
  /// 将过滤器的固定长度描述符部分复制到输出缓冲区。
  fn dma_copy_desc_to(&self, out: &mut [u8]);

  /// Obtains the raw byte slice of the fingerprints to serialize to disk.
  ///
  /// 获取指纹的原始字节切片以序列化到磁盘。
  fn dma_fingerprints(&self) -> &[u8];
}
