use std::{
  borrow::Cow,
  io,
  net::{IpAddr, SocketAddr},
  vec::IntoIter,
};

use compio_buf::{BufResult, bytes::BufMut};
use compio_io::{AsyncReadExt, AsyncWriteExt};
use compio_net::{TcpStream, UdpSocket};
use compio_runtime::time::timeout;

use super::{
  CACHE, DnsError,
  os::{DNS, Dns, HOST_IP},
  protocol::{Message, QueryType, ResponseCode, write_query, write_question},
};

/// 超时错误辅助函数
#[cold]
fn timeout_err(msg: &str) -> io::Error {
  io::Error::new(io::ErrorKind::TimedOut, msg)
}

/// DNS 解析器，支持 `/etc/hosts`、UDP/TCP 查询及缓存
pub struct Resolve {
  dns: &'static Dns,
}

impl Resolve {
  pub fn new() -> io::Result<Self> {
    Ok(Self { dns: &DNS })
  }

  pub async fn lookup(&self, name: &str) -> io::Result<IntoIter<SocketAddr>> {
    // /etc/hosts
    if let Some(addr) = HOST_IP.get(&name.to_lowercase()) {
      return Ok(vec![SocketAddr::new(*addr, 0)].into_iter());
    }

    // IP 字面量
    if let Ok(ip) = name.parse::<IpAddr>() {
      return Ok(vec![SocketAddr::new(ip, 0)].into_iter());
    }

    for name in self.search_list(name) {
      // 优先查缓存
      if let Some(addrs) = CACHE.get(&name).await {
        return Ok(
          addrs
            .into_iter()
            .map(|ip| SocketAddr::new(ip, 0))
            .collect::<Vec<_>>()
            .into_iter(),
        );
      }

      if let Ok(result) = self.query(&name).await {
        {
          let ttl = if result.addrs.is_empty() {
            60
          } else {
            result.min_ttl
          };
          CACHE
            .insert(name.into_owned(), result.addrs.clone(), ttl)
            .await;
        }

        if !result.addrs.is_empty() {
          return Ok(
            result
              .addrs
              .into_iter()
              .map(|ip| SocketAddr::new(ip, 0))
              .collect::<Vec<_>>()
              .into_iter(),
          );
        }
      }
    }

    Err(DnsError::ResolutionFailed.into())
  }

  /// 构建搜索域名列表
  fn search_list<'a>(&'a self, name: &'a str) -> impl Iterator<Item = Cow<'a, str>> {
    let mut names = Vec::with_capacity(1 + self.dns.search.len());

    if name.ends_with('.') {
      names.push(Cow::Borrowed(name.trim_end_matches('.')));
      return names.into_iter();
    }

    let ndots = memchr::memchr_iter(b'.', name.as_bytes()).count();
    if ndots >= self.dns.ndots as usize {
      names.push(Cow::Borrowed(name));
    }
    for domain in &self.dns.search {
      names.push(Cow::Owned(format!("{name}.{domain}")));
    }
    if ndots < self.dns.ndots as usize {
      names.push(Cow::Borrowed(name));
    }
    names.into_iter()
  }

  /// 向所有 nameserver 并发查询，返回第一个成功结果
  async fn query(&self, name: &str) -> io::Result<QueryResult> {
    let futures: Vec<_> = self
      .dns
      .nameservers
      .iter()
      .map(|ns| Box::pin(self.query_ns_all(name, *ns)))
      .collect();

    if futures.is_empty() {
      return Ok(QueryResult::EMPTY);
    }

    use futures_util::future::select_all;

    let mut remaining = futures;
    while !remaining.is_empty() {
      let (result, _, rest) = select_all(remaining).await;
      remaining = rest;
      if let Ok(r) = result
        && !r.addrs.is_empty()
      {
        return Ok(r);
      }
    }

    Ok(QueryResult::EMPTY)
  }

  /// 先查 A 记录，无结果再查 AAAA
  async fn query_ns_all(&self, name: &str, ns: SocketAddr) -> io::Result<QueryResult> {
    let r = self.query_ns(name, ns, QueryType::A).await?;
    if !r.addrs.is_empty() {
      return Ok(r);
    }
    self.query_ns(name, ns, QueryType::Aaaa).await
  }

  /// UDP 查询单个 nameserver
  async fn query_ns(
    &self,
    name: &str,
    ns: SocketAddr,
    qtype: QueryType,
  ) -> io::Result<QueryResult> {
    let id = fastrand::u16(..);
    const UDP_BUFFER_SIZE: usize = 512;
    let mut buf = Vec::with_capacity(UDP_BUFFER_SIZE);
    write_query(id, &mut buf);
    write_question(name, qtype, &mut buf)?;

    let socket = UdpSocket::bind(&SocketAddr::from(([0, 0, 0, 0], 0))).await?;
    socket.connect(ns).await?;

    let recv_timeout = self.dns.timeout;
    // 预分配接收缓冲区
    let mut recv_buf = Vec::with_capacity(UDP_BUFFER_SIZE);

    for _ in 0..self.dns.attempts {
      let BufResult(res, b) = socket.send(buf).await;
      buf = b;
      if res.is_err() {
        continue;
      }

      // 重置接收缓冲区大小（如果需要），Vec::with_capacity 创建的 len 是 0
      // socket.recv 需要 buffer 有 capacity。
      recv_buf.clear();

      let result = timeout(recv_timeout, socket.recv(recv_buf)).await;

      match result {
        Ok(BufResult(Ok(n), b)) => {
          recv_buf = b;
          if n < 12 {
            continue;
          }

          if let Ok(msg) = Message::read(&recv_buf[..n])
            && msg.header.id.get() == id
            && msg.header.is_response()
          {
            if msg.header.truncated() {
              return self.query_tcp(name, ns, qtype).await;
            }
            match msg.header.response_code() {
              ResponseCode::NoError => return Ok(QueryResult::from_answers(&msg.answers)),
              ResponseCode::NameError => return Ok(QueryResult::EMPTY),
              _ => {}
            }
          }
        }
        Ok(BufResult(Err(_), b)) => {
          recv_buf = b;
          continue;
        }
        Err(_b) => {
          recv_buf = Vec::with_capacity(UDP_BUFFER_SIZE);
          continue;
        }
      };
    }

    Err(timeout_err("查询超时"))
  }

  /// TCP 回退查询（UDP 被截断时使用）
  async fn query_tcp(
    &self,
    name: &str,
    ns: SocketAddr,
    qtype: QueryType,
  ) -> io::Result<QueryResult> {
    let tcp_timeout = self.dns.timeout;

    let id = fastrand::u16(..);
    let mut buf = Vec::with_capacity(514);
    buf.put_u16(0); // 长度占位
    write_query(id, &mut buf);
    write_question(name, qtype, &mut buf)?;
    let len = (buf.len() - 2) as u16;
    buf[0..2].copy_from_slice(&len.to_be_bytes());

    let mut socket = timeout(tcp_timeout, TcpStream::connect(ns))
      .await
      .map_err(|_| timeout_err("TCP 连接超时"))??;

    let BufResult(res, _) = timeout(tcp_timeout, socket.write_all(buf))
      .await
      .map_err(|_| timeout_err("TCP 写入超时"))?;
    res?;

    let BufResult(res, len_buf) = timeout(tcp_timeout, socket.read_exact([0u8; 2]))
      .await
      .map_err(|_| timeout_err("TCP 读取超时"))?;
    res?;
    let resp_len = u16::from_be_bytes(len_buf) as usize;

    let BufResult(res, recv_buf) = timeout(tcp_timeout, socket.read_exact(vec![0u8; resp_len]))
      .await
      .map_err(|_| timeout_err("TCP 读取超时"))?;
    res?;

    let msg = Message::read(&recv_buf)?;
    if msg.header.id.get() != id {
      return Err(DnsError::InvalidData.into());
    }

    match msg.header.response_code() {
      ResponseCode::NoError => Ok(QueryResult::from_answers(&msg.answers)),
      ResponseCode::NameError => Ok(QueryResult::EMPTY),
      _ => Err(DnsError::ServerResponseError.into()),
    }
  }
}

/// 查询结果：解析到的 IP 和最小 TTL
struct QueryResult {
  addrs: Vec<IpAddr>,
  min_ttl: u32,
}

impl QueryResult {
  const EMPTY: Self = Self {
    addrs: vec![],
    min_ttl: 0,
  };

  fn from_answers(answers: &[super::protocol::Record<'_>]) -> Self {
    let (addrs, min_ttl) =
      answers
        .iter()
        .fold((Vec::new(), u32::MAX), |(mut addrs, mut min_ttl), r| {
          if let Some(ip) = r.to_ip() {
            addrs.push(ip);
            min_ttl = min_ttl.min(r.ttl);
          }
          (addrs, min_ttl)
        });
    if addrs.is_empty() {
      Self::EMPTY
    } else {
      Self { addrs, min_ttl }
    }
  }
}
