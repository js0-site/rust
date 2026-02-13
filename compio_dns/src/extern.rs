use std::{
    future::Future,
    io,
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use compio_net::resolve::sys::ExternResolve;
use compio_runtime::JoinHandle;

pub struct Resolve {
    handle: JoinHandle<io::Result<Vec<SocketAddr>>>,
}

impl ExternResolve for Resolve {
    fn new(host: &str, port: u16) -> Self {
        let host = host.to_string();
        let handle = compio_runtime::spawn(async move {
            crate::resolve_sock_addrs(&host, port)
                .await
                .map(|iter| iter.collect::<Vec<_>>())
        });
        Self { handle }
    }

    fn poll(&mut self, waker: &Waker) -> Poll<io::Result<Vec<SocketAddr>>> {
        let mut cx = Context::from_waker(waker);
        match Pin::new(&mut self.handle).poll(&mut cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(res)) => Poll::Ready(res),
            Poll::Ready(Err(_)) => Poll::Ready(Err(io::Error::new(
                io::ErrorKind::Other,
                "background resolver task panicked",
            ))),
        }
    }
}

compio_net::resolve_set!(Resolve);
