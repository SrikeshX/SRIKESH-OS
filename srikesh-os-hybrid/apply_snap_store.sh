#!/usr/bin/env bash
set -euo pipefail

WIN_PROJECT="/mnt/c/Users/shres/OneDrive/Desktop/RUST MICRO-OS KERNEL PROJECT/srikesh-os-hybrid"
BUILD_ROOT="/root/srikesh-hybrid-build"
CHROOT="$BUILD_ROOT/chroot"
ISO_DIR="$BUILD_ROOT/iso"
OUT_DIR="$WIN_PROJECT/out"

mount --bind /dev "$CHROOT/dev"
mount --bind /dev/pts "$CHROOT/dev/pts"
mount -t proc proc "$CHROOT/proc"
mount -t sysfs sysfs "$CHROOT/sys"

cleanup_mounts() {
    umount -lf "$CHROOT/dev/pts" 2>/dev/null || true
    umount -lf "$CHROOT/dev" 2>/dev/null || true
    umount -lf "$CHROOT/proc" 2>/dev/null || true
    umount -lf "$CHROOT/sys" 2>/dev/null || true
}
trap cleanup_mounts EXIT

echo "[1/5] Installing Canonical snapd + squashfs-tools + fuse3 + apparmor (no kernel/Wi-Fi/Chromium changes)..."
chroot "$CHROOT" /bin/bash -c '
set -euo pipefail
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y --no-install-recommends \
    snapd \
    squashfs-tools \
    fuse3 \
    apparmor
systemctl enable snapd.socket || true
systemctl enable snapd.service || true
# Prevent offline boot delay from snapd.seeded.service while keeping snapd fully functional
systemctl disable snapd.seeded.service 2>/dev/null || true
ln -sfn /var/lib/snapd/snap /snap
mkdir -p /etc/profile.d
cat > /etc/profile.d/srikesh_snap_path.sh << "EOF"
export PATH="$PATH:/snap/bin"
EOF
chmod 644 /etc/profile.d/srikesh_snap_path.sh
apt-get clean
rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/* /usr/share/doc/* /usr/share/man/*
'

echo "[2/5] Installing SRIKESH OS Snap Store GUI & updated Shell (preserving Wi-Fi, Browser, Welcome splash)..."
sed 's/\r$//' "$WIN_PROJECT/shell/srikesh_shell.py" > "$CHROOT/opt/srikesh/srikesh_shell.py"
sed 's/\r$//' "$WIN_PROJECT/shell/srikesh_store.py" > "$CHROOT/opt/srikesh/srikesh_store.py"
chmod +x "$CHROOT/opt/srikesh/srikesh_shell.py" "$CHROOT/opt/srikesh/srikesh_store.py"

# Add F6 shortcut for Snap Store in Openbox rc.xml (if not already present)
if ! grep -q 'srikesh_store.py' "$CHROOT/home/srikesh/.config/openbox/rc.xml"; then
    sed -i 's|<keybind key="F5">|<keybind key="F6"><action name="Execute"><command>python3 /opt/srikesh/srikesh_store.py</command></action></keybind>\n    <keybind key="F5">|' "$CHROOT/home/srikesh/.config/openbox/rc.xml"
fi
chroot "$CHROOT" chown -R srikesh:srikesh /home/srikesh/.config/openbox

echo "[3/5] Running strict safety & integrity checks inside chroot..."
test -x "$CHROOT/usr/bin/snap"
test -f "$CHROOT/lib/firmware/rtw88/rtw8821c_fw.bin"
test -x "$CHROOT/usr/bin/chromium"
test -x "$CHROOT/usr/bin/nmcli"
chroot "$CHROOT" python3 -m py_compile /opt/srikesh/srikesh_shell.py /opt/srikesh/srikesh_store.py /opt/srikesh/srikesh_sysmon.py
echo "  -> VERIFIED: snapd + RTL8821CE firmware + NetworkManager + Chromium + Python GUI all intact!"

cleanup_mounts
trap - EXIT

echo "[4/5] Repacking ONLY filesystem.squashfs (zstd level 15) — bootloader/kernel untouched..."
mksquashfs "$CHROOT" "$ISO_DIR/live/filesystem.squashfs" -comp zstd -Xcompression-level 15 -noappend -quiet
cp "$ISO_DIR/live/filesystem.squashfs" "$OUT_DIR/usb_stage/live/filesystem.squashfs"

SQUASH_BYTES=$(stat -c%s "$OUT_DIR/usb_stage/live/filesystem.squashfs")
SQUASH_MB=$((SQUASH_BYTES / 1024 / 1024))
TOTAL_USB=$(du -sm "$OUT_DIR/usb_stage" | awk '{print $1}')

echo "[5/5] Building exact FAT32 UEFI test disk image for Oracle VirtualBox verification..."
IMG="/tmp/srikesh_snap_test.img"
rm -f "$IMG"
dd if=/dev/zero of="$IMG" bs=1M count=900 status=none
parted -s "$IMG" mklabel gpt
parted -s "$IMG" mkpart primary fat32 1MiB 100%
parted -s "$IMG" set 1 esp on
mkfs.vfat -F 32 -n SRIKESH_OS --offset=2048 "$IMG" >/dev/null
mmd -i "$IMG@@1M" ::/EFI ::/EFI/BOOT ::/EFI/debian ::/boot ::/boot/grub ::/live
mcopy -i "$IMG@@1M" "$OUT_DIR/usb_stage/EFI/BOOT/BOOTX64.EFI" ::/EFI/BOOT/BOOTX64.EFI
mcopy -i "$IMG@@1M" "$OUT_DIR/usb_stage/EFI/BOOT/grubx64.efi" ::/EFI/BOOT/grubx64.efi
if test -f "$OUT_DIR/usb_stage/EFI/BOOT/mmx64.efi"; then
    mcopy -i "$IMG@@1M" "$OUT_DIR/usb_stage/EFI/BOOT/mmx64.efi" ::/EFI/BOOT/mmx64.efi
fi
mcopy -i "$IMG@@1M" "$OUT_DIR/usb_stage/EFI/BOOT/grub.cfg" ::/EFI/BOOT/grub.cfg
mcopy -i "$IMG@@1M" "$OUT_DIR/usb_stage/EFI/debian/grub.cfg" ::/EFI/debian/grub.cfg
mcopy -i "$IMG@@1M" "$OUT_DIR/usb_stage/boot/grub/grub.cfg" ::/boot/grub/grub.cfg
mcopy -i "$IMG@@1M" "$OUT_DIR/usb_stage/live/vmlinuz" ::/live/vmlinuz
mcopy -i "$IMG@@1M" "$OUT_DIR/usb_stage/live/initrd.img" ::/live/initrd.img
mcopy -i "$IMG@@1M" "$OUT_DIR/usb_stage/live/filesystem.squashfs" ::/live/filesystem.squashfs

cp "$IMG" "$OUT_DIR/srikesh_snap_test.img"
rm -f "$IMG"

echo "============================================================"
echo "  ✓ SNAP STORE INTEGRATION BUILD COMPLETE!"
echo "  Total USB Stage : ${TOTAL_USB} MB (Strict limit: 2048 MB)"
echo "  SquashFS Size   : ${SQUASH_MB} MB ($SQUASH_BYTES bytes)"
echo "  Bootloader      : Signed shimx64 + grubx64 (100% untouched)"
echo "============================================================"
