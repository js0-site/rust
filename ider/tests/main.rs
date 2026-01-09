use aok::{OK, Void};
use ider::Ider;
use log::info;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

#[test]
fn test() -> Void {
  info!("> test {}", 123456);
  OK
}

#[test]
fn test_monotonic() -> Void {
  let mut g = Ider::new();
  let mut prev = g.get();
  for _ in 0..1000 {
    let cur = g.get();
    assert!(cur > prev, "IDs must be monotonic");
    prev = cur;
  }
  OK
}

#[test]
fn test_backward_compatibility() -> Void {
  let mut g = Ider::new();
  let id1 = g.get();
  let id2 = g.get();
  assert!(id2 > id1, "Backward compatibility should work");
  OK
}

#[test]
fn test_init() -> Void {
  let mut g = Ider::new();
  let id1 = g.get();

  // Simulate recovery with future ID / 模拟用未来 ID 恢复
  let future_id = id1 + 1000;
  g.init(future_id);

  let id2 = g.get();
  assert!(id2 > future_id, "ID after init must be greater");
  OK
}

#[test]
fn test_iterator() -> Void {
  let mut g = Ider::new();
  let ids: Vec<_> = g.by_ref().take(5).collect();
  assert_eq!(ids.len(), 5);
  for i in 1..5 {
    assert!(ids[i] > ids[i - 1]);
  }
  OK
}
