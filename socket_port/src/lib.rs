#![cfg_attr(docsrs, feature(doc_cfg))]

use std::net::{Ipv6Addr, SocketAddr, TcpListener};

use socket2::{Domain, Socket, Type};

pub fn listen(port: u16) -> std::io::Result<TcpListener> {
  let socket = Socket::new(Domain::IPV6, Type::STREAM, None)?;

  socket.set_only_v6(false)?;

  #[cfg(not(windows))]
  socket.set_reuse_port(true)?;

  socket.set_nonblocking(true)?;

  let addr = SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), port);
  socket.bind(&addr.into())?;

  socket.listen(1024)?;

  Ok(socket.into())
}
