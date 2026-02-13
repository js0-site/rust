#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "cache")]
mod cache;
mod config;
mod protocol;
pub mod resolve;

use futures_util::future::{Either, select};
use compio_runtime::{JoinHandle,CancelToken};

use std::{
  io,
  net::SocketAddr,
  task::{Poll, Waker},
};

use resolve::Resolve;

pub struct CompioResolver {
  handle: JoinHandle<io::Result<Vec<SocketAddr>>>,
  token: Option<CancelToken>,
}

compio_net::resolve_set!(CompioResolver);

impl Drop for CompioResolver {
  fn drop(&mut self) {
    if let Some(token) = self.token.take() {
      token.cancel();
    }
  }
}

impl compio_net::ExternResolve for CompioResolver {
  fn new(host: &str, port: u16) -> Self {
    let host = host.to_string();
    let token = CancelToken::new();
    let wait = token.clone().wait();
    let handle = compio_runtime::spawn(async move {
      // 解析器初始化（读取 /etc/hosts 等）和查询都在任务中异步执行
      let resolver = Resolve::new()?;
      let fut = async {
        Ok(
          resolver
            .lookup(&host)
            .await?
            .map(|mut addr| {
              addr.set_port(port);
              addr
            })
            .collect(),
        )
      };

      match select(std::pin::pin!(fut), std::pin::pin!(wait)).await {
        Either::Left((res, _)) => res,
        Either::Right(_) => Err(io::Error::new(io::ErrorKind::Interrupted, "task cancelled")),
      }
    });
    Self {
      handle,
      token: Some(token),
    }
  }

  fn poll(&mut self, waker: &Waker) -> Poll<io::Result<Vec<SocketAddr>>> {
    use std::future::Future;
    let mut cx = std::task::Context::from_waker(waker);
    match std::pin::Pin::new(&mut self.handle).poll(&mut cx) {
      Poll::Ready(Ok(res)) => Poll::Ready(res),
      Poll::Ready(Err(_)) => Poll::Ready(Err(io::Error::other("task panicked"))),
      Poll::Pending => Poll::Pending,
    }
  }
}
