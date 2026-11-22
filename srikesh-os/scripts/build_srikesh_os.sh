#!/usr/bin/env bash
# SRIKESH OS - Master Build Script
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

export PATH="$HOME/.cargo/bin:$PATH"

echo "=========================================================="
echo "         SRIKESH OS — MASTER BARE-METAL BUILD             "
echo "=========================================================="
echo "Project Directory: ${PROJECT_DIR}"
echo "Toolchain        : $(rustc --version)"
echo "Cargo            : $(cargo --version)"

cd "${PROJECT_DIR}"

echo ""
echo "[1/4] Cleaning previous build artifacts..."
mkdir -p "${PROJECT_DIR}/build_artifacts"
rm -f "${PROJECT_DIR}/build_artifacts/bios.img" "${PROJECT_DIR}/build_artifacts/uefi.img"

echo "[2/4] Compiling SRIKESH OS Kernel and Bootloader..."
cargo clean -p kernel 2>/dev/null || true
cargo build --release

echo "[3/4] Locating generated bootable disk images..."
RELEASE_BIOS=$(find "${PROJECT_DIR}/target/release/build" -name "bios.img" | head -n 1)
RELEASE_UEFI=$(find "${PROJECT_DIR}/target/release/build" -name "uefi.img" | head -n 1)

if [ -z "${RELEASE_BIOS}" ] || [ -z "${RELEASE_UEFI}" ]; then
    echo "[ERROR] Disk images not found in target/release/build!"
    exit 1
fi

echo "Found BIOS Image: ${RELEASE_BIOS}"
echo "Found UEFI Image: ${RELEASE_UEFI}"

cp "${RELEASE_BIOS}" "${PROJECT_DIR}/build_artifacts/bios.img"
cp "${RELEASE_UEFI}" "${PROJECT_DIR}/build_artifacts/uefi.img"

echo ""
echo "[4/4] Verification & Checksums:"
ls -lh "${PROJECT_DIR}/build_artifacts/bios.img" "${PROJECT_DIR}/build_artifacts/uefi.img"
sha256sum "${PROJECT_DIR}/build_artifacts/bios.img" "${PROJECT_DIR}/build_artifacts/uefi.img"

echo ""
echo "=========================================================="
echo "  ✓ SRIKESH OS BUILD SUCCESSFUL — IMAGES READY FOR BOOT   "
echo "=========================================================="
