use std::{env, fs, path::Path};

fn main() {
  println!("cargo:rerun-if-env-changed=PC_BLOCK_LEN");
  println!("cargo:rerun-if-env-changed=PC_EPSILON");

  let out_dir = env::var_os("OUT_DIR").unwrap();
  let dest_path = Path::new(&out_dir).join("pc_consts.rs");

  let block_len = env::var("PC_BLOCK_LEN").unwrap_or_else(|_| "1024".to_string());
  let epsilon = env::var("PC_EPSILON").unwrap_or_else(|_| "8".to_string());

  // Ensure valid numbers
  let block_len: usize = block_len.parse().expect("PC_BLOCK_LEN must be a number");
  let epsilon: usize = epsilon.parse().expect("PC_EPSILON must be a number");

  let content = format!(
    "pub const BLOCK_LEN: usize = {};\npub const DEFAULT_EPSILON: usize = {};\n",
    block_len, epsilon
  );

  fs::write(&dest_path, content).unwrap();
}
