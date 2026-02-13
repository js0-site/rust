use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Context, Result, bail};

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=patch/");

    // 1. 设置 nightly 环境
    unsafe { std::env::set_var("__CARGO_TEST_CHANNEL_OVERRIDE_DO_NOT_USE_THIS", "nightly") };

    // 2. 手动查找依赖源码，避免调用 cargo metadata 导致死锁
    let home = std::env::var("HOME").context("HOME not set")?;
    let cargo_registry_src = Path::new(&home).join(".cargo/registry/src");
    
    // 需要 patch 的包列表
    let patch_targets = vec![
        ("compio-net", "0.11.0", "patch/compio-net.patch"),
    ];

    for (pkg_name, pkg_version, patch_file) in patch_targets {
        let src_dir = find_package_src(&cargo_registry_src, pkg_name, pkg_version)
            .context(format!("Failed to find source for {} {}", pkg_name, pkg_version))?;
            
        patch_package(pkg_name, pkg_version, &src_dir, patch_file)?;
    }

    Ok(())
}

fn find_package_src(registry_base: &Path, name: &str, version: &str) -> Result<PathBuf> {
    // registry_base 通常包含 github.com-xxxx 和 index.crates.io-xxxx 目录
    if !registry_base.exists() {
        bail!("Cargo registry src dir not found at {:?}", registry_base);
    }

    let target_dir_name = format!("{}-{}", name, version);

    // 遍历 registry_base 下的所有子目录 (例如 index.crates.io-6f17d22bba15001f)
    for entry in std::fs::read_dir(registry_base)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let candidate = path.join(&target_dir_name);
            if candidate.exists() {
                return Ok(candidate);
            }
        }
    }

    bail!("Package dir {} not found in any registry under {:?}", target_dir_name, registry_base);
}

fn patch_package(name: &str, version: &str, src_dir: &Path, patch_file: &str) -> Result<()> {
    let target_patch_dir_name = format!("{}-{}", name, version);
    let target_patch_dir = Path::new("target/patch").join(&target_patch_dir_name);

    if target_patch_dir.exists() {
        // 简单检查是否已有内容，假设已存在则跳过
        return Ok(());
    }

    println!("Patching {} from {:?} to {:?}", name, src_dir, target_patch_dir);

    // 复制源码
    std::fs::create_dir_all(&target_patch_dir)?;
    cp_r(src_dir, &target_patch_dir)?;

    // 应用补丁
    let patch_path = Path::new(patch_file);
    if !patch_path.exists() {
         eprintln!("Warning: Patch file not found at {:?}", patch_path);
    } else {
        let patch_content = std::fs::read_to_string(patch_path)?;
        
        // 确保 patch 工具存在
        let status = Command::new("patch")
            .arg("-p1")
            .arg("-d")
            .arg(&target_patch_dir)
            .stdin(std::process::Stdio::piped())
            .spawn()
            .context("Failed to spawn patch command")?
            .wait_with_input(patch_content.as_bytes())?;

        if !status.status.success() {
            bail!("Patch command failed for {}: {}", name, String::from_utf8_lossy(&status.stderr));
        }
    }
    
    Ok(())
}


fn cp_r(src: &Path, dst: &Path) -> Result<()> {
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let path = entry.path();
        let name = entry.file_name();
        let dst_path = dst.join(name);

        if ty.is_dir() {
            std::fs::create_dir_all(&dst_path)?;
            cp_r(&path, &dst_path)?;
        } else {
            std::fs::copy(&path, &dst_path)?;
        }
    }
    Ok(())
}

trait CommandExt {
    fn wait_with_input(self, input: &[u8]) -> std::io::Result<std::process::Output>;
}

impl CommandExt for std::process::Child {
    fn wait_with_input(mut self, input: &[u8]) -> std::io::Result<std::process::Output> {
         if let Some(mut stdin) = self.stdin.take() {
            use std::io::Write;
            stdin.write_all(input)?;
        }
        self.wait_with_output()
    }
}
