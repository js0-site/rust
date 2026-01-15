use aok::Void;
use ider::Ider;
use log::info;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

#[test]
fn test() -> Void {
  info!("> test {}", 123456);
  Ok(())
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
  Ok(())
}

#[test]
fn test_backward_compatibility() -> Void {
  let mut g = Ider::new();
  let id1 = g.get();
  let id2 = g.get();
  assert!(id2 > id1, "Backward compatibility should work");
  Ok(())
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
  Ok(())
}

#[test]
fn test_iterator() -> Void {
  let mut g = Ider::new();
  let ids: Vec<_> = g.by_ref().take(5).collect();
  assert_eq!(ids.len(), 5);
  for i in 1..5 {
    assert!(ids[i] > ids[i - 1]);
  }
  Ok(())
}

#[cfg(feature = "path")]
mod path_tests {
  use std::path::Path;

  use aok::Void;
  use ider::path::{decode, encode, new};

  #[test]
  fn test_encode_decode() -> Void {
    let test_id = 1234567890u64;
    let encoded = encode(test_id);
    let decoded = decode(&encoded);

    assert!(decoded.is_some(), "Decoding should succeed");
    assert_eq!(
      decoded.unwrap(),
      test_id,
      "Decoded ID should match original"
    );
    Ok(())
  }

  #[test]
  fn test_encode_zero() -> Void {
    let encoded = encode(0);
    assert!(!encoded.is_empty(), "Encoded zero should not be empty");
    let decoded = decode(&encoded);
    assert_eq!(decoded, Some(0), "Decoding zero should return 0");
    Ok(())
  }

  #[test]
  fn test_encode_max() -> Void {
    let max_id = u64::MAX;
    let encoded = encode(max_id);
    let decoded = decode(&encoded);
    assert_eq!(
      decoded,
      Some(max_id),
      "Decoding max u64 should return max u64"
    );
    Ok(())
  }

  #[test]
  fn test_decode_invalid() -> Void {
    assert_eq!(decode(""), None, "Empty string should decode to None");
    assert_eq!(
      decode("invalid!@#"),
      None,
      "Invalid characters should decode to None"
    );
    Ok(())
  }

  #[test]
  fn test_id_path() -> Void {
    let dir = Path::new("/tmp");
    let (id, path) = new(dir);

    assert!(
      path.starts_with(dir),
      "Path should start with the specified directory"
    );
    assert!(id > 0, "Generated ID should be positive");
    Ok(())
  }

  #[test]
  fn test_roundtrip() -> Void {
    let test_cases = vec![0u64, 1, 100, 10000, 1234567890, u64::MAX];

    for id in test_cases {
      let encoded = encode(id);
      let decoded = decode(&encoded);
      assert_eq!(
        decoded,
        Some(id),
        "Roundtrip failed for id: {} -> {} -> {:?}",
        id,
        encoded,
        decoded
      );
    }
    Ok(())
  }
}
