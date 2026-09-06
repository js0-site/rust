#!/usr/bin/env bash
set -ex
cargo clippy --all-features --all-targets -- -D warnings
