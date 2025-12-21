use aok::{OK, Void};
use to_mysql::Mysql;
use tosql::{ToSqlTrait, tosql};

#[tosql]
struct User {
  id: u64,
  name: String,
  hidden: bool,
}

#[test]
fn test_create_table() {
  let mysql = Mysql::new("User", User::META.field_li, User::META.kind_li);
  let actual = mysql.create_table();
  println!("{}", actual);
  assert_eq!(
    actual,
    "CREATE TABLE User(id BIGINT UNSIGNED,name LONGTEXT,hidden BOOLEAN);"
  );
}

#[test]
fn test_insert() -> Void {
  let user = User {
    id: 123,
    name: "Alice".to_string(),
    hidden: true,
  };
  let mysql = Mysql::new("User", User::META.field_li, User::META.kind_li);
  let actual = mysql.insert(&user.dump())?;
  println!("{}", actual);
  assert_eq!(
    actual,
    "INSERT INTO User(id,name,hidden)VALUES(123,'Alice',1);"
  );
  OK
}
