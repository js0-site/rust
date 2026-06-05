use std::time::{Duration, Instant};

use futures::StreamExt;
use log::info;
use race::Race;
use tokio::time::{sleep, timeout};

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

#[tokio::test]
async fn test_basic_race() {
  info!("=== Basic Race Test ===");
  info!("Testing simple tasks without delays");

  let mut race = Race::new(
    Duration::from_millis(10),
    |n: &i32| {
      let n = *n;
      async move {
        info!("Task {n} executing: {n} * 2 = {}", n * 2);
        Ok::<(i32, i32), &'static str>((n, n * 2))
      }
    },
    vec![1, 2, 3],
  );

  info!("Created Race with step=10ms, args=[1, 2, 3]");

  let result1 = race.next().await;
  info!("Result 1: {result1:?}");
  assert_eq!(result1, Some((1, Ok((1, 2)))));

  let result2 = race.next().await;
  info!("Result 2: {result2:?}");
  assert_eq!(result2, Some((2, Ok((2, 4)))));

  let result3 = race.next().await;
  info!("Result 3: {result3:?}");
  assert_eq!(result3, Some((3, Ok((3, 6)))));

  let result4 = race.next().await;
  info!("Result 4 (should be None): {result4:?}");
  assert_eq!(result4, None);

  info!("=== Basic Race Test Passed ===\n");
}

#[tokio::test]
async fn test_race_with_delays() {
  info!("=== Race with Delays Test ===");
  info!("Testing staggered task execution with different completion times");
  info!("Step interval: 50ms");
  info!("Tasks: [100ms, 20ms, 80ms] execution times");
  info!("Expected completion order:");
  info!("  1. Task 20ms  (starts at 50ms,  completes at ~70ms)");
  info!("  2. Task 100ms (starts at 0ms,   completes at ~100ms)");
  info!("  3. Task 80ms  (starts at 100ms, completes at ~180ms)");

  let timeout = timeout(Duration::from_secs(3), async {
    let start = Instant::now();

    let mut race = Race::new(
      Duration::from_millis(50),
      move |delay_ms: &u64| {
        let delay_ms = *delay_ms;
        let task_start = start.elapsed();
        async move {
          info!("Task {delay_ms}ms started at {:?}", task_start);
          sleep(Duration::from_millis(delay_ms)).await;
          let completed_at = task_start + Duration::from_millis(delay_ms);
          info!("Task {delay_ms}ms completed at ~{completed_at:?}");
          Ok::<(u64, u64), &'static str>((delay_ms, delay_ms))
        }
      },
      vec![100, 20, 80],
    );

    let result1 = race.next().await;
    info!("Result 1 at {:?}: {result1:?}", start.elapsed());
    assert_eq!(result1, Some((20, Ok((20, 20)))));

    let result2 = race.next().await;
    info!("Result 2 at {:?}: {result2:?}", start.elapsed());
    assert_eq!(result2, Some((100, Ok((100, 100)))));

    let result3 = race.next().await;
    info!("Result 3 at {:?}: {result3:?}", start.elapsed());
    assert_eq!(result3, Some((80, Ok((80, 80)))));

    let result4 = race.next().await;
    info!("Result 4 (should be None): {result4:?}");
    assert_eq!(result4, None);
  });

  timeout
    .await
    .expect("Test should complete within 3 seconds");

  info!("=== Race with Delays Test Passed ===\n");
}

#[tokio::test]
async fn test_infinite_tasks() {
  info!("=== Infinite Tasks Test ===");
  info!("Testing with infinite iterator, verifying concurrent execution");
  info!("Step interval: 50ms, each task sleeps 100ms");
  info!("If concurrent: 3 results in ~150ms (50ms + 100ms)");
  info!("If sequential: 3 results in ~300ms (3 * 100ms)");

  let timeout = timeout(Duration::from_secs(3), async {
    let start = Instant::now();

    // Infinite iterator / 无限迭代器
    let infinite_args = 0u64..;

    let mut race = Race::new(
      Duration::from_millis(50),
      move |n: &u64| {
        let n = *n;
        let task_start = start.elapsed();
        async move {
          info!("Task {n} started at {task_start:?}");
          sleep(Duration::from_millis(100)).await;
          info!(
            "Task {n} completed at {:?}",
            task_start + Duration::from_millis(100)
          );
          Ok::<(u64, u64), &'static str>((n, n))
        }
      },
      infinite_args,
    );

    // Consume first 3 results / 消费前 3 个结果
    for i in 1..=3 {
      type TaskResult = Option<(u64, Result<(u64, u64), &'static str>)>;
      let result: TaskResult = race.next().await;
      info!("Result {i} at {:?}: {result:?}", start.elapsed());
      assert!(result.is_some());
    }

    let total_time = start.elapsed();
    info!("Total time: {total_time:?}");

    // Verify concurrent execution / 验证并发执行
    // Task 0: starts at 0ms, completes at ~100ms
    // Task 1: starts at 50ms, completes at ~150ms
    // Task 2: starts at 100ms, completes at ~200ms
    // If concurrent, total time should be ~200ms, not 300ms
    assert!(
      total_time < Duration::from_millis(250),
      "Total time {total_time:?} too long, tasks may not be running concurrently"
    );

    info!("Concurrent execution verified: {total_time:?} < 250ms");
  });

  timeout
    .await
    .expect("Test should complete within 3 seconds");

  info!("=== Infinite Tasks Test Passed ===\n");
}
#[tokio::test]
async fn test_non_copy_types() {
  info!("=== Non-Copy Types Test ===");
  info!("Testing with String (non-Copy) as argument and return type");

  // Test with String arguments / 测试 String 参数
  let string_args = vec!["hello".to_string(), "world".to_string(), "rust".to_string()];

  let mut race = Race::new(
    Duration::from_millis(5),
    |s: &String| {
      let s = s.clone();
      async move {
        info!("Processing string: '{s}'");
        sleep(Duration::from_millis(10)).await;
        let result = format!("{s}_processed");
        info!("String '{s}' processed to '{result}'");
        Ok::<String, &'static str>(result)
      }
    },
    string_args,
  );

  let result1 = race.next().await;
  info!("Result 1: {result1:?}");
  assert!(
    matches!(result1, Some((ref arg, Ok(ref res))) if arg == "hello" && res == "hello_processed")
  );

  let result2 = race.next().await;
  info!("Result 2: {result2:?}");
  assert!(
    matches!(result2, Some((ref arg, Ok(ref res))) if arg == "world" && res == "world_processed")
  );

  let result3 = race.next().await;
  info!("Result 3: {result3:?}");
  assert!(
    matches!(result3, Some((ref arg, Ok(ref res))) if arg == "rust" && res == "rust_processed")
  );

  let result4 = race.next().await;
  info!("Result 4 (should be None): {result4:?}");
  assert_eq!(result4, None);

  info!("=== Non-Copy Types Test Passed ===\n");
}

#[derive(Debug, Clone, PartialEq)]
struct CustomData {
  id: u32,
  name: String,
  values: Vec<i32>,
}

#[tokio::test]
async fn test_custom_struct_types() {
  info!("=== Custom Struct Types Test ===");
  info!("Testing with custom struct (non-Copy) as argument and return type");

  let custom_args = vec![
    CustomData {
      id: 1,
      name: "first".to_string(),
      values: vec![1, 2, 3],
    },
    CustomData {
      id: 2,
      name: "second".to_string(),
      values: vec![4, 5, 6],
    },
  ];

  let mut race = Race::new(
    Duration::from_millis(10),
    |data: &CustomData| {
      let data = data.clone();
      async move {
        info!("Processing CustomData: {data:?}");
        sleep(Duration::from_millis(20)).await;

        let sum: i32 = data.values.iter().sum();
        let result = CustomData {
          id: data.id * 10,
          name: format!("{}_processed", data.name),
          values: vec![sum],
        };

        info!("CustomData processed: {result:?}");
        Ok::<CustomData, &'static str>(result)
      }
    },
    custom_args,
  );

  let result1 = race.next().await;
  info!("Result 1: {result1:?}");
  if let Some((arg, Ok(res))) = result1 {
    assert_eq!(arg.id, 1);
    assert_eq!(arg.name, "first");
    assert_eq!(res.id, 10);
    assert_eq!(res.name, "first_processed");
    assert_eq!(res.values, vec![6]); // 1+2+3=6
  } else {
    panic!("Expected successful result");
  }

  let result2 = race.next().await;
  info!("Result 2: {result2:?}");
  if let Some((arg, Ok(res))) = result2 {
    assert_eq!(arg.id, 2);
    assert_eq!(arg.name, "second");
    assert_eq!(res.id, 20);
    assert_eq!(res.name, "second_processed");
    assert_eq!(res.values, vec![15]); // 4+5+6=15
  } else {
    panic!("Expected successful result");
  }

  let result3 = race.next().await;
  info!("Result 3 (should be None): {result3:?}");
  assert_eq!(result3, None);

  info!("=== Custom Struct Types Test Passed ===\n");
}

#[tokio::test]
async fn test_vec_types() {
  info!("=== Vec Types Test ===");
  info!("Testing with Vec<T> (non-Copy) as argument and return type");

  let vec_args = vec![vec![1, 2, 3], vec![10, 20], vec![100]];

  let mut race = Race::new(
    Duration::from_millis(8),
    |nums: &Vec<i32>| {
      let nums = nums.clone();
      async move {
        info!("Processing vector: {nums:?}");
        sleep(Duration::from_millis(15)).await;

        let doubled: Vec<i32> = nums.iter().map(|x| x * 2).collect();
        info!("Vector doubled: {doubled:?}");
        Ok::<Vec<i32>, &'static str>(doubled)
      }
    },
    vec_args,
  );

  let result1 = race.next().await;
  info!("Result 1: {result1:?}");
  if let Some((arg, Ok(res))) = result1 {
    assert_eq!(arg, vec![1, 2, 3]);
    assert_eq!(res, vec![2, 4, 6]);
  } else {
    panic!("Expected successful result");
  }

  let result2 = race.next().await;
  info!("Result 2: {result2:?}");
  if let Some((arg, Ok(res))) = result2 {
    assert_eq!(arg, vec![10, 20]);
    assert_eq!(res, vec![20, 40]);
  } else {
    panic!("Expected successful result");
  }

  let result3 = race.next().await;
  info!("Result 3: {result3:?}");
  if let Some((arg, Ok(res))) = result3 {
    assert_eq!(arg, vec![100]);
    assert_eq!(res, vec![200]);
  } else {
    panic!("Expected successful result");
  }

  let result4 = race.next().await;
  info!("Result 4 (should be None): {result4:?}");
  assert_eq!(result4, None);

  info!("=== Vec Types Test Passed ===\n");
}
