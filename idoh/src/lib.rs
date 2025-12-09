#![cfg_attr(docsrs, feature(doc_cfg))]

mod error;
mod post;
pub mod record_type;
mod resolve;

pub use error::{Error, Result};

#[cfg(feature = "mx")]
pub mod mx;

#[cfg(feature = "mx")]
pub use mx::{Mx, MxLookup};
pub use post::{Answer, post};
pub use resolve::{DOH_LI, resolve};
mod resolve_trait;
pub use resolve_trait::{Resolve, Resolver};
