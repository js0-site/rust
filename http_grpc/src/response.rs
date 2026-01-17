use bytes::Bytes;
use pilota::pb::{EncodeLengthContext, Message};

#[derive(Debug)]
pub struct Response {
  pub code: u32,
  pub body: Bytes,
}

impl<T: Message> From<xrpc::Result<T>> for Response {
  fn from(t: xrpc::Result<T>) -> Self {
    use xrpc::Result;
    let mut ctx = EncodeLengthContext::default();
    match t {
      Result::Ok(t) => Self {
        code: 0,
        body: t.encode_to_vec(&mut ctx).into(),
      },
      Result::Err(err) => Self {
        code: 500,
        body: err.to_string().into(),
      },
      Result::Response(r) => Self {
        code: r.code as _,
        body: r.body,
      },
    }
  }
}
