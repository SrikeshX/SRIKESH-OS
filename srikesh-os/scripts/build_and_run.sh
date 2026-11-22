#!/usr/bin/env bash
# SRIKESH OS - Automated Build & Test Script
set -e

echo "=========================================="
echo "  SRIKESH OS - Build & Boot Verification  "
echo "=========================================="

cd "$(dirname "$0")/.."

echo "[1/3] Checking Rust Nightly Toolchain..."
rustc --version
cargo --version

echo "[2/3] Building SRIKESH OS Kernel and Disk Images..."
cargo build --release

echo "[3/3] Build Complete! Artifacts Generated."
echo "UEFI Image: target/x86_64-unknown-none/release/build/.../uefi.img"
echo "BIOS Image: target/x86_64-unknown-none/release/build/.../bios.img"

if [ "$1" == "uefi" ]; then
    echo "Booting SRIKESH OS in UEFI Mode..."
    cargo run --release -- uefi
elif [ "$1" == "bios" ]; then
    echo "Booting SRIKESH OS in BIOS Mode..."
    cargo run --release -- bios
else
    echo "Usage: ./scripts/build_and_run.sh [uefi|bios]"
fi
