#!/bin/sh
set -eu
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
