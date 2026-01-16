#![cfg_attr(docsrs, feature(doc_cfg))]

use num_enum::{IntoPrimitive, TryFromPrimitive};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
pub enum Kind {
  Bool = 1,
  U8 = 2,
  I8 = 3,
  U16 = 4,
  I16 = 5,
  U32 = 6,
  I32 = 7,
  U64 = 8,
  I64 = 9,
  String = 10,
  Bytes = 11,
}

#[cfg(feature = "sql_field")]
mod sql_field;
#[cfg(feature = "sql_field")]
pub use sql_field::SqlField;
