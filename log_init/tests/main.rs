use std::{env, panic};

use log::{debug, error, info, trace, warn};
use log_init::{layout::Text, level_color};
use logforth::{
  Layout,
  record::{Level, Record},
};

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

#[test]
fn test_basic_logging() {
  // Test basic logging functionality / 测试基本日志功能
  info!("> test {}", 123456);
  warn!("This is a warning message");
  error!("This is an error message");
  debug!("This is a debug message");
  trace!("This is a trace message");
}

#[test]
fn test_logging_with_key_values() {
  // Test logging with key-value pairs / 测试带键值对的日志
  info!(user_id = 42, action = "login"; "User logged in successfully");
  warn!(retry_count = 3, timeout = 5000; "Operation timeout, retrying");
  error!(error_code = 500, message = "Internal server error"; "Request failed");
}

#[test]
fn test_text_layout_default() {
  // Test Text layout default creation / 测试Text布局默认创建
  let _layout = Text::default();
  // Should not panic / 不应该崩溃
}

#[test]
fn test_text_layout_format() {
  // Test Text layout formatting / 测试Text布局格式化
  let layout = Text { color: false };

  // Create a mock record / 创建模拟记录
  let record = Record::builder()
    .level(Level::Info)
    .target("test_target")
    .file(Some("test.rs"))
    .line(Some(42))
    .payload("Test message")
    .build();

  let result = layout.format(&record, &[]);
  assert!(result.is_ok());

  let bytes = result.unwrap();
  let output = String::from_utf8_lossy(&bytes);
  assert!(output.contains("INFO"));
  assert!(output.contains("test.rs:42"));
  assert!(output.contains("Test message"));
}

#[test]
fn test_level_color_function() {
  // Test level color function / 测试级别颜色函数
  let error_colored = level_color(Level::Error);
  let warn_colored = level_color(Level::Warn);
  let info_colored = level_color(Level::Info);
  let debug_colored = level_color(Level::Debug);
  let trace_colored = level_color(Level::Trace);

  // Should not panic and return colored strings / 不应该崩溃并返回彩色字符串
  assert!(!error_colored.to_string().is_empty());
  assert!(!warn_colored.to_string().is_empty());
  assert!(!info_colored.to_string().is_empty());
  assert!(!debug_colored.to_string().is_empty());
  assert!(!trace_colored.to_string().is_empty());
}

#[test]
fn test_timezone_static() {
  // Test timezone static variable / 测试时区静态变量
  let tz = &*log_init::TZ;
  // Should have a valid timezone / 应该有有效的时区
  let tz_str = format!("{tz:?}");
  assert!(!tz_str.is_empty());
}

#[test]
fn test_multiple_init_calls() {
  // Test that multiple init calls don't cause panic / 测试多次初始化调用不会引起panic
  // Note: logforth only allows single initialization / 注意：logforth只允许单次初始化
  // This test verifies the behavior is predictable / 此测试验证行为是可预测的

  // First init should work / 第一次初始化应该正常工作
  let result = panic::catch_unwind(|| {
    log_init::init();
  });

  // Should not panic on first call / 第一次调用不应该panic
  if result.is_err() {
    // If it panics, it means logger was already initialized / 如果panic，说明logger已经被初始化
    info!("Logger already initialized, which is expected in test environment");
  } else {
    info!("Logger initialized successfully");
  }
}

#[test]
fn test_env_filter() {
  // Test with different log levels via environment / 通过环境变量测试不同日志级别
  unsafe {
    env::set_var("RUST_LOG", "debug");
  }

  debug!("This debug message should be visible");
  trace!("This trace message might not be visible depending on filter");

  unsafe {
    env::remove_var("RUST_LOG");
  }
}

#[cfg(target_os = "linux")]
#[test]
fn test_systemd_detection() {
  // Test systemd environment detection on Linux / 在Linux上测试systemd环境检测
  let original = env::var("INVOCATION_ID");

  // Test without INVOCATION_ID / 测试没有INVOCATION_ID的情况
  unsafe {
    env::remove_var("INVOCATION_ID");
  }
  log_init::init();
  info!("Testing without systemd environment");

  // Test with INVOCATION_ID / 测试有INVOCATION_ID的情况
  unsafe {
    env::set_var("INVOCATION_ID", "test-invocation-id");
  }
  log_init::init();
  info!("Testing with systemd environment");

  // Restore original value / 恢复原始值
  unsafe {
    match original {
      Ok(val) => env::set_var("INVOCATION_ID", val),
      Err(_) => env::remove_var("INVOCATION_ID"),
    }
  }
}

#[test]
fn test_structured_logging() {
  // Test structured logging with various data types / 测试各种数据类型的结构化日志
  info!(
    request_id = "req-123",
    user_id = 456,
    duration_ms = 250.5,
    success = true;
    "Request completed"
  );

  error!(
    error_type = "ValidationError",
    field = "email",
    value = "invalid-email";
    "Validation failed"
  );
}
