use crate::{Answer, QType};

#[cfg(feature = "cache")]
#[static_init::dynamic(lazy)]
pub static CACHE: crate::Cache<Mx> = crate::Cache::new(600);

#[derive(Debug, Clone)]
pub struct Mx {
  /// Priority of the mail server (lower values have higher priority)
  pub priority: u16,
  /// Mail server hostname
  pub server: String,
  /// Time to live in seconds
  pub ttl: u64,
}

impl super::Parse for Mx {
  const QTYPE: QType = QType::MX;

  fn li(answers: impl IntoIterator<Item = Answer>) -> Vec<Self> {
    let mut r: Vec<Mx> = answers
      .into_iter()
      .filter(|a| a.type_id == Self::QTYPE as u16)
      .filter_map(|a| {
        // Format: "priority server" e.g. "10 mail.example.com."
        a.val.split_once(' ').and_then(|(p, s)| {
          p.parse().ok().map(|priority| Mx {
            priority,
            server: s.trim_end_matches('.').into(),
            ttl: a.ttl.into(),
          })
        })
      })
      .collect();
    r.sort_unstable_by_key(|i| i.priority);
    r
  }
}
