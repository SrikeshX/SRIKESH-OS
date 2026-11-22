#!/usr/bin/env bash
set -euo pipefail

WIN_OUT="/mnt/c/Users/shres/OneDrive/Desktop/RUST MICRO-OS KERNEL PROJECT/srikesh-os-hybrid/out"
STAGE="$WIN_OUT/usb_stage"
WORK="/tmp/srikesh_universal_iso_work"
ISO_OUT="/tmp/srikesh-os-hybrid-universal.iso"
FINAL_ISO="$WIN_OUT/srikesh-os-hybrid-universal.iso"

echo "[1/4] Verifying v3 universal usb_stage files (read-only)..."
test -f "$STAGE/EFI/BOOT/BOOTX64.EFI"
test -f "$STAGE/EFI/BOOT/grubx64.efi"
test -f "$STAGE/boot/grub/grub.cfg"
test -f "$STAGE/live/vmlinuz"
test -f "$STAGE/live/initrd.img"
test -f "$STAGE/live/filesystem.squashfs"

rm -rf "$WORK" "$ISO_OUT"
mkdir -p "$WORK/EFI/BOOT" "$WORK/EFI/debian" "$WORK/boot/grub" "$WORK/live"

cp "$STAGE/EFI/BOOT/BOOTX64.EFI" "$WORK/EFI/BOOT/"
cp "$STAGE/EFI/BOOT/grubx64.efi" "$WORK/EFI/BOOT/"
if test -f "$STAGE/EFI/BOOT/mmx64.efi"; then
    cp "$STAGE/EFI/BOOT/mmx64.efi" "$WORK/EFI/BOOT/"
fi
cp "$STAGE/boot/grub/grub.cfg" "$WORK/boot/grub/grub.cfg"
cp "$STAGE/boot/grub/grub.cfg" "$WORK/EFI/BOOT/grub.cfg"
cp "$STAGE/boot/grub/grub.cfg" "$WORK/EFI/debian/grub.cfg"
cp "$STAGE/live/vmlinuz" "$WORK/live/vmlinuz"
cp "$STAGE/live/initrd.img" "$WORK/live/initrd.img"
cp "$STAGE/live/filesystem.squashfs" "$WORK/live/filesystem.squashfs"

echo "[2/4] Creating signed-UEFI El Torito boot image (efi.img)..."
EFI_IMG="$WORK/boot/grub/efi.img"
dd if=/dev/zero of="$EFI_IMG" bs=1M count=12 status=none
mkfs.vfat -F 16 -n "SRIKESH_EFI" "$EFI_IMG" >/dev/null
mmd -i "$EFI_IMG" ::/EFI ::/EFI/BOOT ::/EFI/debian ::/boot ::/boot/grub
mcopy -i "$EFI_IMG" "$WORK/EFI/BOOT/BOOTX64.EFI" ::/EFI/BOOT/BOOTX64.EFI
mcopy -i "$EFI_IMG" "$WORK/EFI/BOOT/grubx64.efi" ::/EFI/BOOT/grubx64.efi
if test -f "$WORK/EFI/BOOT/mmx64.efi"; then
    mcopy -i "$EFI_IMG" "$WORK/EFI/BOOT/mmx64.efi" ::/EFI/BOOT/mmx64.efi
fi
mcopy -i "$EFI_IMG" "$WORK/boot/grub/grub.cfg" ::/EFI/BOOT/grub.cfg
mcopy -i "$EFI_IMG" "$WORK/boot/grub/grub.cfg" ::/EFI/debian/grub.cfg
mcopy -i "$EFI_IMG" "$WORK/boot/grub/grub.cfg" ::/boot/grub/grub.cfg

echo "[3/4] Creating Legacy BIOS El Torito image (bios.img)..."
cat > /tmp/bios_early.cfg << 'EOF'
search --no-floppy --file --set=root /live/vmlinuz
set prefix=($root)/boot/grub
configfile /boot/grub/grub.cfg
EOF

grub-mkimage \
    -O i386-pc-eltorito \
    -p /boot/grub \
    -c /tmp/bios_early.cfg \
    -o "$WORK/boot/grub/bios.img" \
    biosdisk iso9660 part_gpt part_msdos fat search search_fs_file search_label normal linux all_video configfile echo test

echo "[4/4] Building hybrid UEFI (Secure Boot) + Legacy BIOS ISO..."
xorriso -as mkisofs \
    -iso-level 3 \
    -full-iso9660-filenames \
    -volid "SRIKESH_OS" \
    -joliet -joliet-long -rock \
    --grub2-mbr /usr/lib/grub/i386-pc/boot_hybrid.img \
    -eltorito-boot boot/grub/bios.img \
    -no-emul-boot \
    -boot-load-size 4 \
    -boot-info-table \
    --grub2-boot-info \
    -eltorito-alt-boot \
    -e boot/grub/efi.img \
    -no-emul-boot \
    -isohybrid-gpt-basdat \
    -output "$ISO_OUT" \
    "$WORK"

cp "$ISO_OUT" "$FINAL_ISO"
rm -rf "$WORK" "$ISO_OUT" /tmp/bios_early.cfg

ISO_BYTES=$(stat -c%s "$FINAL_ISO")
ISO_MB=$((ISO_BYTES / 1024 / 1024))
SHA=$(sha256sum "$FINAL_ISO" | awk '{print $1}')
echo "============================================================"
echo "  ✓ UNIVERSAL v3 ISO BUILT SUCCESSFULLY!"
echo "  File   : $FINAL_ISO"
echo "  Size   : ${ISO_MB} MB ($ISO_BYTES bytes)"
echo "  SHA256 : $SHA"
echo "============================================================"
