extern crate compio_net_extern_resolve;

mod error;
mod os;
mod protocol;
pub mod resolve;
pub mod r#extern;
pub use error::DnsError;

pub(crate) mod cache;


#[static_init::dynamic]
pub(crate) static CACHE: cache::Cache = cache::Cache::new();

use std::{io, net::SocketAddr, vec::IntoIter};

pub use resolve::Resolve;

/// 解析域名为 SocketAddr 迭代器
///
/// 该函数会使用系统默认配置（`/etc/hosts`, `/etc/resolv.conf`）进行解析。
/// 会使用 LRU 缓存
pub async fn resolve_sock_addrs(host: &str, port: u16) -> io::Result<IntoIter<SocketAddr>> {
  let resolver = Resolve::new()?;
  Ok(
    resolver
      .lookup(host)
      .await?
      .map(|mut addr| {
        addr.set_port(port);
        addr
      })
      .collect::<Vec<_>>()
      .into_iter(),
  )
}
