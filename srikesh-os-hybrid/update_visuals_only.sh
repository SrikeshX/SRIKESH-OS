#!/usr/bin/env bash
set -euo pipefail

WIN_PROJECT="/mnt/c/Users/shres/OneDrive/Desktop/RUST MICRO-OS KERNEL PROJECT/srikesh-os-hybrid"
WALLPAPER_SRC="/mnt/c/Users/shres/.gemini/antigravity/brain/43e9f22a-f7d1-4d29-a088-e60efad3b868/modern_srikesh_wallpaper_1795281175130.jpg"
BUILD_ROOT="/root/srikesh-hybrid-build"
CHROOT="$BUILD_ROOT/chroot"
ISO_DIR="$BUILD_ROOT/iso"
OUT_DIR="$WIN_PROJECT/out"

echo "[1/3] Installing new flagship 1920x1080 wallpaper & Welcome Splash..."
python3 -c "
from PIL import Image
im = Image.open('$WALLPAPER_SRC').convert('RGB')
resample = getattr(getattr(Image, 'Resampling', Image), 'LANCZOS')
im = im.resize((1920, 1080), resample)
im.save('$CHROOT/opt/srikesh/aurora_bloom.png', 'PNG', optimize=True)
print('  -> Saved 1920x1080 flagship wallpaper to /opt/srikesh/aurora_bloom.png')
"

sed 's/\r$//' "$WIN_PROJECT/shell/srikesh_shell.py" > "$CHROOT/opt/srikesh/srikesh_shell.py"
chmod +x "$CHROOT/opt/srikesh/srikesh_shell.py"

cat > "$CHROOT/home/srikesh/.xinitrc" << 'EOF'
#!/bin/sh
xsetroot -solid "#090E1A"
xrandr --auto 2>/dev/null || true
feh --bg-fill /opt/srikesh/aurora_bloom.png &
pulseaudio --start 2>/dev/null || true
openbox-session &
sleep 0.5
exec python3 /opt/srikesh/srikesh_shell.py
EOF
chmod +x "$CHROOT/home/srikesh/.xinitrc"
chroot "$CHROOT" chown srikesh:srikesh /home/srikesh/.xinitrc

echo "[2/3] Repacking ONLY filesystem.squashfs (zstd)..."
mksquashfs "$CHROOT" "$ISO_DIR/live/filesystem.squashfs" -comp zstd -Xcompression-level 15 -noappend -quiet

SQUASH_BYTES=$(stat -c%s "$ISO_DIR/live/filesystem.squashfs")
SQUASH_MB=$((SQUASH_BYTES / 1024 / 1024))
echo "  -> filesystem.squashfs size: ${SQUASH_MB} MB ($SQUASH_BYTES bytes)"

echo "[3/3] Updating usb_stage/live/filesystem.squashfs (preserving fixed BOOTX64.EFI & grub.cfg) + ISO..."
cp "$ISO_DIR/live/filesystem.squashfs" "$OUT_DIR/usb_stage/live/filesystem.squashfs"
cp "$OUT_DIR/usb_stage/boot/grub/grub.cfg" "$ISO_DIR/boot/grub/grub.cfg"
grub-mkrescue -o "$OUT_DIR/srikesh-os-hybrid.iso" "$ISO_DIR" -- -volid SRIKESH_OS

echo "============================================================"
echo "  ✓ VISUAL UPDATE COMPLETE (BOOTX64.EFI & Wi-Fi UNTOUCHED)"
echo "  SquashFS Size : ${SQUASH_MB} MB ($SQUASH_BYTES bytes)"
echo "============================================================"
