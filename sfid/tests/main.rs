use rapidhash::RapidHashSet as HashSet;

use aok::{OK, Void};
use sfid::{EPOCH, SfId, parse};

static SF: SfId = SfId::new(EPOCH, 1);

fn test_snowflake(sf: &SfId) {
  let mut ids = HashSet::new();
  for _ in 0..10000 {
    let id = sf.get();
    assert!(ids.insert(id), "duplicate id: {id}");
  }
  println!("generated {} unique ids", ids.len());
}

fn test_parse(sf: &SfId) {
  let id = sf.get();
  let parsed = parse(id);
  println!(
    "id={id} ts={} pid={} seq={}",
    parsed.ts, parsed.pid, parsed.seq
  );
  assert!(parsed.ts > 0);
}

async fn test_concurrent(sf: &'static SfId) {
  let handles: Vec<_> = (0..4)
    .map(|_| {
      tokio::spawn(async move {
        let mut ids = Vec::with_capacity(1000);
        for _ in 0..1000 {
          ids.push(sf.get());
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

async fn test_new() -> Void {
  let sf = sfid::new("test").await?;
  let id = sf.get();
  let parsed = parse(id);
  println!(
    "new: id={id} ts={} pid={} seq={}",
    parsed.ts, parsed.pid, parsed.seq
  );
  assert!(parsed.ts > 0);

  OK
}

#[tokio::test(flavor = "multi_thread")]
async fn test() -> Void {
  log_init::init();

  test_snowflake(&SF);
  test_parse(&SF);
  test_concurrent(&SF).await;

  xboot::init().await?;
  test_new().await?;
  OK
}
