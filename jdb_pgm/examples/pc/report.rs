use crate::{lang::Lang, library::Metrics, scenario::Scenario};

pub fn run<S: Scenario>(lang: &dyn Lang, libs: &[(&str, Metrics)], n: usize) {
  println!("\n--- {} (N={}) ---", lang.t(S::NAME_ZH, S::NAME_EN), n);

  let b = &libs[0].1;

  fn pr(
    lang: &dyn Lang,
    libs: &[(&str, Metrics)],
    zh: &'static str,
    en: &'static str,
    u: &str,
    f: impl Fn(&Metrics) -> f64,
    _bv: f64,
    higher_better: bool,
  ) {
    if libs.len() >= 2 {
      // ratio = pc / sucds (since b is pc, and bv is passed correctly)
      // Wait, b is libs[0].libs.
      // If we want to show pc performance relative to sucds:
      // ratio = f(&libs[0].1) / f(&libs[1].1)
      let v_pc = f(&libs[0].1);
      let v_sucds = f(&libs[1].1);
      let ratio = v_pc / v_sucds;

      let worse = if higher_better {
        ratio < 1.0 // pc slower
      } else {
        ratio > 1.0 // pc larger
      };
      let note = if worse { " * 需优化" } else { "" };
      println!("{} ({}): {:.2}x{}", lang.t(zh, en), u, ratio, note);
    }
    for (name, m) in libs {
      println!("  {}: {:.2}", name, f(m));
    }
  }

  pr(
    lang,
    libs,
    "大小",
    "Size",
    "MB",
    |m| m.size_mb,
    b.size_mb,
    false,
  );
  pr(
    lang,
    libs,
    "压缩率",
    "Ratio",
    "%",
    |m| m.ratio_pct,
    b.ratio_pct,
    false,
  );
  pr(
    lang,
    libs,
    "构建",
    "Build",
    "M/s",
    |m| m.build_mops,
    b.build_mops,
    true,
  );
  pr(
    lang,
    libs,
    "随机",
    "Get",
    "M/s",
    |m| m.get_mops,
    b.get_mops,
    true,
  );
  pr(
    lang,
    libs,
    "顺序",
    "Iter",
    "M/s",
    |m| m.iter_mops,
    b.iter_mops,
    true,
  );

  if libs.len() >= 2 {
    if let (Some(bv), Some(v2)) = (b.rev_mops, libs[1].1.rev_mops) {
      let ratio = v2 / bv;
      let note = if ratio > 1.0 { " * 需优化" } else { "" };
      println!("{} (M/s): {:.2}x{}", lang.t("逆向", "Rev"), ratio, note);
    } else {
      println!("{} (M/s):", lang.t("逆向", "Rev"));
    }
  } else {
    println!("{} (M/s):", lang.t("逆向", "Rev"));
  }
  for (name, m) in libs {
    match m.rev_mops {
      Some(v) => println!("  {}: {:.2}", name, v),
      None => println!("  {}: N/A", name),
    }
  }
}
