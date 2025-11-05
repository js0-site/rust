use std::sync::Arc;

use aok::OK;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use futures_util::stream::StreamExt;
use http_body_util::BodyExt;
use tokio::sync::mpsc::Sender;
use volo_http::{
  body::Body, http::header::HeaderMap, request::Request, response::Response, server::IntoResponse,
};
use xbin::concat;
use xrpc::{HeadersExt, volo::http::ExtMap};

use crate::{
  Grpc,
  error::Error,
  pb::{encode_u32, get_u32, get_u32_bin},
};

async fn read_body_frame(body: &mut Body, buf: &mut BytesMut) -> Result<bool, Error> {
  if let Some(Ok(chunk)) = body.frame().await {
    buf.put(chunk.into_data().unwrap_or_default());
    Ok(true)
  } else {
    Ok(false)
  }
}

pub async fn http_grpc<G: Grpc>(req: Request) -> Response {
  match _http_grpc::<G>(req).await {
    Ok(body) => body.into_response(),
    Err(err) => Response::builder()
      .status(400)
      .body(err.to_string().into())
      .unwrap(),
  }
}

pub const MAX_CALL: usize = 64;

async fn _http_grpc<G: Grpc>(req: Request) -> std::result::Result<Body, Error> {
  let (req, mut body) = req.into_parts();

  let mut buf = BytesMut::new();

  macro_rules! read {
    () => {
      read_body_frame(&mut body, &mut buf).await?
    };
    ($($label:tt)+) => {
      if !read!() {
        break $($label)+;
      }
    };
  }

  let mut pending = vec![];
  let mut n = 0;

  if read!() {
    'out: loop {
      if let Some(call_bin) = get_u32_bin(&mut buf)? {
        loop {
          if let Some(func_id) = get_u32(&mut buf)? {
            loop {
              if let Some(data_len) = get_u32(&mut buf)? {
                let data_len = data_len as usize;
                while buf.remaining() < data_len {
                  read!('out);
                }
                let call_bin = call_bin.clone();
                let bytes = buf.split_to(data_len).freeze();
                pending.push((call_bin, func_id, bytes));

                n += 1;
                if n > MAX_CALL {
                  return Err(Error::TooManyCalls);
                }
                continue 'out;
              }
              read!('out)
            }
          }
          read!('out)
        }
      }
      read!('out);
    }
  }

  let req = Arc::new(HeadersExt {
    headers: req.headers,
    ext: ExtMap::new(),
  });

  let (sender, recv) = tokio::sync::mpsc::channel(1);

  let last = pending.pop();

  for task in pending {
    run::<G>(sender.clone(), req.clone(), task);
  }

  if let Some(task) = last {
    run::<G>(sender, req, task);
  }

  let stream = tokio_stream::wrappers::ReceiverStream::new(recv);
  let stream = stream.map(|data| Ok(volo_http::hyper::body::Frame::data(data)));

  Ok(Body::from_stream(stream))
}

pub fn run<G: Grpc>(
  sender: Sender<Bytes>,
  req: Arc<HeadersExt<HeaderMap, ExtMap>>,
  (call_bin, func_id, bytes): (Bytes, u32, Bytes),
) {
  tokio::spawn(async move {
    let r = G::run(req, func_id, bytes).await;
    xerr::log!(
      sender
        .send(
          if let Some(r) = r {
            concat!(
              call_bin,
              encode_u32(r.code),
              encode_u32(r.body.len() as _),
              r.body
            )
          } else {
            concat!(call_bin, encode_u32(u32::MAX))
          }
          .into(),
        )
        .await
    );
    OK
  });
}
