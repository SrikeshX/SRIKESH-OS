#!/usr/bin/env bash
set -e
export HOME=/home/shre
source /home/shre/.cargo/env 2>/dev/null || true
export PATH="$HOME/.cargo/bin:/usr/local/bin:/usr/bin:/bin:$PATH"

PROJECT="/mnt/c/Users/shres/OneDrive/Desktop/RUST MICRO-OS KERNEL PROJECT/srikesh-os"
cd "$PROJECT"

echo "--- Locating build images ---"
BIOS=$(find "$PROJECT/target/release/build" -name "bios.img" | head -1)
UEFI=$(find "$PROJECT/target/release/build" -name "uefi.img" | head -1)

echo "BIOS: $BIOS"
echo "UEFI: $UEFI"

if [ -z "$BIOS" ] || [ -z "$UEFI" ]; then
    echo "ERROR: Could not find disk images!"
    exit 1
fi

cp "$BIOS" "$PROJECT/build_artifacts/bios.img"
cp "$UEFI" "$PROJECT/build_artifacts/uefi.img"

echo "--- Checksums ---"
ls -lh "$PROJECT/build_artifacts/bios.img" "$PROJECT/build_artifacts/uefi.img"
sha256sum "$PROJECT/build_artifacts/bios.img" "$PROJECT/build_artifacts/uefi.img"

echo "--- Extracting UEFI to usb_root ---"
DEST="$PROJECT/build_artifacts/usb_root"
mkdir -p "$DEST" /mnt/uefi_tmp
DEV=$(losetup -Pf --show "$PROJECT/build_artifacts/uefi.img")
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
echo "--- usb_root contents ---"
ls -laR "$DEST"
echo "DONE"
