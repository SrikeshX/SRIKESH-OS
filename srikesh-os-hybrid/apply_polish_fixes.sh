#!/usr/bin/env bash
set -euo pipefail

WIN_PROJECT="/mnt/c/Users/shres/OneDrive/Desktop/RUST MICRO-OS KERNEL PROJECT/srikesh-os-hybrid"
BUILD_ROOT="/root/srikesh-hybrid-build"
CHROOT="$BUILD_ROOT/chroot"
ISO_DIR="$BUILD_ROOT/iso"
OUT_DIR="$WIN_PROJECT/out"

echo "[1/4] Updating srikesh_shell.py & Openbox rc.xml (Chromium real sandbox enabled)..."
sed 's/\r$//' "$WIN_PROJECT/shell/srikesh_shell.py" > "$CHROOT/opt/srikesh/srikesh_shell.py"
chmod +x "$CHROOT/opt/srikesh/srikesh_shell.py"

sed -i 's/--no-sandbox //g' "$CHROOT/home/srikesh/.config/openbox/rc.xml"

mkdir -p "$CHROOT/etc/sysctl.d"
cat > "$CHROOT/etc/sysctl.d/99-chromium-sandbox.conf" << 'EOF'
kernel.unprivileged_userns_clone=1
EOF

echo "[2/4] Silencing routine tty1 boot/login text for clean splash transition..."
: > "$CHROOT/etc/motd"
: > "$CHROOT/etc/issue"
touch "$CHROOT/home/srikesh/.hushlogin"
chroot "$CHROOT" chown srikesh:srikesh /home/srikesh/.hushlogin

cat > "$CHROOT/etc/systemd/system/getty@tty1.service.d/autologin.conf" << 'EOF'
[Service]
ExecStart=
ExecStart=-/sbin/agetty --autologin srikesh --skip-login --noissue --noclear %I $TERM
StandardInput=tty
StandardOutput=null
EOF

echo "[3/4] Repacking filesystem.squashfs (zstd)..."
mksquashfs "$CHROOT" "$ISO_DIR/live/filesystem.squashfs" -comp zstd -Xcompression-level 15 -noappend -quiet

SQUASH_BYTES=$(stat -c%s "$ISO_DIR/live/filesystem.squashfs")
SQUASH_MB=$((SQUASH_BYTES / 1024 / 1024))
echo "  -> filesystem.squashfs size: ${SQUASH_MB} MB ($SQUASH_BYTES bytes)"

echo "[4/4] Rebuilding BOOTX64.EFI (with proven FAT32/GPT/MBR modules + silent grub.cfg) & ISO..."
sed -i 's/\r$//' "$OUT_DIR/usb_stage/boot/grub/grub.cfg"
cp "$OUT_DIR/usb_stage/boot/grub/grub.cfg" "$ISO_DIR/boot/grub/grub.cfg"
cp "$ISO_DIR/live/filesystem.squashfs" "$OUT_DIR/usb_stage/live/filesystem.squashfs"

grub-mkstandalone \
    --format=x86_64-efi \
    --output="$OUT_DIR/usb_stage/EFI/BOOT/BOOTX64.EFI" \
    --modules="part_gpt part_msdos fat iso9660 search search_fs_file search_label linux normal configfile all_video efi_gop efi_uga" \
    --locales="" \
    --fonts="" \
    "boot/grub/grub.cfg=$OUT_DIR/usb_stage/boot/grub/grub.cfg"

grub-mkrescue -o "$OUT_DIR/srikesh-os-hybrid.iso" "$ISO_DIR" -- -volid SRIKESH_OS

echo "============================================================"
echo "  ✓ POLISH BUILD COMPLETE!"
echo "  SquashFS : ${SQUASH_MB} MB ($SQUASH_BYTES bytes)"
echo "  BOOTX64  : $(stat -c%s "$OUT_DIR/usb_stage/EFI/BOOT/BOOTX64.EFI") bytes"
echo "============================================================"
