use std::sync::atomic::{AtomicUsize, Ordering};

use aok::{OK, Void};
use fred::interfaces::{KeysInterface, StreamsInterface};
use log::info;
use msgq::{Kv, ReadGroup};
use tokio::time::Duration;
use xkv::R;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

static SUCCESS_COUNT: AtomicUsize = AtomicUsize::new(0);
static ERROR_COUNT: AtomicUsize = AtomicUsize::new(0);
static RETRY_COUNT: AtomicUsize = AtomicUsize::new(0);

struct MailParser;

impl msgq::Parse for MailParser {
  async fn run(&self, kv: &Kv, retry: u64) -> aok::Result<Option<Kv>> {
    if kv.is_empty() {
      return Err(aok::anyhow!("Empty message"));
    }

    let n = String::from_utf8(kv[0].1.to_vec())?.parse::<u64>()?;
    info!("Processing message n={} retry={}", n, retry);

    // Simulate different scenarios:
    // n=1: Fail twice, succeed on retry 2
    // n=2: Always fail (will go to on_error after max_retry)
    // Others: Succeed immediately

    if n == 1 {
      RETRY_COUNT.fetch_add(1, Ordering::SeqCst);
      if retry < 2 {
        // Simulate processing delay
        tokio::time::sleep(Duration::from_millis(50)).await;
        return Err(aok::anyhow!("Simulated failure for n=1, retry={}", retry));
      }
    } else if n == 2 {
      return Err(aok::anyhow!("Simulated permanent failure for n=2"));
    }

    SUCCESS_COUNT.fetch_add(1, Ordering::SeqCst);
    Ok(None)
  }

  async fn on_error(&self, kv: Kv, error: String) -> Void {
    let n = if !kv.is_empty() {
      String::from_utf8(kv[0].1.to_vec())
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
    } else {
      None
    };

    info!("on_error: n={:?} error={}", n, error);
    ERROR_COUNT.fetch_add(1, Ordering::SeqCst);
    OK
  }
}

#[tokio::test]
async fn test_async() -> Void {
  xboot::init().await?;
  let stream = "msgq_test";

  // Cleanup and prepare
  let p = R.pipeline();
  let _: () = p.del(stream).await?;

  // Insert test messages (0-9)
  for i in 0..10 {
    let _: () = p
      .xadd(stream, false, None, "*", vec![("n", i.to_string())])
      .await?;
  }
  let _: () = p.last().await?;

  // Configure consumer with batch size 2
  // Use 0 block time and very short claim time for testing
  let config = msgq::Conf::new(
    stream,
    "test_group",
    "test_consumer",
    0, // block_sec (0 = no blocking, return immediately)
    0, // claim_idle_sec (0 = claim immediately)
    2, // count (batch size)
    3, // max_retry
  );

  let read_group = ReadGroup::new(MailParser, config);

  // Run consumer in background and let it process for a bit
  let handle = tokio::spawn(async move { read_group.run().await });

  // Give it time to process and retry messages
  tokio::time::sleep(Duration::from_secs(2)).await;

  // Note: The consumer will keep running until stream is empty
  // With claim_idle_ms=0, failed messages should be claimed immediately
  // Wait for completion
  let _ = tokio::time::timeout(Duration::from_secs(5), handle).await;

  // Verify results
  let success = SUCCESS_COUNT.load(Ordering::SeqCst);
  let error = ERROR_COUNT.load(Ordering::SeqCst);
  let retry = RETRY_COUNT.load(Ordering::SeqCst);

  info!(
    "Test results: success={} error={} retry={}",
    success, error, retry
  );

  // Expected results:
  // n=0, 3-9: 8 messages succeed immediately
  // n=1: Fails at retry 0 and 1, succeeds at retry 2 (3 total runs, 1 success)
  // n=2: Fails all retries (0,1,2,3), goes to on_error (4 total runs, 0 success, 1 error)
  // Total: 9 successes, 1 error

  assert_eq!(success, 9, "Expected 9 successful messages");
  assert_eq!(error, 1, "Expected 1 error (n=2)");
  assert!(
    retry >= 3,
    "Expected at least 3 retry attempts for n=1, got {}",
    retry
  );

  // Cleanup
  let _: () = R.del(stream).await?;
  OK
}
