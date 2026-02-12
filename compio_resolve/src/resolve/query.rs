use std::{
    io,
    net::{IpAddr, SocketAddr},
};

use compio_buf::{bytes::BufMut, BufResult};
use compio_io::{AsyncReadExt, AsyncWriteExt};

use crate::{
    config::ResolvConf,
    protocol::{Message, QueryType, ResponseCode, write_query, write_question},
};

pub(crate) struct QueryResult {
    pub addrs: Vec<IpAddr>,
    pub ttl: u32,
}

impl QueryResult {
    pub fn empty() -> Self {
        Self {
            addrs: vec![],
            ttl: 0,
        }
    }

    pub fn from_answers(answers: &[crate::protocol::Record<'_>]) -> Self {
        let mut addrs = Vec::new();
        let mut min_ttl = u32::MAX;
        for r in answers {
            if let Some(ip) = r.to_ip() {
                addrs.push(ip);
                if r.ttl() < min_ttl {
                    min_ttl = r.ttl();
                }
            }
        }
        if min_ttl == u32::MAX {
            min_ttl = 0;
        }
        Self {
            addrs,
            ttl: min_ttl,
        }
    }
}

pub(crate) async fn query(
    resolv_conf: &ResolvConf,
    name: &str,
) -> io::Result<QueryResult> {
    let futures: Vec<_> = resolv_conf
        .nameservers
        .iter()
        .map(|ns| Box::pin(query_ns_all(resolv_conf, name, *ns)))
        .collect();

    if futures.is_empty() {
        return Ok(QueryResult::empty());
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

    Ok(QueryResult::empty())
}

async fn query_ns_all(
    resolv_conf: &ResolvConf,
    name: &str,
    ns: SocketAddr,
) -> io::Result<QueryResult> {
    let r = query_ns(resolv_conf, name, ns, QueryType::A).await?;
    if !r.addrs.is_empty() {
        return Ok(r);
    }
    query_ns(resolv_conf, name, ns, QueryType::Aaaa).await
}

async fn query_ns(
    resolv_conf: &ResolvConf,
    name: &str,
    ns: SocketAddr,
    qtype: QueryType,
) -> io::Result<QueryResult> {
    let id = fastrand::u16(..);
    let mut buf = Vec::with_capacity(512);
    write_query(id, &mut buf);
    write_question(name, qtype, &mut buf)?;

    let socket = compio_net::UdpSocket::bind(&SocketAddr::from(([0, 0, 0, 0], 0))).await?;
    socket.connect(ns).await?;

    let mut recv_buf = Vec::with_capacity(512);
    for _ in 0..resolv_conf.attempts {
        let BufResult(res, b) = socket.send(buf).await;
        buf = b;
        res?;

        let BufResult(res, b) =
            compio_runtime::time::timeout(resolv_conf.timeout, socket.recv(recv_buf))
                .await
                .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "recv timed out"))?;
        recv_buf = b;
        let n = res?;

        if n < 12 {
            continue;
        }

        if let Ok(msg) = Message::read(&recv_buf[..n])
            && msg.header.id.get() == id
            && msg.header.is_response()
        {
            if msg.header.truncated() {
                return query_ns_tcp(resolv_conf, name, ns, qtype).await;
            }
            match msg.header.response_code() {
                ResponseCode::NoError => return Ok(QueryResult::from_answers(&msg.answers)),
                ResponseCode::NameError => return Ok(QueryResult::empty()),
                _ => {}
            }
        }
    }

    Err(io::Error::new(io::ErrorKind::TimedOut, "query timed out"))
}

async fn query_ns_tcp(
    resolv_conf: &ResolvConf,
    name: &str,
    ns: SocketAddr,
    qtype: QueryType,
) -> io::Result<QueryResult> {
    let timeout = resolv_conf.timeout;

    let id = fastrand::u16(..);
    let mut buf = Vec::with_capacity(514);
    buf.put_u16(0); // length placeholder
    write_query(id, &mut buf);
    write_question(name, qtype, &mut buf)?;
    let len = (buf.len() - 2) as u16;
    buf[0..2].copy_from_slice(&len.to_be_bytes());

    let mut socket = compio_runtime::time::timeout(timeout, compio_net::TcpStream::connect(ns))
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "TCP connect timed out"))?
        ?;

    let BufResult(res, _) = compio_runtime::time::timeout(timeout, socket.write_all(buf))
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "TCP write timed out"))?;
    res?;

    let BufResult(res, len_buf) =
        compio_runtime::time::timeout(timeout, socket.read_exact([0u8; 2]))
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "TCP read timed out"))?;
    res?;
    let resp_len = u16::from_be_bytes(len_buf) as usize;

    let BufResult(res, recv_buf) =
        compio_runtime::time::timeout(timeout, socket.read_exact(vec![0u8; resp_len]))
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "TCP read timed out"))?;
    res?;

    let msg = Message::read(&recv_buf)?;
    if msg.header.id.get() != id {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "ID mismatch"));
    }

    match msg.header.response_code() {
        ResponseCode::NoError => Ok(QueryResult::from_answers(&msg.answers)),
        ResponseCode::NameError => Ok(QueryResult::empty()),
        _ => Err(io::Error::other("DNS server returned error")),
    }
}
