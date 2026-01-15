//! Error definitions for Pgm-Index
//! Pgm 索引错误定义

use thiserror::Error;

pub type Result<T> = std::result::Result<T, PgmError>;

#[derive(Debug, Clone, Error)]
#[non_exhaustive]
pub enum PgmError {
  #[error("Data cannot be empty / 数据不能为空")]
  EmptyData,

  #[error("Data must be sorted / 数据必须已排序")]
  NotSorted,

  #[error(
    "Epsilon must be >= {min} (provided: {provided}) / Epsilon 必须 >= {min} (提供的值: {provided})"
  )]
  InvalidEpsilon { provided: usize, min: usize },
}
