mod dns_server;
use std::{
  array,
  cmp::Reverse,
  fs,
  sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
  },
  thread,
};

use aok::{OK, Void};
use dns_server::{DNS_SERVER_LI, DnsServer};
use pick_fast::PickFast;
use svg::{
  Document,
  node::element::{Group, Polygon, Rectangle, Text},
};

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

// Latency in microseconds for each DNS server / 每个 DNS 服务器的延时（微秒）
const LATENCIES_US: [u32; 8] = [
  100_000, 80_000, 5_000, 60_000, 40_000, 20_000, 70_000, 90_000,
];
const LATENCIES_MS: [u32; 8] = [100, 80, 5, 60, 40, 20, 70, 90];

#[test]
fn test_pick_count_with_chart() -> Void {
  let lb = Arc::new(PickFast::<DnsServer, pick_fast::Inverse>::new(
    DNS_SERVER_LI,
  ));
  let pick_counts: Arc<[AtomicU32; 8]> = Arc::new([const { AtomicU32::new(0) }; 8]);

  println!("Running 10000 picks to verify fast node is selected more than slow node...");

  let handles: Vec<_> = (0..8)
    .map(|_| {
      let lb = lb.clone();
      let counts = pick_counts.clone();
      thread::spawn(move || {
        for _ in 0..1250 {
          let node = lb.pick();
          counts[node.index].fetch_add(1, Ordering::Relaxed);
          lb.set(node.index, LATENCIES_US[node.index]);
        }
      })
    })
    .collect();

  for h in handles {
    let _ = h.join();
  }

  let counts: [u32; 8] = array::from_fn(|i| pick_counts[i].load(Ordering::Relaxed));

  println!("\n=== 节点选择统计 ===");
  for (i, &count) in counts.iter().enumerate() {
    println!(
      "Node {}: {}ms -> {count} times",
      DNS_SERVER_LI[i].ip, LATENCIES_MS[i]
    );
  }

  let (slow_count, fast_count) = (counts[0], counts[2]);
  println!("\n慢节点 (8.8.8.8, 100ms) 被选中: {slow_count} 次");
  println!("快节点 (223.5.5.5, 5ms) 被选中: {fast_count} 次");
  println!("比例: {:.2}", fast_count as f64 / slow_count as f64);

  assert!(
    fast_count > slow_count,
    "Fast node should be picked more than slow node"
  );

  // Sort by selection count / 按选中次数排序
  let mut data: Vec<_> = DNS_SERVER_LI.iter().zip(counts).zip(LATENCIES_MS).collect();
  data.sort_by_key(|&((_, count), _)| Reverse(count));

  let (servers, counts, latencies): (Vec<_>, Vec<_>, Vec<_>) =
    data.into_iter().map(|((s, c), l)| (s, c, l)).fold(
      (vec![], vec![], vec![]),
      |(mut s, mut c, mut l), (sv, cv, lv)| {
        s.push(sv);
        c.push(cv);
        l.push(lv);
        (s, c, l)
      },
    );

  draw_chart(&servers, &latencies, &counts)?;

  OK
}

fn draw_chart(servers: &[&DnsServer], latencies: &[u32], counts: &[u32]) -> Void {
  fs::create_dir_all("readme")?;

  for (filename, is_zh) in [("readme/rank-zh.svg", true), ("readme/rank-en.svg", false)] {
    draw_3d_chart(servers, latencies, counts, filename, is_zh)?;
  }

  println!("SVG图表已保存到 readme/rank-zh.svg 和 readme/rank-en.svg");
  println!("SVG charts saved to readme/rank-zh.svg and readme/rank-en.svg");

  OK
}

fn draw_3d_chart(
  servers: &[&DnsServer],
  latencies: &[u32],
  counts: &[u32],
  filename: &str,
  is_zh: bool,
) -> Void {
  let (width, height, margin, title_margin) = (1000, 480, 50, 60);
  let (chart_width, chart_height) = (width - 2 * margin, height - 2 * margin - title_margin - 60);
  let max_count = *counts.iter().max().unwrap_or(&1);

  // 3D parameters
  let (depth, angle_rad) = (40.0, 0.5_f64);
  let (dx, dy) = (depth * angle_rad.cos(), depth * angle_rad.sin());

  let mut doc = Document::new()
    .set("viewBox", (0, 0, width, height))
    .set("width", width)
    .set("height", height);

  // Colors
  let (front, top, right) = (
    "rgb(147, 197, 253)",
    "rgb(191, 219, 254)",
    "rgb(96, 165, 250)",
  );

  // Title
  let title = if is_zh {
    "PickFast 使用演示：DNS 响应延时 与 选中次数"
  } else {
    "PickFast Demo: DNS Response Latency vs Selection Count"
  };

  doc = doc.add(
    Text::new(title)
      .set("x", width / 2)
      .set("y", 35)
      .set("text-anchor", "middle")
      .set("font-family", "Arial, sans-serif")
      .set("font-size", 22)
      .set("font-weight", "bold")
      .set("fill", "rgb(30, 41, 59)"),
  );

  // Draw 3D bars
  let bar_width = chart_width as f64 / 8.0 * 0.7;
  let bar_spacing = chart_width as f64 / 8.0;

  for (i, &count) in counts.iter().enumerate() {
    if count == 0 {
      continue;
    }

    let x = margin as f64 + i as f64 * bar_spacing + bar_spacing * 0.15;
    let bar_height = (count as f64 / max_count as f64) * chart_height as f64;
    let y = margin as f64 + title_margin as f64 + chart_height as f64 - bar_height;

    let mut group = Group::new();

    // Front face
    group = group.add(
      Rectangle::new()
        .set("x", x)
        .set("y", y)
        .set("width", bar_width)
        .set("height", bar_height)
        .set("fill", front)
        .set("stroke", "rgba(0,0,0,0.2)")
        .set("stroke-width", 1),
    );

    // Top face
    group = group.add(
      Polygon::new()
        .set(
          "points",
          format!(
            "{x},{y} {},{y} {},{} {},{}",
            x + bar_width,
            x + bar_width + dx,
            y - dy,
            x + dx,
            y - dy
          ),
        )
        .set("fill", top)
        .set("stroke", "rgba(0,0,0,0.2)")
        .set("stroke-width", 1),
    );

    // Right face
    group = group.add(
      Polygon::new()
        .set(
          "points",
          format!(
            "{},{y} {},{} {},{} {},{}",
            x + bar_width,
            x + bar_width,
            y + bar_height,
            x + bar_width + dx,
            y + bar_height - dy,
            x + bar_width + dx,
            y - dy
          ),
        )
        .set("fill", right)
        .set("stroke", "rgba(0,0,0,0.2)")
        .set("stroke-width", 1),
    );

    let label_x = x + bar_width / 2.0;

    // Value label (stroke + text)
    for (fill, stroke, stroke_w) in [("none", "white", 3), ("black", "none", 0)] {
      group = group.add(
        Text::new(format!("{count}"))
          .set("x", label_x)
          .set("y", y - 10.0)
          .set("text-anchor", "middle")
          .set("font-family", "Arial, sans-serif")
          .set("font-size", 13)
          .set("font-weight", "bold")
          .set("fill", fill)
          .set("stroke", stroke)
          .set("stroke-width", stroke_w),
      );
    }

    // Latency label
    group = group.add(
      Text::new(format!("{}ms", latencies[i]))
        .set("x", label_x)
        .set("y", margin + title_margin + chart_height + 20)
        .set("text-anchor", "middle")
        .set("font-family", "Arial, sans-serif")
        .set("font-size", 12)
        .set("fill", "rgb(71, 85, 105)"),
    );

    // IP label
    group = group.add(
      Text::new(format!("{}", servers[i].ip))
        .set("x", label_x)
        .set("y", margin + title_margin + chart_height + 38)
        .set("text-anchor", "middle")
        .set("font-family", "Arial, sans-serif")
        .set("font-size", 10)
        .set("fill", "rgb(100, 116, 139)"),
    );

    doc = doc.add(group);
  }

  // Y-axis label
  let (y_label_x, y_label_y) = (35, margin + title_margin + chart_height / 2);
  let y_desc = if is_zh {
    "选择次数"
  } else {
    "Selection Count"
  };
  doc = doc.add(
    Text::new(y_desc)
      .set("x", y_label_x)
      .set("y", y_label_y)
      .set("text-anchor", "middle")
      .set("font-family", "Arial, sans-serif")
      .set("font-size", 16)
      .set("fill", "rgb(71, 85, 105)")
      .set(
        "transform",
        format!("rotate(-90, {y_label_x}, {y_label_y})"),
      ),
  );

  fs::write(filename, doc.to_string())?;

  OK
}
