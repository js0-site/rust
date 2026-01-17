#![cfg_attr(docsrs, feature(doc_cfg))]

use xstr::cut255 as cut;

pub fn user_host(mail: impl AsRef<str>) -> Option<(String, String)> {
  let mail = mail.as_ref();
  let (user, host) = mail.split_once('@')?;

  if let Some((prefix, suffix)) = host.split_once(".") {
    if prefix.is_empty() || suffix.is_empty() {
      return None;
    }
  } else {
    return None;
  }

  if user.is_empty() {
    None
  } else {
    Some((cut(user).to_owned(), cut(host).into()))
  }
}

fn decode_host(host: &str) -> String {
  host
    .split('.')
    .map(|i| {
      let i = i.trim();
      if i.starts_with("xn--")
        && let Ok(i) = punycode::decode(&i[3..])
      {
        i
      } else {
        i.to_owned()
      }
    })
    .collect::<Vec<_>>()
    .join(".")
}

pub fn norm_user_host(mail: impl AsRef<str>) -> Option<(String, String)> {
  let mail = xstr::lowtrim(mail);
  let (user, host) = user_host(mail)?;
  Some((user, decode_host(&host)))
}

pub fn norm(mail: impl AsRef<str>) -> Option<String> {
  let (user, host) = norm_user_host(mail)?;
  Some(user + "@" + &host)
}

#[cfg(feature = "tld")]
pub fn norm_tld(mail: impl AsRef<str>) -> Option<(String, String)> {
  let (user, host) = norm_user_host(mail)?;
  Some((user + "@" + &host, xtld::tld(&host).into()))
}
