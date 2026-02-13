use crate::{lang::Lang, library::Metrics, scenario::Scenario};

pub fn run<S: Scenario>(lang: &dyn Lang, libs: &[(&str, Metrics)], n: usize) {
  println!("\n--- {} (N={}) ---", lang.t(S::NAME_ZH, S::NAME_EN), n);

  let b = &libs[0].1;

  struct ReportParams<'a, F> {
    lang: &'a dyn Lang,
    libs: &'a [(&'a str, Metrics)],
    zh: &'static str,
    en: &'static str,
    unit: &'static str,
    value_fn: F,
    higher_better: bool,
  }

  fn pr<F>(params: ReportParams<'_, F>)
  where
    F: Fn(&Metrics) -> f64,
  {
    if params.libs.len() >= 2 {
      let v_pc = (params.value_fn)(&params.libs[0].1);
      let v_sucds = (params.value_fn)(&params.libs[1].1);
      let ratio = v_pc / v_sucds;

      let worse = if params.higher_better {
        ratio < 1.0 // pc slower
      } else {
        ratio > 1.0 // pc larger
      };
      let note = if worse { " * 需优化" } else { "" };
      println!(
        "{} ({}): {:.2}x{}",
        params.lang.t(params.zh, params.en),
        params.unit,
        ratio,
        note
      );
    }
    for (name, m) in params.libs {
      println!("  {}: {:.2}", name, (params.value_fn)(m));
    }
  }

  pr(ReportParams {
    lang,
    libs,
    zh: "大小",
    en: "Size",
    unit: "MB",
    value_fn: |m: &Metrics| m.size_mb,
    higher_better: false,
  });
  pr(ReportParams {
    lang,
    libs,
    zh: "压缩率",
    en: "Ratio",
    unit: "%",
    value_fn: |m: &Metrics| m.ratio_pct,
    higher_better: false,
  });
  pr(ReportParams {
    lang,
    libs,
    zh: "构建",
    en: "Build",
    unit: "MB/s",
    value_fn: |m: &Metrics| m.build_mops * 1_000_000.0 * 8.0 / 1024.0 / 1024.0,
    higher_better: true,
  });
  pr(ReportParams {
    lang,
    libs,
    zh: "随机",
    en: "Get",
    unit: "MB/s",
    value_fn: |m: &Metrics| m.get_mops * 1_000_000.0 * 8.0 / 1024.0 / 1024.0,
    higher_better: true,
  });
  pr(ReportParams {
    lang,
    libs,
    zh: "顺序",
    en: "Iter",
    unit: "MB/s",
    value_fn: |m: &Metrics| m.iter_mops * 1_000_000.0 * 8.0 / 1024.0 / 1024.0,
    higher_better: true,
  });
  pr(ReportParams {
    lang,
    libs,
    zh: "延迟 P99",
    en: "Delay P99",
    unit: "ns",
    value_fn: |m: &Metrics| m.latency_p99_ns,
    higher_better: false,
  });
  pr(ReportParams {
    lang,
    libs,
    zh: "延迟 Avg",
    en: "Delay Avg",
    unit: "ns",
    value_fn: |m: &Metrics| 1000.0 / m.get_mops,
    higher_better: false,
  });

  if libs.len() >= 2 {
    if let (Some(bv), Some(v2)) = (b.rev_mops, libs[1].1.rev_mops) {
      let ratio = v2 / bv;
      let note = if ratio > 1.0 { " * 需优化" } else { "" };
      println!("{} (MB/s): {:.2}x{}", lang.t("逆向", "Rev"), ratio, note);
    } else {
      println!("{} (MB/s):", lang.t("逆向", "Rev"));
    }
  } else {
    println!("{} (MB/s):", lang.t("逆向", "Rev"));
  }
  for (name, m) in libs {
    match m.rev_mops {
      Some(v) => println!("  {}: {:.2}", name, v * 1_000_000.0 * 8.0 / 1024.0 / 1024.0),
      None => println!("  {}: N/A", name),
    }
  }
}
