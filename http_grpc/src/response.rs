use bytes::Bytes;
use pilota::{LinkedBytes, pb::Message};

#[derive(Debug)]
pub struct Response {
  pub code: u32,
  pub body: Bytes,
}

impl<T: Message> From<xrpc::Result<T>> for Response {
  fn from(t: xrpc::Result<T>) -> Self {
    use xrpc::Result;
    match t {
      Result::Ok(t) => {
        let mut body = LinkedBytes::with_capacity(t.encoded_len());
        match t.encode(&mut body) {
          Ok(_) => Self {
            code: 0,
            body: body.into_bytes_mut().into(),
          },
          Err(err) => Self {
            code: 500,
            body: err.to_string().into(),
          },
        }
      }
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
