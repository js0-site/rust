#![cfg_attr(docsrs, feature(doc_cfg))]

mod error;
pub use error::Error;
mod http_grpc;
mod response;
use bytes::Bytes;
use xrpc::volo::http::Req;

mod pb;
pub use http_grpc::http_grpc;
pub use pb::{DecodeError, decode_u32, encode_u32, get_u32, get_u32_bin};
pub use response::Response;

pub trait Grpc {
  fn run(req: Req, func_id: u32, args: Bytes) -> impl Future<Output = Option<Response>> + Send;
}
