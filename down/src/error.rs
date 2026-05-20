use std::{io, result};

use crossfire::SendError;
use ireq::reqwest::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  // HTTP 响应错误 | HTTP response error
  #[error("HTTP error: status={0}")]
  HttpResponse(StatusCode),

  // HTTP 请求错误 | HTTP request error
  #[error(transparent)]
  Reqwest(#[from] ireq::reqwest::Error),

  // IO 错误 | IO error
  #[error(transparent)]
  Io(#[from] io::Error),

  // 通道发送错误 | Channel send error
  #[error("Channel send error")]
  SendError,
}

impl<T> From<SendError<T>> for Error {
  fn from(_: SendError<T>) -> Self {
    Error::SendError
  }
}

pub type Result<T> = result::Result<T, Error>;
