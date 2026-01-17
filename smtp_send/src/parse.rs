use jiff::Zoned;
use mail_parser::{HeaderName, MessageParser};

// 最大 Received 头部数量 / max Received header count (industry standard: 25-100)
pub const MAX_RECEIVED: usize = 30;

// Received 头部是否超限 / check if Received headers exceed limit
pub fn recv_overflow(body: &[u8]) -> bool {
  let Some(msg) = MessageParser::default().parse(body) else {
    return false;
  };

  msg
    .headers()
    .iter()
    .filter(|h| h.name == HeaderName::Received)
    .count()
    >= MAX_RECEIVED
}

// 添加 Received 头部 / add Received header
pub fn add_received(body: &mut Vec<u8>, from: &str, by: &str) {
  let now = Zoned::now().strftime("%a, %d %b %Y %H:%M:%S %z");
  let header = format!("Received: from {from} by {by}; {now}\r\n");
  body.splice(0..0, header.bytes());
}
