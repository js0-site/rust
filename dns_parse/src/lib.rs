#![cfg_attr(docsrs, feature(doc_cfg))]

mod error;
mod parse;

pub use error::{Error, Result};
pub use parse::{build, parse};
