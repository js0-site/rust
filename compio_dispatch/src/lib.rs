#![cfg_attr(docsrs, feature(doc_cfg))]

use std::future::Future;

use compio_dispatcher::Dispatcher;
use compio_driver::DispatchError;
use futures_channel::oneshot::Receiver;
use static_init::dynamic;

#[dynamic]
pub static DISPATCH: Dispatcher = Dispatcher::new().unwrap();

pub fn run<F, Fut, R>(f: F) -> Result<Receiver<R>, DispatchError<F>>
where
  F: FnOnce() -> Fut + 'static + Send,
  Fut: Future<Output = R> + 'static,
  R: Send + 'static,
{
  DISPATCH.dispatch(f)
}

pub fn blocking<F, R>(f: F) -> Result<Receiver<R>, DispatchError<F>>
where
  F: FnOnce() -> R + 'static + Send,
  R: 'static + Send,
{
  DISPATCH.dispatch_blocking(f)
}
