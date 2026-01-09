use std::{
  borrow::Cow,
  net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

use ip_set::{IpRange, Ipv4Map, Ipv6Map};

use crate::{Answer, QType, Query};

const V4_BITS: u8 = 32;
const V6_BITS: u8 = 128;
const MAX_LOOKUPS: u8 = 16;
const MAX_MX: usize = 10;
const SPF_PREFIX: &str = "v=spf1";

/// SPF check result (RFC 7208) / SPF 检查结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
  Pass,
  Fail,
  SoftFail,
  Neutral,
  None,
  TempError,
  PermError,
}

#[cfg(feature = "cache")]
#[static_init::dynamic(lazy)]
pub static CACHE: crate::Cache<Spf> = crate::Cache::new(600);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Act {
  #[default]
  Pass,
  Fail,
  SoftFail,
  Neutral,
}

#[derive(Debug, Clone)]
pub struct Host {
  pub act: Act,
  pub kind: HostKind,
}

#[derive(Debug, Clone)]
pub enum HostKind {
  A(Option<String>, u8),
  Mx(Option<String>, u8),
  Include(String),
  Exists(String),
  Redirect(String),
}

#[derive(Debug, Clone)]
pub struct Spf {
  pub ip4: Ipv4Map<Act>,
  pub ip6: Ipv6Map<Act>,
  pub all: Option<Act>,
  pub host: Vec<Host>,
  pub ttl: u64,
}

impl Spf {
  pub async fn verify<Q: Query>(q: &Q, domain: &str, ip: IpAddr) -> Status {
    let mut lookups = 0u8;
    let r = Self::check_host(q, domain, ip, &mut lookups).await;

    if !matches!(r, Status::Pass | Status::PermError | Status::TempError) {
      let bits = if ip.is_ipv4() { V4_BITS } else { V6_BITS };
      if let DnsResult::Match = mx_match(q, domain, bits, ip, &mut lookups).await {
        return Status::Pass;
      }
    }
    r
  }

  async fn check_host<Q: Query>(q: &Q, domain: &str, ip: IpAddr, lookups: &mut u8) -> Status {
    if inc_lookup(lookups, domain) {
      return Status::PermError;
    }

    let li = match CACHE.query(q, domain).await {
      Ok(Some(li)) => li,
      Ok(None) => return Status::None,
      Err(e) => {
        log::warn!("SPF {domain}: {e}");
        return Status::TempError;
      }
    };

    if li.len() > 1 {
      log::warn!("SPF {domain}: multiple SPF records");
      return Status::PermError;
    }

    match li.into_iter().next() {
      Some(spf) => spf.eval(q, domain, ip, lookups).await,
      None => Status::None,
    }
  }

  fn check(&self, ip: IpAddr) -> Option<Act> {
    let fallback = if self.host.is_empty() { self.all } else { None };
    match ip {
      IpAddr::V4(addr) => self.ip4.get(addr).copied().or(fallback),
      IpAddr::V6(addr) => self.ip6.get(addr).copied().or(fallback),
    }
  }

  async fn eval<Q: Query>(&self, q: &Q, domain: &str, ip: IpAddr, lookups: &mut u8) -> Status {
    if let Some(act) = self.check(ip) {
      return act.into();
    }

    let mut redirect = None;

    for h in &self.host {
      let result = match &h.kind {
        HostKind::Include(d) => {
          let r = Box::pin(Self::check_host(q, d, ip, lookups)).await;
          match r {
            Status::Pass => Some(h.act),
            Status::None | Status::PermError => return Status::PermError,
            Status::TempError => return Status::TempError,
            _ => None,
          }
        }
        HostKind::Redirect(d) => {
          redirect = Some(d.as_str());
          continue;
        }
        HostKind::A(d, bits) => {
          if inc_lookup(lookups, domain) {
            return Status::PermError;
          }
          let target = d.as_deref().unwrap_or(domain);
          match host_match(q, target, *bits, ip).await {
            DnsResult::Match => Some(h.act),
            DnsResult::Error => return Status::TempError,
            DnsResult::NoMatch => None,
          }
        }
        HostKind::Mx(d, bits) => {
          if inc_lookup(lookups, domain) {
            return Status::PermError;
          }
          let target = d.as_deref().unwrap_or(domain);
          match mx_match(q, target, *bits, ip, lookups).await {
            DnsResult::Match => Some(h.act),
            DnsResult::Error => return Status::PermError,
            DnsResult::NoMatch => None,
          }
        }
        HostKind::Exists(d) => {
          if inc_lookup(lookups, domain) {
            return Status::PermError;
          }
          let target = expand_macro(d, ip, domain);
          host_exists(q, &target).await.then_some(h.act)
        }
      };

      if let Some(act) = result {
        return act.into();
      }
    }

    if let Some(d) = redirect {
      let r = Box::pin(Self::check_host(q, d, ip, lookups)).await;
      return if r == Status::None {
        log::warn!("SPF {domain}: redirect {d} not found");
        Status::PermError
      } else {
        r
      };
    }

    self.all.map_or(Status::Neutral, Into::into)
  }
}

enum DnsResult {
  Match,
  NoMatch,
  Error,
}

/// Increment lookup counter, return true if limit exceeded
/// 增加查询计数，超限返回 true
fn inc_lookup(lookups: &mut u8, domain: &str) -> bool {
  *lookups += 1;
  if *lookups > MAX_LOOKUPS {
    log::warn!("SPF {domain}: DNS lookup limit exceeded");
    true
  } else {
    false
  }
}

async fn host_match<Q: Query>(q: &Q, host: &str, bits: u8, ip: IpAddr) -> DnsResult {
  let matched = match ip {
    IpAddr::V4(target) => super::a::CACHE
      .query(q, host)
      .await
      .map(|li| li.is_some_and(|v| v.iter().any(|a| Ipv4Addr::in_cidr(a.ip, bits, target)))),
    IpAddr::V6(target) => super::aaaa::CACHE
      .query(q, host)
      .await
      .map(|li| li.is_some_and(|v| v.iter().any(|a| Ipv6Addr::in_cidr(a.ip, bits, target)))),
  };
  match matched {
    Ok(true) => DnsResult::Match,
    Ok(false) => DnsResult::NoMatch,
    Err(_) => DnsResult::Error,
  }
}

async fn mx_match<Q: Query>(
  q: &Q,
  domain: &str,
  bits: u8,
  ip: IpAddr,
  lookups: &mut u8,
) -> DnsResult {
  let li = match super::mx::CACHE.query(q, domain).await {
    Ok(Some(li)) => li,
    Ok(None) => return DnsResult::NoMatch,
    Err(_) => return DnsResult::Error,
  };

  for mx in li.iter().take(MAX_MX) {
    if inc_lookup(lookups, domain) {
      return DnsResult::Error;
    }
    match host_match(q, &mx.server, bits, ip).await {
      DnsResult::Match => return DnsResult::Match,
      DnsResult::Error => return DnsResult::Error,
      DnsResult::NoMatch => {}
    }
  }
  DnsResult::NoMatch
}

async fn host_exists<Q: Query>(q: &Q, host: &str) -> bool {
  matches!(super::a::CACHE.query(q, host).await, Ok(Some(li)) if !li.is_empty())
}

fn expand_macro<'a>(s: &'a str, ip: IpAddr, domain: &str) -> Cow<'a, str> {
  if !s.contains('%') {
    return Cow::Borrowed(s);
  }

  let mut out = String::with_capacity(s.len() * 2);
  let mut chars = s.chars().peekable();

  while let Some(c) = chars.next() {
    if c != '%' {
      out.push(c);
      continue;
    }

    match chars.next() {
      Some('%') => out.push('%'),
      Some('_') => out.push(' '),
      Some('-') => out.push_str("%20"),
      Some('{') => {
        let mut letter = None;
        let mut reverse = false;

        while let Some(&ch) = chars.peek() {
          if ch == '}' {
            chars.next();
            break;
          }
          chars.next();
          match ch {
            'i' | 'I' => letter = Some('i'),
            'd' | 'D' => letter = Some('d'),
            'r' | 'R' => reverse = true,
            _ => {}
          }
        }

        match letter {
          Some('i') if reverse => out.push_str(&ip_reverse(ip)),
          Some('i') => out.push_str(&ip.to_string()),
          Some('d') => out.push_str(domain),
          _ => {}
        }
      }
      Some(c) => {
        out.push('%');
        out.push(c);
      }
      None => out.push('%'),
    }
  }
  Cow::Owned(out)
}

fn ip_reverse(ip: IpAddr) -> String {
  match ip {
    IpAddr::V4(v4) => {
      let [a, b, c, d] = v4.octets();
      format!("{d}.{c}.{b}.{a}")
    }
    IpAddr::V6(v6) => {
      // Low nibble first, then high nibble / 低位在前，高位在后
      const HEX: &[u8; 16] = b"0123456789abcdef";
      let mut out = String::with_capacity(63);
      for (i, b) in v6.octets().iter().rev().enumerate() {
        if i > 0 {
          out.push('.');
        }
        out.push(HEX[(b & 0xf) as usize] as char);
        out.push('.');
        out.push(HEX[(b >> 4) as usize] as char);
      }
      out
    }
  }
}

impl From<Act> for Status {
  fn from(act: Act) -> Self {
    match act {
      Act::Pass => Status::Pass,
      Act::Fail => Status::Fail,
      Act::SoftFail => Status::SoftFail,
      Act::Neutral => Status::Neutral,
    }
  }
}

fn parse_act(s: &str) -> (Act, &str) {
  match s.as_bytes().first() {
    Some(b'+') => (Act::Pass, &s[1..]),
    Some(b'-') => (Act::Fail, &s[1..]),
    Some(b'~') => (Act::SoftFail, &s[1..]),
    Some(b'?') => (Act::Neutral, &s[1..]),
    _ => (Act::Pass, s),
  }
}

fn parse_prefix(s: &str, default: u8) -> u8 {
  s.parse().unwrap_or(default)
}

fn parse_ip<T: std::str::FromStr>(s: &str, default: u8) -> Option<(T, u8)> {
  let (addr, bits) = s
    .split_once('/')
    .map_or((s, default), |(a, p)| (a, parse_prefix(p, default)));
  addr.parse().ok().map(|a| (a, bits))
}

fn parse_domain_bits(s: &str) -> (Option<String>, u8) {
  if s.is_empty() {
    return (None, V4_BITS);
  }
  if let Some(p) = s.strip_prefix('/') {
    return (None, parse_prefix(p, V4_BITS));
  }
  let rest = s.strip_prefix(':').unwrap_or(s);
  match rest.split_once('/') {
    Some((domain, p)) => (Some(domain.into()), parse_prefix(p, V4_BITS)),
    None => (Some(rest.into()), V4_BITS),
  }
}

fn strip_mech<'a>(s: &'a str, mech: &str) -> Option<&'a str> {
  if s == mech {
    return Some("");
  }
  s.strip_prefix(mech)
    .filter(|r| r.starts_with(':') || r.starts_with('/'))
}

impl super::Parse for Spf {
  const QTYPE: QType = QType::TXT;

  fn li(answers: impl IntoIterator<Item = Answer>) -> Vec<Self> {
    answers
      .into_iter()
      .filter(|a| a.type_id == QType::TXT as u16)
      .filter_map(|a| parse_spf(&a.val, a.ttl.into()))
      .collect()
  }
}

fn parse_spf(raw: &str, ttl: u64) -> Option<Spf> {
  let val = raw.trim().trim_matches('"');
  if !val.get(..6).is_some_and(|s| s.eq_ignore_ascii_case(SPF_PREFIX)) {
    return None;
  }

  let mut spf = Spf {
    ip4: Ipv4Map::new(),
    ip6: Ipv6Map::new(),
    all: None,
    host: Vec::new(),
    ttl,
  };

  for part in val.split_whitespace().skip(1) {
    let lower = part.to_ascii_lowercase();
    let part = lower.as_str();

    if let Some(domain) = part.strip_prefix("redirect=") {
      spf.host.push(Host {
        act: Act::Pass,
        kind: HostKind::Redirect(domain.into()),
      });
      continue;
    }

    if part.starts_with("exp=") {
      continue;
    }

    let (act, rest) = parse_act(part);

    if rest == "all" {
      spf.all = Some(act);
      continue;
    }

    if let Some(s) = rest.strip_prefix("ip4:") {
      if let Some((addr, bits)) = parse_ip::<Ipv4Addr>(s, V4_BITS) {
        spf.ip4.add_cidr(addr, bits, act);
      }
      continue;
    }

    if let Some(s) = rest.strip_prefix("ip6:") {
      if let Some((addr, bits)) = parse_ip::<Ipv6Addr>(s, V6_BITS) {
        spf.ip6.add_cidr(addr, bits, act);
      }
      continue;
    }

    if let Some(domain) = rest.strip_prefix("include:") {
      spf.host.push(Host {
        act,
        kind: HostKind::Include(domain.into()),
      });
      continue;
    }

    if let Some(domain) = rest.strip_prefix("exists:") {
      spf.host.push(Host {
        act,
        kind: HostKind::Exists(domain.into()),
      });
      continue;
    }

    if let Some(s) = strip_mech(rest, "a") {
      let (domain, bits) = parse_domain_bits(s);
      spf.host.push(Host {
        act,
        kind: HostKind::A(domain, bits),
      });
      continue;
    }

    if let Some(s) = strip_mech(rest, "mx") {
      let (domain, bits) = parse_domain_bits(s);
      spf.host.push(Host {
        act,
        kind: HostKind::Mx(domain, bits),
      });
    }
  }

  Some(spf)
}
