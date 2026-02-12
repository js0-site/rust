use std::{
    collections::HashMap,
    io,
    net::{IpAddr, SocketAddr},
};

use once_cell::sync::OnceCell;

use crate::config::ResolvConf;

mod files;
mod query;

static RESOLV_CONF: OnceCell<ResolvConf> = OnceCell::new();
static HOSTS: OnceCell<HashMap<String, IpAddr>> = OnceCell::new();

#[cfg(feature = "cache")]
static DNS_CACHE: OnceCell<crate::cache::DnsCache> = OnceCell::new();

pub struct Resolve {
    resolv_conf: &'static ResolvConf,
}

impl Resolve {
    pub fn new() -> io::Result<Self> {
        let resolv_conf = RESOLV_CONF.get_or_try_init(ResolvConf::load)?;
        HOSTS.get_or_try_init(files::load_hosts)?;

        #[cfg(feature = "cache")]
        DNS_CACHE.get_or_init(crate::cache::DnsCache::new);

        Ok(Self { resolv_conf })
    }

    pub async fn lookup(&self, name: &str) -> io::Result<std::vec::IntoIter<SocketAddr>> {
        // /etc/hosts
        if let Some(hosts) = HOSTS.get() {
            if let Some(addr) = hosts.get(&name.to_lowercase()) {
                return Ok(vec![SocketAddr::new(*addr, 0)].into_iter());
            }
        }

        // IP literal
        if let Ok(ip) = name.parse::<IpAddr>() {
            return Ok(vec![SocketAddr::new(ip, 0)].into_iter());
        }

        #[cfg(feature = "cache")]
        if let Some(cache) = DNS_CACHE.get() {
            if let Some(addrs) = cache.get(name).await {
                return Ok(addrs
                    .into_iter()
                    .map(|ip| SocketAddr::new(ip, 0))
                    .collect::<Vec<_>>()
                    .into_iter());
            }
        }

        for name in self.build_search_list(name) {
            if let Ok(result) = query::query(self.resolv_conf, &name).await {
                if !result.addrs.is_empty() {
                    #[cfg(feature = "cache")]
                    if let Some(cache) = DNS_CACHE.get() {
                        cache
                            .insert(name.clone(), result.addrs.clone(), result.ttl)
                            .await;
                    }

                    return Ok(result
                        .addrs
                        .into_iter()
                        .map(|ip| SocketAddr::new(ip, 0))
                        .collect::<Vec<_>>()
                        .into_iter());
                }
            }
        }

        Err(io::Error::other("failed to resolve"))
    }

    fn build_search_list(&self, name: &str) -> Vec<String> {
        let mut names = Vec::new();
        if name.ends_with('.') {
            names.push(name.trim_end_matches('.').to_string());
            return names;
        }

        let ndots = name.bytes().filter(|&b| b == b'.').count();
        if ndots >= self.resolv_conf.ndots as usize {
            names.push(name.to_string());
        }
        for domain in &self.resolv_conf.search {
            names.push(format!("{name}.{domain}"));
        }
        if ndots < self.resolv_conf.ndots as usize {
            names.push(name.to_string());
        }
        names
    }
}
