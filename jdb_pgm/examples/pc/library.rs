#[derive(Default, Clone)]
pub struct Metrics {
  pub size_mb: f64,
  pub ratio_pct: f64,
  pub build_mops: f64,
  pub get_mops: f64,
  pub iter_mops: f64,
  pub rev_mops: Option<f64>,
  pub latency_p99_ns: f64,
}

pub trait Library {
  const NAME: &'static str;
  fn measure(data: &[u64]) -> Metrics;
}

pub const N_QUERIES: usize = 200_000;
pub const SEED: u64 = 12345;
