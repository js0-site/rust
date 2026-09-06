#![cfg_attr(docsrs, feature(doc_cfg))]

pub use kind2sql::{Kind, SqlField};
use sqle::mysql;

pub const KIND: [&str; 11] = [
  "BOOLEAN",           // Bool
  "TINYINT UNSIGNED",  // U8
  "TINYINT",           // I8
  "SMALLINT UNSIGNED", // U16
  "SMALLINT",          // I16
  "INT UNSIGNED",      // U32
  "INT",               // I32
  "BIGINT UNSIGNED",   // U64
  "BIGINT",            // I64
  "LONGTEXT",          // String
  "LONGBLOB",          // Bytes
];

pub struct Mysql {
  pub kind_li: Vec<Kind>,
  pub field_li: Vec<String>,
  pub table_name: String,
  pub insert_prefix: String,
}

impl SqlField for Mysql {
  fn blob(data: &[u8]) -> String {
    mysql::blob(data)
  }
}

impl Mysql {
  pub fn new(table_name: impl Into<String>, kind_li: &[Kind], field_li: &[&str]) -> Self {
    let table_name = table_name.into();
    let insert_prefix = format!("INSERT INTO {table_name}({})VALUES(", field_li.join(","));
    Self {
      kind_li: kind_li.into(),
      field_li: field_li.iter().map(|s| s.to_string()).collect(),
      table_name,
      insert_prefix,
    }
  }

  pub fn create_table(&self) -> String {
    let cols = self
      .field_li
      .iter()
      .zip(self.kind_li.iter())
      .map(|(name, kind)| {
        let mysql_type = KIND[*kind as usize - 1];
        format!("{} {}", name, mysql_type)
      })
      .collect::<Vec<_>>()
      .join(",");
    format!("CREATE TABLE {}({});", self.table_name, cols)
  }

  pub fn insert(&self, bytes: &[u8]) -> vb::Result<String> {
    let sql_values = Mysql::sql_field(&self.kind_li, bytes)?;
    Ok(format!("{}{});", self.insert_prefix, sql_values.join(",")))
  }
}
