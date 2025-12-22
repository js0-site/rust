use std::collections::HashSet;

use aok::{OK, Void};
use sfid::Snowflake;

fn test_snowflake(sf: &Snowflake) {
  let mut ids = HashSet::new();
  for _ in 0..10000 {
    let id = sf.next();
    assert!(ids.insert(id), "duplicate id: {id}");
  }
  println!("generated {} unique ids", ids.len());
}

async fn test_concurrent(sf: &'static Snowflake) {
  let handles: Vec<_> = (0..4)
    .map(|_| {
      tokio::spawn(async move {
        let mut ids = Vec::with_capacity(1000);
        for _ in 0..1000 {
          ids.push(sf.next());
        }
        ids
      })
    })
    .collect();

  let mut all_ids = HashSet::new();
  for h in handles {
    let ids = h.await.unwrap();
    for id in ids {
      assert!(all_ids.insert(id), "duplicate id: {id}");
    }
  }
  println!("concurrent: {} unique ids", all_ids.len());
}

#[tokio::test(flavor = "multi_thread")]
async fn test() -> Void {
  xboot::init().await?;

  let sf = Box::leak(Box::new(Snowflake::auto("test", sfid::EPOCH).await?));

  test_snowflake(sf);
  test_concurrent(sf).await;

  OK
}
