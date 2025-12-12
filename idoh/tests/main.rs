use aok::{OK, Void};
use idoh::{MxLookup, Resolve};
use log::info;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

#[tokio::test]
async fn test_async() -> Void {
  let domain = "gmail.com";

  let mx_records = Resolve.mx(domain).await?;

  if let Some(mx_records) = mx_records {
    for mx in mx_records.iter() {
      info!(
        "MX Record - Priority: {}, Server: {}, TTL: {}",
        mx.priority, mx.server, mx.ttl
      );
    }
  }

  OK
}

#[tokio::test]
async fn test_cache() -> Void {
  use idoh::mx::cache::Cache;

  let domain = "gmail.com";

  // First call - should fetch from DNS and cache it
  let start = std::time::Instant::now();
  let mx_records1 = Cache.mx(domain).await?;
  let first_duration = start.elapsed();

  info!("First call took: {:?}", first_duration);
  if let Some(ref records) = mx_records1 {
    info!("Found {} MX records", records.len());
  }

  // Second call - should return from cache (faster)
  let start = std::time::Instant::now();
  let mx_records2 = Cache.mx(domain).await?;
  let second_duration = start.elapsed();

  info!("Second call took: {:?}", second_duration);

  assert!(second_duration.as_micros() <= 3);
  // Verify results are the same
  if let (Some(records1), Some(records2)) = (&mx_records1, &mx_records2) {
    assert_eq!(records1.len(), records2.len());
    for (mx1, mx2) in records1.iter().zip(records2.iter()) {
      assert_eq!(mx1.priority, mx2.priority);
      assert_eq!(mx1.server, mx2.server);
    }
  }

  info!("Cache test passed - second call was faster and results matched");

  OK
}
