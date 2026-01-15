#![cfg_attr(docsrs, feature(doc_cfg))]

pub use bytes::{BufMut, Bytes, BytesMut};
pub use kind2sql::{Kind, SqlField};
pub use linkme;
pub use tosql_derive::ToSql;
pub use tosql_linkme;
pub use tosql_macro::tosql;
pub use tosql_static_meta::Meta;
pub use vb;

// pub mod mysql {
//   pub use kind2sql::mysql::{KIND, Mysql};
// }

pub trait ToSqlTrait {
  const META: Meta;
  fn dump(&self) -> Bytes;
}

tosql_linkme::turn! {
  mod sql_struct_li;
  pub use sql_struct_li::{SQL_STRUCT_LI, SqlStruct};
}
