use std::{net::SocketAddr, time::Instant};

use compio::net::ToSocketAddrsAsync;

#[compio::main]
async fn main() {
    // 强制链接 compio_resolve
    // 在实际项目中，这通常由库的使用者通过导入或依赖来完成
    // 在这个 example 中，我们需要确保链接器包含了 compio_resolve 的符号
    #[allow(unused_imports)]
    use compio_resolve::CompioResolver;

    let domain = "google.com";
    let port = 80;

    println!("开始解析域名: {}:{}", domain, port);

    // 第一次解析（冷启动/无缓存）
    let start = Instant::now();
    let addrs: Vec<SocketAddr> = (domain, port).to_socket_addrs_async().await.unwrap().collect();
    let elapsed1 = start.elapsed();
    println!("第一次解析耗时: {:?} (结果: {:?})", elapsed1, addrs);

    // 第二次解析（应该命中缓存）
    let start = Instant::now();
    let addrs_cached: Vec<SocketAddr> = (domain, port).to_socket_addrs_async().await.unwrap().collect();
    let elapsed2 = start.elapsed();
    println!("第二次解析耗时: {:?} (结果: {:?})", elapsed2, addrs_cached);

    if elapsed2.as_nanos() > 0 {
        let speedup = elapsed1.as_secs_f64() / elapsed2.as_secs_f64();
        println!("加速比: {:.2}x", speedup);
    } else {
        println!("加速比: ∞ (耗时过短无法计算)");
    }
}
