use std::collections::HashSet;

use aok::{OK, Void};
use sfid::{MACHINE_ID, Snowflake};

static SF: Snowflake = Snowflake::new();

fn test_machine_id() {
  let id = **MACHINE_ID;
  println!("machine_id: {id}");
  assert!(id < 1024);
}

fn test_snowflake() {
  let mut ids = HashSet::new();
  for _ in 0..10000 {
    let id = SF.next().unwrap();
    assert!(ids.insert(id), "duplicate id: {id}");
  }
  println!("generated {} unique ids", ids.len());
}

async fn test_concurrent() {
  let handles: Vec<_> = (0..4)
    .map(|_| {
      tokio::spawn(async {
        let mut ids = Vec::with_capacity(1000);
        for _ in 0..1000 {
          ids.push(SF.next()?);
        }
        Ok::<_, sfid::Error>(ids)
      })
    })
    .collect();

  let mut all_ids = HashSet::new();
  for h in handles {
    let ids = h.await.unwrap().unwrap();
    for id in ids {
      assert!(all_ids.insert(id), "duplicate id: {id}");
    }
  }
  println!("concurrent: {} unique ids", all_ids.len());
}

#[tokio::test]
async fn test() -> Void {
  // Init MACHINE_ID directly
  // 直接初始化 MACHINE_ID
  MACHINE_ID.init().await?;

  test_machine_id();
  test_snowflake();
  test_concurrent().await;

  OK
}
