use tosql_static_meta::Meta;

#[derive(Debug)]
pub struct SqlStruct {
  pub path: &'static str,
  pub meta: Meta,
}

#[linkme::distributed_slice]
pub static SQL_STRUCT_LI: [SqlStruct];
