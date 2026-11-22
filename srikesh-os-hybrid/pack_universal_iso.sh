#!/usr/bin/env bash
set -euo pipefail

WIN_OUT="/mnt/c/Users/shres/OneDrive/Desktop/RUST MICRO-OS KERNEL PROJECT/srikesh-os-hybrid/out"
STAGE="$WIN_OUT/usb_stage"
WORK="/tmp/srikesh_universal_iso_work"
ISO_OUT="/tmp/srikesh-os-hybrid-universal.iso"
FINAL_ISO="$WIN_OUT/srikesh-os-hybrid-universal.iso"
FINAL_SHA="$WIN_OUT/srikesh-os-hybrid-universal.iso.sha256"

echo "[1/3] Verifying v3 universal usb_stage files (read-only)..."
test -f "$STAGE/boot/grub/grub.cfg"
test -f "$STAGE/live/vmlinuz"
test -f "$STAGE/live/initrd.img"
test -f "$STAGE/live/filesystem.squashfs"

rm -rf "$WORK" "$ISO_OUT"
mkdir -p "$WORK/boot/grub" "$WORK/live"

cp "$STAGE/boot/grub/grub.cfg" "$WORK/boot/grub/grub.cfg"
cp "$STAGE/live/vmlinuz" "$WORK/live/vmlinuz"
cp "$STAGE/live/initrd.img" "$WORK/live/initrd.img"
cp "$STAGE/live/filesystem.squashfs" "$WORK/live/filesystem.squashfs"

echo "[2/3] Building standard UEFI + BIOS hybrid ISO with grub-mkrescue..."
grub-mkrescue -o "$ISO_OUT" "$WORK" -- -volid SRIKESH_OS

echo "[3/3] Copying ISO and generating SHA-256 checksum..."
cp "$ISO_OUT" "$FINAL_ISO"
rm -rf "$WORK" "$ISO_OUT"

ISO_BYTES=$(stat -c%s "$FINAL_ISO")
ISO_MB=$((ISO_BYTES / 1024 / 1024))
SHA=$(sha256sum "$FINAL_ISO" | awk '{print $1}')
echo "${SHA}  srikesh-os-hybrid-universal.iso" > "$FINAL_SHA"

echo "============================================================"
echo "  ✓ UNIVERSAL v3 ISO (grub-mkrescue) BUILT SUCCESSFULLY!"
echo "  File   : $FINAL_ISO"
echo "  Size   : ${ISO_MB} MB ($ISO_BYTES bytes)"
echo "  SHA256 : $SHA"
echo "============================================================"
