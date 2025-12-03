use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  #[error("Network error: {0}")]
  Network(#[from] reqwest::Error),

  #[error("JSON parse error: {0}")]
  Json(#[from] serde_json::Error),

  #[error("DNS resolution error: {0}")]
  Dns(String),

  #[error("No MX records found for {0}")]
  NoMxRecords(String),

  #[error("No IP records found for {0}")]
  NoIpRecords(String),
}

pub type Result<T> = std::result::Result<T, Error>;
