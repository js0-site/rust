use std::{io, result};

use thiserror::Error;
use tokio::task::JoinError;

#[derive(Error, Debug)]
pub enum Error {
  // IO 错误 | IO errors
  #[error(transparent)]
  Io(#[from] io::Error),

  // 下载错误 | Download errors
  #[error("Download error: {0}")]
  Down(#[from] Box<down::Error>),

  // DNS 解析错误 | DNS resolution errors
  #[error("DNS resolution error: {0}")]
  Idoh(#[from] Box<idoh::Error>),

  // 升级验证错误 | Upgrade verification errors
  #[error("Upgrade verification error: {0}")]
  UpgradeVerify(#[from] Box<upgrade_verify::Error>),

  // Tokio join 错误 | Tokio join errors
  #[error(transparent)]
  TokioJoin(#[from] JoinError),

  #[error("Ver from txt error: {0}")]
  VerFromTxt(#[from] Box<ver_from_txt::Error>),
}

pub type Result<T> = result::Result<T, Error>;
