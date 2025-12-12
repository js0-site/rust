use crate::{Resolver, Result, record_type};

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
  ) -> impl std::future::Future<Output = Result<Option<Self::VecMx<'a>>>> + Send + 'a;
}

impl<T: Resolver + Sync> MxLookup for T {
  type VecMx<'a>
    = Vec<Mx>
  where
    Self: 'a;
  async fn mx<'a>(
    &'a self,
    domain: impl AsRef<str> + Send + 'a,
  ) -> Result<Option<Self::VecMx<'a>>> {
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
        r.sort_unstable_by_key(|i| i.priority);
        Ok(Some(r))
      })
      .await
  }
}

#[cfg(feature = "cache")]
pub mod cache {

  use dashmap::DashMap;
  use expire_cache::{AsyncInit, Expire};

  use super::Mx;
  use crate::{MxLookup, Resolve, Result};

  pub struct MxRef<'a>(expire_cache::map::RefVal<'a, String, Option<Vec<Mx>>>);

  impl<'a> std::ops::Deref for MxRef<'a> {
    type Target = [Mx];
    fn deref(&self) -> &Self::Target {
      match &*self.0 {
        None => &[],
        Some(li) => li,
      }
    }
  }

  #[static_init::dynamic(lazy)]
  pub static CACHE: Expire<DashMap<String, Option<Vec<Mx>>>> = Expire::new(600);

  pub struct Cache;

  #[derive(Default)]
  struct MxAsyncInit;

  impl AsyncInit for MxAsyncInit {
    type Error = crate::Error;
    type Key = String;
    type Val = Option<Vec<Mx>>;

    async fn init(key: &Self::Key) -> Result<Self::Val> {
      Resolve.mx(key).await
    }
  }

  impl super::MxLookup for Cache {
    type VecMx<'a> = MxRef<'a>;
    async fn mx<'a>(&'a self, domain: impl AsRef<str> + Send + 'a) -> Result<Option<MxRef<'a>>> {
      let domain = domain.as_ref();
      let r = CACHE
        .get_or_init_async::<MxAsyncInit>(domain.to_owned())
        .await?;
      Ok(Some(MxRef(r)))
    }
  }
}
