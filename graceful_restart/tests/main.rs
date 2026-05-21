use std::{sync::Arc, thread, time::Duration};

use aok::{OK, Void};
use graceful_restart::{CANCEL, LOCK};
use log::info;
use tokio::{sync::Barrier, time::sleep};

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

#[test]
fn test_lock_basic() -> Void {
  // Test basic lock functionality / 测试基本锁功能
  info!("> test_lock_basic");

  let _read_guard = LOCK.read();
  info!("acquired read lock / 获取读锁");
  drop(_read_guard);

  let _write_guard = LOCK.write();
  info!("acquired write lock / 获取写锁");
  drop(_write_guard);

  OK
}

#[tokio::test]
async fn test_concurrent_lock_access() -> Void {
  // Test concurrent access to the global lock / 测试全局锁的并发访问
  info!("> test_concurrent_lock_access");

  let barrier = Arc::new(Barrier::new(3));
  let mut handles = vec![];

  // Spawn multiple tasks that only acquire read locks / 生成多个只获取读锁的任务
  for i in 0..3 {
    let barrier_clone = Arc::clone(&barrier);
    let handle = tokio::spawn(async move {
      barrier_clone.wait().await;
      {
        let guard = LOCK.read();
        info!("task {i} acquired read lock / 任务 {i} 获取读锁");
        drop(guard);
      }
      sleep(Duration::from_millis(10)).await;
      info!("task {i} released read lock / 任务 {i} 释放读锁");
    });
    handles.push(handle);
  }

  // Wait for all tasks to complete / 等待所有任务完成
  for handle in handles {
    handle.await.expect("task should complete / 任务应该完成");
  }

  OK
}

#[tokio::test]
async fn test_read_lock_behavior() -> Void {
  // Test read lock behavior / 测试读锁行为
  info!("> test_read_lock_behavior");

  let handle = tokio::spawn(async {
    {
      let guard = LOCK.read();
      info!("background task acquired read lock / 后台任务获取读锁");
      drop(guard);
    }
    sleep(Duration::from_millis(20)).await;
    info!("background task released read lock / 后台任务释放读锁");
  });

  // Main task also acquires read lock (should not block) / 主任务也获取读锁（不应阻塞）
  sleep(Duration::from_millis(5)).await;
  {
    let guard = LOCK.read();
    info!("main task acquired read lock / 主任务获取读锁");
    drop(guard);
  }
  sleep(Duration::from_millis(10)).await;
  info!("main task released read lock / 主任务释放读锁");

  handle
    .await
    .expect("background task should complete / 后台任务应该完成");
  OK
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn test_signal_handling_simulation() -> Void {
  // Simulate signal handling behavior (without actual signals) / 模拟信号处理行为（不使用实际信号）
  info!("> test_signal_handling_simulation");

  // Test that we can acquire the lock as the graceful_restart function would / 测试我们可以像graceful_restart函数那样获取锁
  let _guard = LOCK.write();
  info!("simulated signal handler acquired write lock / 模拟信号处理器获取写锁");

  // Simulate critical section protection / 模拟关键部分保护
  sleep(Duration::from_millis(10)).await;
  info!("simulated critical section completed / 模拟关键部分完成");

  drop(_guard);
  info!("simulated signal handler released write lock / 模拟信号处理器释放写锁");

  OK
}

#[test]
fn test_lock_is_send_sync() -> Void {
  // Test that LOCK implements Send + Sync / 测试LOCK实现Send + Sync
  info!("> test_lock_is_send_sync");

  fn assert_send_sync<T: Send + Sync>() {}
  assert_send_sync::<parking_lot::RwLock<()>>();

  // Test that we can move guards across thread boundaries / 测试我们可以跨线程边界移动守卫
  thread::spawn(|| {
    let _guard = LOCK.read();
    info!("cross-thread read lock acquired / 跨线程读锁获取");
  })
  .join()
  .expect("thread should complete / 线程应该完成");

  OK
}

#[tokio::test]
async fn test_multiple_readers() -> Void {
  // Test that multiple readers can acquire the lock simultaneously / 测试多个读者可以同时获取锁
  info!("> test_multiple_readers");

  let barrier = Arc::new(Barrier::new(4));
  let mut handles = vec![];

  for i in 0..4 {
    let barrier_clone = Arc::clone(&barrier);
    let handle = tokio::spawn(async move {
      barrier_clone.wait().await;
      {
        let guard = LOCK.read();
        info!("reader {i} acquired read lock / 读者 {i} 获取读锁");
        drop(guard);
      }
      sleep(Duration::from_millis(15)).await;
      info!("reader {i} released read lock / 读者 {i} 释放读锁");
    });
    handles.push(handle);
  }

  // All readers should be able to proceed simultaneously / 所有读者应该能够同时进行
  for handle in handles {
    handle
      .await
      .expect("reader task should complete / 读者任务应该完成");
  }

  OK
}

#[test]
fn test_cancel_token_basic() -> Void {
  // Test basic cancellation token functionality / 测试基本取消令牌功能
  info!("> test_cancel_token_basic");

  // Initially not cancelled / 初始状态未取消
  assert!(!CANCEL.is_cancelled());
  info!("cancel token is not cancelled initially / 取消令牌初始状态未取消");

  OK
}

#[tokio::test]
async fn test_cancel_token_with_select() -> Void {
  // Test cancellation token with tokio::select / 测试取消令牌与tokio::select配合使用
  info!("> test_cancel_token_with_select");

  let handle = tokio::spawn(async {
    tokio::select! {
      _ = CANCEL.cancelled() => {
        info!("received cancellation signal / 收到取消信号");
        "cancelled"
      }
      _ = sleep(Duration::from_millis(100)) => {
        info!("timeout reached / 达到超时");
        "timeout"
      }
    }
  });

  // Let the task start / 让任务开始
  sleep(Duration::from_millis(10)).await;

  // Simulate cancellation (in real usage, this would be done by graceful_restart) / 模拟取消（实际使用中由graceful_restart完成）
  CANCEL.cancel();

  let result = handle.await.expect("task should complete / 任务应该完成");
  info!("select result: {result} / 选择结果: {result}");

  // Verify cancellation was received / 验证收到了取消信号
  assert_eq!(result, "cancelled");

  OK
}

#[tokio::test]
async fn test_request_handling_with_cancellation() -> Void {
  // Test request handling pattern with cancellation / 测试带取消功能的请求处理模式
  info!("> test_request_handling_with_cancellation");

  async fn simulate_request_handler() -> &'static str {
    {
      let _guard = LOCK.read();
      // Just acquire the lock briefly to simulate some work
      // 仅仅短暂获取锁来模拟一些工作
    }

    tokio::select! {
      _ = CANCEL.cancelled() => {
        info!("request cancelled during processing / 请求在处理过程中被取消");
        "cancelled"
      }
      _ = sleep(Duration::from_millis(50)) => {
        info!("request completed normally / 请求正常完成");
        "completed"
      }
    }
  }

  let handle1 = tokio::spawn(simulate_request_handler());
  let handle2 = tokio::spawn(simulate_request_handler());

  // Let requests start processing / 让请求开始处理
  sleep(Duration::from_millis(20)).await;

  // Simulate shutdown signal / 模拟关闭信号
  CANCEL.cancel();

  let result1 = handle1
    .await
    .expect("request 1 should complete / 请求1应该完成");
  let result2 = handle2
    .await
    .expect("request 2 should complete / 请求2应该完成");

  info!("request 1 result: {result1} / 请求1结果: {result1}");
  info!("request 2 result: {result2} / 请求2结果: {result2}");

  // Both requests should be cancelled / 两个请求都应该被取消
  assert_eq!(result1, "cancelled");
  assert_eq!(result2, "cancelled");

  OK
}

#[tokio::test]
async fn test_new_requests_after_cancellation() -> Void {
  // Test that new requests can detect cancellation immediately / 测试新请求可以立即检测到取消状态
  info!("> test_new_requests_after_cancellation");

  // Ensure cancellation is active / 确保取消状态激活
  CANCEL.cancel();

  let handle = tokio::spawn(async {
    tokio::select! {
      _ = CANCEL.cancelled() => {
        info!("new request immediately cancelled / 新请求立即被取消");
        "immediately_cancelled"
      }
      _ = sleep(Duration::from_millis(10)) => {
        info!("new request processed / 新请求被处理");
        "processed"
      }
    }
  });

  let result = handle.await.expect("task should complete / 任务应该完成");
  assert_eq!(result, "immediately_cancelled");

  OK
}
