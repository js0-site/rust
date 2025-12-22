#![cfg_attr(docsrs, feature(doc_cfg))]

mod bits;
pub use bits::{Layout, DefaultLayout};

mod error;
pub use error::{Error, Result};

#[cfg(feature = "auto_pid")]
mod pid;
#[cfg(feature = "auto_pid")]
#[cfg_attr(docsrs, doc(cfg(feature = "auto_pid")))]
pub use pid::{Pid, allocate};

#[cfg(feature = "snowflake")]
mod snowflake;
#[cfg(feature = "snowflake")]
#[cfg_attr(docsrs, doc(cfg(feature = "snowflake")))]
pub use snowflake::{EPOCH, Snowflake};

#[cfg(feature = "parse")]
mod parse;
#[cfg(feature = "parse")]
#[cfg_attr(docsrs, doc(cfg(feature = "parse")))]
pub use parse::{ParsedId, parse, parse_with};
