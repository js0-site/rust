use std::time::Duration;

use aok::{OK, Void};
use fred::interfaces::HashesInterface;
use kvid::{KVID_KEY, KvId};
use tokio::time::sleep;
use xkv::R;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

static KVID_TEST: KvId = KvId::const_new("test");

#[tokio::test]
async fn test() -> Void {
  xboot::init().await?;
  let t1 = tokio::spawn(async {
    for _ in 0..50 {
      let id = KVID_TEST.next().await?;
      println!("t1: {id}");
      sleep(Duration::from_millis(10)).await;
    }
    Ok::<_, kvid::Error>(())
  });
  let t2 = tokio::spawn(async {
    for _ in 0..50 {
      let id = KVID_TEST.next().await?;
      println!("t2: {id}");
      sleep(Duration::from_millis(10)).await;
    }
    Ok::<_, kvid::Error>(())
  });
  t1.await??;
  t2.await??;
  // 清理测试key / cleanup test key
  R.hdel::<(), _, _>(KVID_KEY, "test").await?;
  OK
}
