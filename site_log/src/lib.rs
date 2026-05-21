#![cfg_attr(docsrs, feature(doc_cfg))]

use std::{ops::Deref, sync::Arc};

use anyhow::Result;
use kvid::KvId;
pub use site_log_derive::linkme;
use tokio::task::JoinHandle as TokioJoinHandle;
use tosql::ToSqlTrait;
mod ctx;
pub use ctx::Ctx;
pub use tokio::spawn;

pub static SITE_LOG_ID: KvId = KvId::const_new("siteLog");

pub struct Site {
  pub ctx: Arc<ctx::Ctx>,
}

#[derive(Clone)]
pub struct Arg<T: Sized> {
  pub ctx: Arc<ctx::Ctx>,
  pub inner: Arc<T>,
}

impl<T> Deref for Arg<T> {
  type Target = T;
  fn deref(&self) -> &Self::Target {
    self.inner.as_ref()
  }
}

pub type ArcArg<T> = Arc<Arg<T>>;
pub type JoinHandle = TokioJoinHandle<Result<()>>;
pub type Hook<T> = fn(ArcArg<T>) -> JoinHandle;
pub type HookLi<T> = [Hook<T>];
pub type HookSlice<T> = &'static linkme::DistributedSlice<HookLi<T>>;

pub trait TypeHook
where
  Self: 'static + Sized,
{
  const ID: u64;
  const HOOK: HookSlice<Self>;
}

impl Site {
  pub async fn log<T: Send + Sync + TypeHook + ToSqlTrait>(
    &self,
    row: impl Into<Arc<T>>,
  ) -> Result<()> {
    let row = row.into();
    let ctx = self.ctx.clone();
    let (log_result, task_result) = tokio::join!(
      tokio::spawn({
        let dump = row.dump();
        let ctx = ctx.clone();
        async move {
          use fred::types::streams::XID;
          use xkv::fred::interfaces::StreamsInterface;
          ctx
            .xadd::<(), _, _, _, _>(
              &ctx.site_key[..],
              false,
              None,
              XID::Auto,
              (&ctx.ipbin[..], xbin::concat!(ctx.bin, dump)),
            )
            .await
        }
      }),
      tokio::spawn({
        let arg = Arc::new(Arg { ctx, inner: row });
        async move {
          for i in T::HOOK.iter().map(|f| f(arg.clone())) {
            let _ = i.await?;
          }
          Ok::<_, anyhow::Error>(())
        }
      })
    );
    let _ = task_result?;
    let _ = log_result?;
    Ok(())
  }
}
