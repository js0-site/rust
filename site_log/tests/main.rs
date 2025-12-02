#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

use std::sync::Arc;

use anyhow::Result;
use aok::{OK, Void};
use site_log::{Arg, Ctx, HookLi, HookSlice, Site, TypeHook};
use tosql::ToSql;

#[derive(ToSql)]
pub struct User {
  pub id: u64,
}

#[linkme::distributed_slice]
pub static USER: HookLi<User>;

impl TypeHook for User {
  const ID: u64 = 1;
  const HOOK: HookSlice<Self> = &USER;
}

#[linkme::distributed_slice(USER)]
pub fn test_abc(user: Arc<Arg<User>>) -> tokio::task::JoinHandle<Result<()>> {
  tokio::spawn(async move {
    println!(">>> USER ID {}", user.id);
    Ok(())
  })
}

#[tokio::test]
async fn test() -> Void {
  let ctx = Ctx::new("127.0.0.1".parse().unwrap(), 1, 1, 1);
  let site = Site { ctx: ctx.into() };
  let user = User { id: 1 };
  site.log(user).await?;
  OK
}
