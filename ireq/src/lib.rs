use std::time::Duration;
mod error;
use bytes::Bytes;
pub use error::{Error, Result};
pub use reqwest;
use reqwest::{Body, Client, IntoUrl, RequestBuilder, StatusCode, redirect::Policy};

#[static_init::dynamic]
pub static REQ: Client = {
  let b = Client::builder()
    .redirect(Policy::limited(6))
    .timeout(Duration::from_secs(100))
    .zstd(true)
    .connect_timeout(Duration::from_secs(9));

  #[cfg(feature = "proxy")]
  let b = {
    use reqwest::Proxy;
    // export https_proxy=http://127.0.0.1:7890 http_proxy=http://127.0.0.1:7890 all_proxy=socks5://127.0.0.1:7890
    let mut b = b;
    if let Ok(url) = std::env::var("https_proxy") {
      b = b.proxy(Proxy::https(url).unwrap());
    }

    b
  };

  b.build().unwrap()
};

pub const SUCCESS_STATUS: [StatusCode; 5] = [
  StatusCode::OK,
  StatusCode::NO_CONTENT,
  StatusCode::PERMANENT_REDIRECT,
  StatusCode::TEMPORARY_REDIRECT,
  StatusCode::PARTIAL_CONTENT,
];

pub async fn req(req: RequestBuilder) -> Result<Bytes> {
  let res = req.send().await?;
  let status = res.status();
  if SUCCESS_STATUS.contains(&status) {
    let bin = res.bytes().await?;
    Ok(bin)
  } else {
    Err(Error::Status(res))
  }
}

pub async fn getbin(url: impl IntoUrl) -> Result<Bytes> {
  let url = url.into_url()?;
  req(REQ.get(url)).await
}

pub async fn get(url: impl IntoUrl) -> Result<String> {
  let url = url.into_url()?;
  Ok(String::from_utf8_lossy(&req(REQ.get(url)).await?).into())
}

macro_rules! method {
  ($($method: ident),*) => {
    $(
    pub async fn $method(url: impl IntoUrl, body:impl Into<Body>) -> Result<String> {
      let url = url.into_url()?;
      let r = REQ.$method(url.clone()).body(body);
      Ok(String::from_utf8_lossy(&req(r).await?).into())
    }
    )*
  };
}

method!(post, delete, patch, put);
