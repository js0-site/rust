use std::{sync::Arc, time::Duration};

use aok::{OK, Void};
use expire_set::ExpireSet;
use log::info;
use tokio::{runtime::Runtime, time::sleep};

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

#[tokio::test]
async fn test_basic_insert_and_contains() -> Void {
  info!(">test_basic_insert_and_contains");
  let set = ExpireSet::<String>::new(10);

  // 测试插入和查询
  set.insert("key1".to_string());
  assert!(set.contains(&"key1".to_string()));

  set.insert("key2".to_string());
  assert!(set.contains(&"key1".to_string()));
  assert!(set.contains(&"key2".to_string()));

  // 测试不存在的键
  assert!(!set.contains(&"key3".to_string()));

  info!("basic insert and contains works");
  OK
}

#[tokio::test]
async fn test_expiration() -> Void {
  info!(">test_expiration");
  // 创建一个2秒过期的集合
  let set = ExpireSet::<String>::new(2);

  // 插入一些键
  set.insert("expire1".to_string());
  set.insert("expire2".to_string());

  assert!(set.contains(&"expire1".to_string()));
  assert!(set.contains(&"expire2".to_string()));

  info!("waiting 1 second...");
  sleep(Duration::from_secs(1)).await;

  // 1秒后应该还存在
  assert!(set.contains(&"expire1".to_string()));

  info!("waiting another 2.5 seconds for rotation...");
  sleep(Duration::from_millis(2500)).await;

  // 第一次轮转后，旧的集合被清空，但数据仍在另一个集合中
  // 在下一次轮转后，expire1 和 expire2 才会完全消失
  // 所以现在它们应该仍然存在（在旧集合中）
  assert!(set.contains(&"expire1".to_string()));

  info!("waiting another 2 seconds for second rotation...");
  sleep(Duration::from_secs(2)).await;

  // 第二次轮转后，expire1 和 expire2 所在的集合被清空
  assert!(!set.contains(&"expire1".to_string()));
  assert!(!set.contains(&"expire2".to_string()));

  info!("expiration works correctly");
  OK
}

#[tokio::test]
async fn test_cache_rotation() -> Void {
  info!(">test_cache_rotation");
  // 创建一个1秒过期的集合
  let set = ExpireSet::<i32>::new(1);

  // 第一批数据
  for i in 0..10 {
    set.insert(i);
  }

  assert!(set.contains(&5));

  info!("waiting 1.2 seconds for rotation...");
  sleep(Duration::from_millis(1200)).await;

  // 切换后，插入第二批数据
  for i in 10..20 {
    set.insert(i);
  }

  // 第一批应该还在（虽然在旧的集合中）
  assert!(set.contains(&5));
  assert!(set.contains(&15));

  info!("waiting another 1.2 seconds...");
  sleep(Duration::from_millis(1200)).await;

  // 再次切换后，第一批应该被清除了
  assert!(!set.contains(&5));
  // 第二批应该还在
  assert!(set.contains(&15));

  info!("cache rotation works correctly");
  OK
}

#[tokio::test]
async fn test_concurrent_access() -> Void {
  info!(">test_concurrent_access");
  let set = Arc::new(ExpireSet::<String>::new(5));

  let mut handles = vec![];

  // 启动多个任务并发插入
  for i in 0..10 {
    let set_clone = set.clone();
    let handle = tokio::spawn(async move {
      for j in 0..100 {
        let key = format!("key_{}_{}", i, j);
        set_clone.insert(key.clone());
        assert!(set_clone.contains(&key));
      }
    });
    handles.push(handle);
  }

  // 等待所有任务完成
  for handle in handles {
    handle.await.unwrap();
  }

  // 验证一些键存在
  assert!(set.contains(&"key_0_0".to_string()));
  assert!(set.contains(&"key_5_50".to_string()));
  assert!(set.contains(&"key_9_99".to_string()));

  info!("concurrent access works correctly");
  OK
}

#[tokio::test]
async fn test_duplicate_inserts() -> Void {
  info!(">test_duplicate_inserts");
  let set = ExpireSet::<String>::new(10);

  // 重复插入同一个键
  set.insert("duplicate".to_string());
  set.insert("duplicate".to_string());
  set.insert("duplicate".to_string());

  // 应该只存在一次（DashSet 去重）
  assert!(set.contains(&"duplicate".to_string()));

  info!("duplicate inserts handled correctly");
  OK
}

#[test]
fn test_sync() -> Void {
  info!(">test_sync");
  // 测试 ExpireSet 是否可以在同步代码中使用
  let rt = Runtime::new().unwrap();
  rt.block_on(async {
    let set = ExpireSet::<i32>::new(10);
    set.insert(42);
    assert!(set.contains(&42));
  });

  info!("sync usage works");
  OK
}
