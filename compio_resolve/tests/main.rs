use aok::{OK, Void};
use compio_net::ToSocketAddrsAsync;
use log::info;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

// 必须引入 compio_resolve，确保 resolve_set! 生成的符号被链接
extern crate compio_resolve;

#[compio::test]
async fn test_resolve() -> Void {
  let domain = ("rust-lang.org", 443);

  // 第一次请求（冷启动：解析 resolv.conf + DNS 查询）
  let start = std::time::Instant::now();
  let addrs: Vec<_> = domain.to_socket_addrs_async().await?.collect();
  let first = start.elapsed();
  info!("第一次解析 {:?} -> {:?} 耗时 {:?}", domain, addrs, first);
  assert!(!addrs.is_empty(), "应该至少解析出一个地址");

  // 第二次请求（resolv.conf 已缓存在 OnceCell 中）
  let start = std::time::Instant::now();
  let addrs2: Vec<_> = domain.to_socket_addrs_async().await?.collect();
  let second = start.elapsed();
  info!("第二次解析 {:?} -> {:?} 耗时 {:?}", domain, addrs2, second);
  assert!(!addrs2.is_empty());

  info!("加速比: {:.1}x", first.as_secs_f64() / second.as_secs_f64());

  OK
}
