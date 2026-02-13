use std::time::Duration;

use aok::{OK, Void};
use cpu_load::{CPU_LOAD, CpuLoad};
use log::trace;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

#[compio::test]
async fn test_iterator() -> Void {
  // 创建 CPU 负载监控器，采样间隔为 100ms
  let cpu_monitor = CpuLoad::init(Duration::from_millis(100));

  // 等待一段时间让后台任务收集数据
  // 需要等待足够长时间让sysinfo库收集到真实的CPU使用率数据
  compio::time::sleep(Duration::from_millis(1000)).await;

  // 获取 CPU 核心数量
  let core_count = cpu_monitor.len();
  trace!("检测到 {} 个 CPU 核心", core_count);

  // 测试迭代器基本功能
  let loads: Vec<u8> = cpu_monitor.into_iter().collect();
  assert_eq!(loads.len(), core_count, "迭代器应返回与核心数量相同的元素");

  // 测试迭代器枚举
  for (i, load) in cpu_monitor.into_iter().enumerate() {
    trace!("核心 {} 负载: {}%", i, load);
    assert!(load <= 100, "负载值应在 0-100 范围内");
    assert!(i < core_count, "索引应在有效范围内");
  }

  // 测试迭代器大小提示
  let iter = cpu_monitor.into_iter();
  let (lower, upper) = iter.size_hint();
  assert_eq!(lower, core_count, "大小提示下限应等于核心数量");
  assert_eq!(upper, Some(core_count), "大小提示上限应等于核心数量");

  // 测试 ExactSizeIterator
  assert_eq!(iter.len(), core_count, "迭代器长度应等于核心数量");

  // 测试迭代器链式操作
  let avg_load = cpu_monitor.into_iter().map(|x| x as u16).sum::<u16>() as f32 / core_count as f32;
  trace!("平均负载: {:.2}%", avg_load);
  assert!(
    (0.0..=100.0).contains(&avg_load),
    "平均负载应在 0-100 范围内"
  );

  // 测试过滤操作
  let high_load_cores: Vec<usize> = cpu_monitor
    .into_iter()
    .enumerate()
    .filter(|(_, load)| *load > 50)
    .map(|(i, _)| i)
    .collect();

  if !high_load_cores.is_empty() {
    trace!("高负载核心 (>50%): {:?}", high_load_cores);
  }

  // 测试查找操作
  if let Some(max_load) = cpu_monitor.into_iter().max() {
    trace!("最高负载: {}%", max_load);
    assert!(max_load <= 100, "最大负载应在 0-100 范围内");
  }

  if let Some(min_load) = cpu_monitor.into_iter().min() {
    trace!("最低负载: {}%", min_load);
    assert!(min_load <= 100, "最小负载应在 0-100 范围内");
  }

  trace!("迭代器测试完成");

  // 测试迭代器收集到不同集合类型
  let loads_vec: Vec<u8> = cpu_monitor.into_iter().collect();
  let loads_array = loads_vec.clone().into_iter().collect::<Vec<u8>>();
  trace!("收集到的负载数组: {:?}", &loads_array);
  trace!("实际收集的负载向量: {:?}", loads_vec);
  OK
}

#[compio::test]
async fn test_cpu_load_static() -> Void {
  // Wait for background task to collect data
  // 等待后台任务收集数据
  compio::time::sleep(Duration::from_millis(1100)).await;

  let core_count = CPU_LOAD.len();
  trace!("CPU_LOAD 检测到 {core_count} 个核心");
  assert!(core_count > 0);

  // Test multiple samples over 3 seconds
  // 测试 3 秒内的多次采样
  for round in 0..3 {
    compio::time::sleep(Duration::from_secs(1)).await;

    let global = CPU_LOAD.global();
    trace!("第 {round} 轮: 全局负载 {global}%");
    assert!(global <= 100);

    let idlest = CPU_LOAD.idlest();
    trace!("第 {round} 轮: 最空闲核心 {idlest}");
    assert!(idlest < core_count);

    // Test iteration
    // 测试迭代
    for (i, load) in CPU_LOAD.into_iter().enumerate() {
      trace!("第 {round} 轮: 核心 {i}: {load}%");
      assert!(load <= 100);
    }
  }

  OK
}
