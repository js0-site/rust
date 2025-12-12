use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  // IO 错误 | IO errors
  #[error(transparent)]
  Io(#[from] std::io::Error),

  // 下载错误 | Download errors
  #[error(transparent)]
  Down(#[from] down::Error),

  // DNS 解析错误 | DNS resolution errors
  #[error(transparent)]
  Idoh(#[from] idoh::Error),

  // 升级验证错误 | Upgrade verification errors
  #[error(transparent)]
  UpgradeVerify(#[from] upgrade_verify::Error),

  // Tokio join 错误 | Tokio join errors
  #[error(transparent)]
  TokioJoin(#[from] tokio::task::JoinError),

  #[error(transparent)]
  VerFromTxt(#[from] ver_from_txt::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
