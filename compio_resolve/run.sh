#!/usr/bin/env bash

set -e
DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR

# 检查是否存在已生成的补丁目录
# 我们查找 target/patch 目录下名为 compio-net-* 和 compio-runtime-* 的目录
if ! ls -d target/patch/compio-net-* >/dev/null 2>&1 || ! ls -d target/patch/compio-runtime-* >/dev/null 2>&1; then
    echo "Bootstrapping patch..."
    cp Cargo.toml Cargo.toml.orig
    
    # 暂时移除 patch section 以便 build.rs 能运行
    sed -i.bak '/\[patch.crates-io\]/,$d' Cargo.toml
    
    # 触发 build.rs (虽然编译会失败，但 build.rs 会运行)
    echo "Running build.rs to prepare patch..."
    cargo check || true
    
    # 恢复 Cargo.toml
    mv Cargo.toml.orig Cargo.toml
    rm Cargo.toml.bak
fi

set -x
exec cargo run --release --example resolve
