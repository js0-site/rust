use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  // HTTP 响应错误 | HTTP response error
  #[error("HTTP error: status={0}")]
  HttpResponse(ireq::reqwest::StatusCode),

  // HTTP 请求错误 | HTTP request error
  #[error(transparent)]
  Reqwest(#[from] ireq::reqwest::Error),

  // IO 错误 | IO error
  #[error(transparent)]
  Io(#[from] std::io::Error),

  // 通道发送错误 | Channel send error
  #[error("Channel send error")]
  SendError,
}

impl<T> From<crossfire::SendError<T>> for Error {
  fn from(_: crossfire::SendError<T>) -> Self {
    Error::SendError
  }
}

pub type Result<T> = std::result::Result<T, Error>;
