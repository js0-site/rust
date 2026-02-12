use std::{env, fs, path::Path, str::FromStr};

/// Get an environment variable or a default value and register it for cargo rerun.
/// 获取环境变量或默认值，并将其注册为 cargo rerun 触发项。
fn get_env_or_default<T: FromStr>(name: &str, default: T) -> T {
  println!("cargo:rerun-if-env-changed={}", name);
  env::var(name)
    .ok()
    .and_then(|v| v.parse().ok())
    .unwrap_or(default)
}

fn main() {
  println!("cargo:rerun-if-changed=build.rs");

  let out_dir = env::var_os("OUT_DIR").expect("OUT_DIR not set");
  let dest_path = Path::new(&out_dir).join("pc_consts.rs");

  // Defaults based on tune.py results (128, 32, 4)
  // 基于 tune.py 调优结果的默认值 (128, 32, 4)
  let block_len = get_env_or_default("PC_BLOCK_LEN", 128usize);
  let epsilon = get_env_or_default("PC_EPSILON", 32usize);
  let ex_penalty = get_env_or_default("PC_EX_PENALTY", 2u8);

  let content = format!(
    r#" 
pub const BLOCK_LEN: usize = {block_len};
pub const DEFAULT_EPSILON: usize = {epsilon};
pub const DEFAULT_EX_PENALTY: u8 = {ex_penalty};
"#
  );

  fs::write(&dest_path, content).expect("Failed to write pc_consts.rs");
}
