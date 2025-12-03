use aok::Result;

use crate::{Resolver, record_type};

/// MX record structure
#[derive(Debug, Clone)]
pub struct Mx {
  /// Priority of the mail server (lower values have higher priority)
  pub priority: u16,
  /// Mail server hostname
  pub server: String,
  /// Time to live in seconds
  pub ttl: u64,
}

pub trait MxLookup {
  type VecMx<'a>: std::ops::Deref<Target = [Mx]> + 'a
  where
    Self: 'a;
  fn mx<'a>(
    &'a self,
    domain: impl AsRef<str> + Send + 'a,
  ) -> impl std::future::Future<Output = Result<Self::VecMx<'a>>> + Send + 'a;
}

impl<T: Resolver + Sync> MxLookup for T {
  type VecMx<'a>
    = Vec<Mx>
  where
    Self: 'a;
  async fn mx<'a>(&'a self, domain: impl AsRef<str> + Send + 'a) -> Result<Self::VecMx<'a>> {
    self
      .resolve(domain, "MX", |li| {
        let mut r = Vec::with_capacity(li.len());
        for i in li {
          if i.r#type == record_type::MX {
            // MX data format: "priority server"
            // Example: "10 mail.example.com."
            if let Some((priority_str, server)) = i.data.split_once(' ')
              && let Ok(priority) = priority_str.parse::<u16>()
            {
              r.push(Mx {
                priority,
                server: server.trim_end_matches('.').to_string(),
                ttl: i.ttl,
              });
            }
          }
        }
        Ok(Some(r))
      })
      .await
  }
}

#[cfg(feature = "cache")]
pub mod cache {
  use std::ops::Deref;

  use aok::Result;
  use dashmap::DashMap;
  use expire_cache::Expire;

  use super::Mx;
  use crate::Resolve;

  #[static_init::dynamic]
  pub static CACHE: Expire<DashMap<String, Vec<Mx>>> = Expire::new(600);

  pub struct Cache;

  /// Wrapper around RefVal that derefs to [Mx] instead of Vec<Mx>
  pub struct MxRef<'a> {
    inner: expire_cache::map::RefVal<'a, String, Vec<Mx>>,
  }

  impl<'a> Deref for MxRef<'a> {
    type Target = [Mx];
    fn deref(&self) -> &Self::Target {
      &self.inner
    }
  }

  impl super::MxLookup for Cache {
    type VecMx<'a> = MxRef<'a>;
    async fn mx<'a>(&'a self, domain: impl AsRef<str> + Send + 'a) -> Result<Self::VecMx<'a>> {
      let domain = domain.as_ref();
      let domain_key = domain.to_string();
      if let Some(result) = CACHE.get(&domain_key) {
        return Ok(MxRef { inner: result });
      }
      let result = match Resolve.mx(domain).await {
        Ok(r) => r,
        Err(err) => {
          log::warn!("{domain} MX: {err}");
          Default::default()
        }
      };
      loop {
        CACHE.insert(domain_key.clone(), result.to_vec());
        if let Some(r) = CACHE.get(&domain_key).map(|inner| MxRef { inner }) {
          return Ok(r);
        }
      }
    }
  }
}
