#![allow(non_upper_case_globals)]
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AbcEfg {
  pub id: u64,
}

pub type JoinHandle = site_log::JoinHandle;
pub type Task<T> = fn(site_log::ArcArg<T>) -> JoinHandle;

#[linkme::distributed_slice]
pub static AbcEfg: [Task<AbcEfg>] = [..];

// Use macro - infer type from param name | 使用宏 - 从参数名推断类型
site_log_derive::linkme!(|abc_efg| {
  println!("Processing AbcEfg with id: {}", abc_efg.inner.id);
});

#[tokio::main]
async fn main() {
  println!("Number of registered tasks: {}", AbcEfg.len());

  let inner = Arc::new(AbcEfg { id: 123 });
  let ctx = Arc::new(site_log::Ctx::new(
    std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
    0,
    0,
    0,
  ));

  for task in AbcEfg {
    let arg = Arc::new(site_log::Arg {
      ctx: ctx.clone(),
      inner: inner.clone(),
    });
    let handle = task(arg);
    match handle.await {
      Ok(Ok(())) => println!("Task completed successfully"),
      Ok(Err(e)) => eprintln!("Task failed: {:?}", e),
      Err(e) => eprintln!("Join error: {:?}", e),
    }
  }
}
