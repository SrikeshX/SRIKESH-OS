#!/usr/bin/env bash
export HOME=/home/shre
source /home/shre/.cargo/env 2>/dev/null || true
export PATH="$HOME/.cargo/bin:/usr/local/bin:/usr/bin:/bin:$PATH"

echo "--- Rust toolchain ---"
which cargo && cargo --version
which rustc && rustc --version

echo "--- Building SRIKESH OS ---"
cd "/mnt/c/Users/shres/OneDrive/Desktop/RUST MICRO-OS KERNEL PROJECT/srikesh-os"
cargo build --release 2>&1
echo "BUILD_EXIT_CODE: $?"
