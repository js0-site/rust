use std::{path::Path, time::Instant};

use aok::{OK, Void};
use compio_net::ToSocketAddrsAsync;
use serde::Serialize;

#[cfg(compio_dns)]
extern crate compio_dns;

#[derive(Serialize)]
struct BenchResult {
    first_resolution: u64,
    repeated_resolution: u64,
}

fn random_domain() -> String {
    format!("{}.sslip.io:80", fastrand::u64(..))
}

async fn benchmark_first_resolution() -> f64 {
    let n = 100;
    let domains: Vec<_> = (0..n).map(|_| random_domain()).collect();
    let start = Instant::now();
    for domain in domains {
        let _ = domain.to_socket_addrs_async().await;
    }
    let duration = start.elapsed();
    n as f64 / duration.as_secs_f64()
}

async fn benchmark_repeated_resolution() -> f64 {
    let n = 10000;
    let domain = random_domain();
    // Pre-warm cache (if any) or just resolve once to ensure connectivity?
    // User wants "多次解析", usually implies cached or hot path performance.
    // The previous implementation loop also included the resolution.
    
    let start = Instant::now();
    for _ in 0..n {
        let _ = domain.to_socket_addrs_async().await;
    }
    let duration = start.elapsed();
    n as f64 / duration.as_secs_f64()
}

#[compio::main]
async fn main() -> Void {
    let name = std::env::var("NAME").unwrap_or_else(|_| "unknown".to_string());
    println!("Benchmarking: {}", name);

    // Warmup / Initial check
    let _ = "baidu.com:80".to_socket_addrs_async().await;

    let first_rps = benchmark_first_resolution().await;
    println!("First resolution RPS: {:.2}", first_rps);

    let repeated_rps = benchmark_repeated_resolution().await;
    println!("Repeated resolution RPS: {:.2}", repeated_rps);

    let result = BenchResult {
        first_resolution: first_rps.round() as u64,
        repeated_resolution: repeated_rps.round() as u64,
    };

    let json = sonic_rs::to_string_pretty(&result)?;
    
    let bench_dir = Path::new("../bench");
    if !bench_dir.exists() {
        std::fs::create_dir_all(bench_dir)?;
    }
    
    let output_path = bench_dir.join(format!("{}.json", name));
    std::fs::write(&output_path, json)?;
    println!("Result saved to {:?}", output_path);

    OK
}
