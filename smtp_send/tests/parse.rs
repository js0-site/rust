use smtp_send::{add_received, recv_overflow};

#[test]
fn test_no_overflow() {
  let body = b"From: test@example.com\r\nTo: recv@example.com\r\n\r\nHello";
  assert!(!recv_overflow(body));
}

#[test]
fn test_add_received() {
  let mut body = b"From: test@example.com\r\n\r\nHello".to_vec();
  add_received(&mut body, "sender.com", "relay.com");

  let content = String::from_utf8_lossy(&body);
  assert!(content.starts_with("Received: from sender.com by relay.com;"));
  assert!(content.contains("From: test@example.com"));
}
