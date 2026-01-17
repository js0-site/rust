use aok::{OK, Void};

#[test]
fn test() -> Void {
  let id1 = osid::get()?;
  println!("id: {id1}");
  assert!(!id1.is_empty());
  assert!(id1.contains(':'));

  // Should return same ID on second call
  // 第二次调用应返回相同ID
  let id2 = osid::get()?;
  assert_eq!(id1, id2);

  OK
}
