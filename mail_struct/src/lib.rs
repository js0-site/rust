#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "send")]
mod send;

#[cfg(feature = "decode")]
use bitcode::Decode;
#[cfg(feature = "encode")]
use bitcode::Encode;
#[cfg(feature = "send")]
pub use send::DomainMail;

#[derive(Debug, Clone)]
pub struct UserMail {
  pub mail: Mail,
  pub user_id: u64,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "decode", derive(Decode))]
#[cfg_attr(feature = "encode", derive(Encode))]
pub struct Mail {
  pub sender: String,
  pub to_li: Vec<String>,
  pub body: Vec<u8>,
}
