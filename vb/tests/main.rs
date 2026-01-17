use vb::{d, d_li, e_li};

#[test]
fn test_single_value_roundtrip() {
  let values = vec![0, 1, 127, 128, 300, 16383, 16384, u64::MAX];
  for &v in &values {
    let encoded = e_li([v]);
    let (decoded, consumed) = d(&encoded).unwrap();
    assert_eq!(decoded, v, "Failed to decode value: {}", v);
    assert_eq!(
      consumed,
      encoded.len(),
      "Consumed length mismatch for value: {}",
      v
    );
  }
}

#[test]
fn test_list_roundtrip() {
  let list = vec![0, 1, 127, 128, 300, 16383, 16384, 1234567890];
  let encoded = e_li(list.iter().cloned());
  let decoded = d_li(&encoded).unwrap();
  assert_eq!(decoded, list);
}

#[cfg(feature = "diff")]
#[test]
fn test_diff_roundtrip() {
  use vb::{d_diff, e_diff};

  // Case 1: Strictly increasing sequence
  let list = vec![1, 5, 10, 100, 1000, 10000, 1234567890];
  let encoded = e_diff(&list);
  let decoded = d_diff(&encoded).unwrap();
  assert_eq!(decoded, list);

  // Case 2: Empty list
  let list: Vec<u64> = vec![];
  let encoded = e_diff(&list);
  assert!(encoded.is_empty());
  let decoded = d_diff(&encoded).unwrap();
  assert_eq!(decoded, list);

  // Case 3: Single element
  let list = vec![42];
  let encoded = e_diff(&list);
  let decoded = d_diff(&encoded).unwrap();
  assert_eq!(decoded, list);

  // Case 4: Large gaps
  let list = vec![0, 1000000, 2000000000, u64::MAX];
  let encoded = e_diff(&list);
  let decoded = d_diff(&encoded).unwrap();
  assert_eq!(decoded, list);
}

#[cfg(feature = "diff")]
#[test]
fn test_diff_compression() {
  use vb::{d_diff, e_diff};
  // Sequence with small differences
  let list = vec![10000, 10001, 10002, 10003, 10004];
  let encoded_normal = e_li(list.iter().cloned());
  let encoded_diff = e_diff(&list);

  // 10000 takes 2 bytes (0x2710 -> 0010 0111 0001 0000 -> 2 groups of 7 bits).
  // Actually 10000 is 0x2710. 14 bits.
  // 10000 = 10 0111 0001 0000. 14 bits needed. 2 bytes.
  // Normal: 2 * 5 = 10 bytes.
  // Diff: 10000 (2 bytes), 1 (1 byte), 1 (1 byte)... -> 2 + 4 = 6 bytes.

  println!(
    "Normal len: {}, Diff len: {}",
    encoded_normal.len(),
    encoded_diff.len()
  );
  assert!(
    encoded_diff.len() < encoded_normal.len(),
    "Diff encoding should be smaller for sequential data"
  );

  let decoded = d_diff(&encoded_diff).unwrap();
  assert_eq!(decoded, list);
}
