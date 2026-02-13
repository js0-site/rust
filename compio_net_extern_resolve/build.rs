use std::{
  env, fs, io,
  path::{Path, PathBuf},
  process::Command,
  result,
};

use serde::Deserialize;

#[derive(thiserror::Error, Debug)]
enum Error {
  #[error("cargo metadata exited with error: {0}")]
  Metadata(String),
  #[error("Failed to execute cargo metadata: {0}")]
  MetadataExec(#[from] io::Error),
  #[error("Failed to parse cargo metadata JSON: {0}")]
  MetadataParse(#[from] sonic_rs::Error),
  #[error("Dependency {0} not found in metadata")]
  DependencyNotFound(String),
  #[error("No parent directory for manifest_path: {0}")]
  NoParentDir(String),
  #[error("Failed to read file {0}: {1}")]
  ReadFile(PathBuf, io::Error),
  #[error("Failed to write file {0}: {1}")]
  WriteFile(PathBuf, io::Error),
}

const DIR: &str = env!("CARGO_MANIFEST_DIR");

const PATH_EXTERN_RESOLVE: &str = r#"#[path = "extern_resolve.rs"]"#;

const TOML: &str = "Cargo.toml";
const COMPIO_NET: &str = "compio-net";
const CFG_IF: &str = "cfg_if::cfg_if! {";
const MOD_RS: &str = "src/resolve/mod.rs";
const CARGO_CMD: &str = "cargo";

type Result<T> = result::Result<T, Error>;

#[derive(Deserialize, Debug)]
struct Metadata {
  packages: Vec<Package>,
}

#[derive(Deserialize, Debug)]
struct Package {
  name: String,
  manifest_path: String,
}

fn metadata() -> Result<Metadata> {
  let output = Command::new(env::var("CARGO").unwrap_or_else(|_| CARGO_CMD.to_string()))
    .args(["metadata", "--format-version=1"])
    .output()?;

  if !output.status.success() {
    return Err(Error::Metadata(
      String::from_utf8_lossy(&output.stderr).to_string(),
    ));
  }

  Ok(sonic_rs::from_slice(&output.stdout)?)
}

#[derive(Debug)]
struct Patcher {
  net: PathBuf,
}

impl Patcher {
  fn new(net: PathBuf) -> Self {
    Self { net }
  }

  fn run(&self) -> Result<()> {
    let mod_path = self.net.join(MOD_RS);
    let (patched, content) = self.is_patched(&mod_path)?;
    if patched {
      return Ok(());
    }

    self.copy()?;
    self.patch_mod(&mod_path, content)?;
    self.patch_lib_rs()?;

    Ok(())
  }

  fn is_patched(&self, mod_path: &Path) -> Result<(bool, String)> {
    let content =
      fs::read_to_string(mod_path).map_err(|e| Error::ReadFile(mod_path.to_path_buf(), e))?;
    Ok((content.contains(PATH_EXTERN_RESOLVE), content))
  }

  fn copy(&self) -> Result<()> {
    let name = "extern_resolve.rs";
    let src = "src";
    let target = self.net.join(src).join("resolve").join(name);
    let extern_resolve = Path::new(DIR).join(src).join(name);

    fs::copy(extern_resolve, &target)
      .map(|_| ())
      .map_err(|e| Error::WriteFile(target.to_path_buf(), e))
  }

  fn patch_mod(&self, path: &Path, content: String) -> Result<()> {
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let start_idx = lines.iter().position(|line| line.contains(CFG_IF));

    if let Some(start) = start_idx {
      let start = start + 1;
      lines[start] = format!(
        r#"    if #[cfg(compio_dns)] {{
      {PATH_EXTERN_RESOLVE}
      pub mod sys;
    }} else {}"#,
        lines[start].trim()
      );

      fs::write(path, lines.join("\n")).map_err(|e| Error::WriteFile(path.to_path_buf(), e))?;
    }

    Ok(())
  }

  fn patch_lib_rs(&self) -> Result<()> {
    let path = self.net.join("src/lib.rs");
    let content = fs::read_to_string(&path).map_err(|e| Error::ReadFile(path.to_path_buf(), e))?;

    let new_content = content.replace("\nmod resolve;", "\npub mod resolve;");
    fs::write(&path, new_content).map_err(|e| Error::WriteFile(path.to_path_buf(), e))?;

    Ok(())
  }
}

fn main() -> Result<()> {
    println!("cargo:rerun-if-env-changed=RUSTFLAGS");
    let rustflags = std::env::var("RUSTFLAGS").unwrap_or_default();
    if !rustflags.contains("compio_dns") {
        for line in r#"
⚠️  compio_dns is NOT enabled!
To integrate into compio ecosystem, you must set `compio_dns` cfg.

Example (Bash):
  export RUSTFLAGS="--cfg compio_dns $RUSTFLAGS"

Example (mise.toml):
  [env]
  RUSTFLAGS = "--cfg compio_dns {{ env.RUSTFLAGS }}"

⚠️  compio_dns 未启用！
若要整合到 compio 生态，必须设置 `compio_dns` cfg。

示例 (Bash):
  export RUSTFLAGS="--cfg compio_dns $RUSTFLAGS"
"#
        .trim()
        .lines()
        {
            println!("cargo:warning={}", line);
        }
        return Ok(())
    }
  println!("cargo:rerun-if-changed={TOML}");
  println!("cargo:rerun-if-changed=build.rs");
  println!("cargo:rerun-if-changed=src");
  // println!("cargo:rustc-check-cfg=cfg(compio_dns)");
  // println!("cargo:rustc-cfg=compio_dns");

  let meta = metadata()?;
  let net = pkg(&meta, COMPIO_NET)?;

  let patcher = Patcher::new(net);
  patcher.run()?;

  Ok(())
}

fn pkg(meta: &Metadata, name: &str) -> Result<PathBuf> {
  let pkg = meta
    .packages
    .iter()
    .find(|p| p.name == name)
    .ok_or_else(|| Error::DependencyNotFound(name.to_string()))?;

  Path::new(&pkg.manifest_path)
    .parent()
    .ok_or_else(|| Error::NoParentDir(pkg.manifest_path.clone()))
    .map(|p| p.to_path_buf())
}
