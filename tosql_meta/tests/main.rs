use aok::{OK, Result};
use kind2sql::Kind;
use log::info;
use tosql_meta::Meta;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

#[test]
fn test_basic_load() -> Result<()> {
  info!("> test_basic_load");

  // Use tosql::Meta to create binary data
  let tosql_meta = tosql::Meta {
    name: "test_struct",
    kind_li: &["id", "name"],
    field_li: &[Kind::U64, Kind::String],
  };
  let bin = tosql_meta.dump();

  let meta = Meta::load(&bin)?;

  assert_eq!(meta.kind_li.len(), 2);
  assert_eq!(meta.kind_li[0], "id");
  assert_eq!(meta.kind_li[1], "name");

  assert_eq!(meta.field_li.len(), 2);
  assert_eq!(meta.field_li[0], Kind::U64);
  assert_eq!(meta.field_li[1], Kind::String);

  OK
}
