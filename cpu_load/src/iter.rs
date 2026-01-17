use std::sync::atomic::{AtomicU8, Ordering};

/// Iterator for CPU core loads
/// CPU 核心负载迭代器
pub struct CpuLoadIter<'a> {
  cores: &'a [AtomicU8],
  idx: usize,
}

impl<'a> CpuLoadIter<'a> {
  #[inline]
  pub(crate) fn new(cores: Option<&'a [AtomicU8]>) -> Self {
    Self {
      cores: cores.unwrap_or(&[]),
      idx: 0,
    }
  }
}

impl Iterator for CpuLoadIter<'_> {
  type Item = u8;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    if self.idx < self.cores.len() {
      // SAFETY: idx < len checked above
      let v = unsafe { self.cores.get_unchecked(self.idx) }.load(Ordering::Relaxed);
      self.idx += 1;
      Some(v)
    } else {
      None
    }
  }

  #[inline]
  fn size_hint(&self) -> (usize, Option<usize>) {
    let rem = self.cores.len() - self.idx;
    (rem, Some(rem))
  }
}

impl ExactSizeIterator for CpuLoadIter<'_> {
  #[inline]
  fn len(&self) -> usize {
    self.cores.len() - self.idx
  }
}
