#!/usr/bin/env bash
set -e
IMG="/mnt/c/Users/shres/OneDrive/Desktop/RUST MICRO-OS KERNEL PROJECT/srikesh-os/build_artifacts/uefi.img"
DEST="/mnt/c/Users/shres/OneDrive/Desktop/RUST MICRO-OS KERNEL PROJECT/srikesh-os/build_artifacts/usb_root"
mkdir -p "$DEST" /mnt/uefi_tmp
DEV=$(losetup -Pf --show "$IMG")
echo "Attached loop device: $DEV"
sleep 1
if [ -b "${DEV}p1" ]; then
    mount "${DEV}p1" /mnt/uefi_tmp
else
    mount "$DEV" /mnt/uefi_tmp
fi
cp -rv /mnt/uefi_tmp/* "$DEST/"
umount /mnt/uefi_tmp
losetup -d "$DEV"
echo "Extracted UEFI files to $DEST:"
ls -laR "$DEST"
