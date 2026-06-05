use std::{io, net::SocketAddr, sync::Arc, time::Duration};

mod dns_server;
use aok::{OK, Void};
use dns_server::{DNS_SERVER_LI, DnsServer};
use futures::StreamExt;
use hickory_resolver::{
  Resolver,
  config::{NameServerConfig, ProtocolConfig, ResolverConfig},
  name_server::TokioConnectionProvider,
};
use pick_fast::PickFast;
use race::Race;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

// Create resolver with specific DNS server / 使用指定 DNS 服务器创建解析器
fn create_resolver(server: &DnsServer) -> Resolver<TokioConnectionProvider> {
  let ns = NameServerConfig::new(SocketAddr::new(server.ip, 53), ProtocolConfig::Udp);
  let mut config = ResolverConfig::default();
  config.add_name_server(ns);

  let provider = TokioConnectionProvider::default();
  Resolver::builder_with_config(config, provider).build()
}

/// Task struct for tracking DNS resolution / DNS 解析任务结构体
struct Task {
  pub index: usize,
  pub start: u64,
}

#[tokio::test]
async fn test_iter_with_race_dns() -> Void {
  let lb = Arc::new(PickFast::<DnsServer, pick_fast::Inverse>::new(
    DNS_SERVER_LI,
  ));
  const HOST: &str = "qq.com";

  println!("Testing iter() with race crate for real DNS resolution...");

  // Create Race with Task struct / 使用 Task 结构体创建 Race
  let mut race = Race::new(
    Duration::from_millis(500),
    |task: &Task| {
      let index = task.index;
      let start = task.start;
      let lb = lb.clone();
      let resolver = create_resolver(&lb.li[index].data);
      let server_ip = lb.li[index].data.ip;

      async move {
        match resolver.lookup_ip(HOST).await {
          Ok(response) => {
            if let Some(addr) = response.iter().next() {
              let latency = (ts_::milli() - start) as u32;
              // Successful: update latency weight / 成功：更新延时权重
              lb.set(index, latency);
              println!("  ✅ {HOST} via {server_ip} -> {addr} ({latency}ms)");
              Ok(addr)
            } else {
              lb.failed(index);
              let error = io::Error::new(
                io::ErrorKind::NotFound,
                format!("No address found for {HOST}"),
              );
              println!("  ❌ {HOST} via {server_ip} failed: {error}");
              Err(error)
            }
          }
          Err(e) => {
            lb.failed(index);
            let error = io::Error::other(e.to_string());
            println!("  ❌ {HOST} via {server_ip} failed: {e}");
            Err(error)
          }
        }
      }
    },
    lb.iter().map(|i| Task {
      index: i.0,
      start: ts_::milli(),
    }),
  );

  println!("Starting staggered DNS resolution with 500ms intervals...");

  let mut resolved_ip = None;
  while let Some((_task, result)) = race.next().await {
    match result {
      Ok(addr) => {
        println!("🎯 Resolved: {addr}");
        resolved_ip = Some(addr);

        // Mark remaining tasks as failed / 将剩余任务标记为失败
        for (t, _) in race.ing {
          lb.failed(t.index);
        }
        break;
      }
      Err(_) => {
        // Already called lb.failed(index) in the async closure
        // 已在异步闭包中调用 lb.failed(index)
      }
    }
  }

  if resolved_ip.is_some() {
    println!("✅ Race-based DNS resolution completed successfully");
  } else {
    println!("❌ All DNS resolutions failed (network issue?)");
  }

  OK
}
