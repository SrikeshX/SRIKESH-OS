#!/usr/bin/env bash
set -euo pipefail

WIN_PROJECT="/mnt/c/Users/shres/OneDrive/Desktop/RUST MICRO-OS KERNEL PROJECT/srikesh-os-hybrid"
BUILD_ROOT="/root/srikesh-hybrid-build"
CHROOT="$BUILD_ROOT/chroot"
ISO_DIR="$BUILD_ROOT/iso"
OUT_DIR="$WIN_PROJECT/out"

echo "============================================================"
echo "   SRIKESH OS HYBRID — REAL LIVE USB & ISO BUILD ENGINE     "
echo "============================================================"

rm -rf "$BUILD_ROOT"
mkdir -p "$CHROOT" "$ISO_DIR/live" "$ISO_DIR/boot/grub" "$OUT_DIR"

echo "[1/7] Bootstrapping minimal Debian 12 (Bookworm) rootfs..."
debootstrap --arch=amd64 --variant=minbase bookworm "$CHROOT" http://deb.debian.org/debian

echo "[2/7] Configuring APT repositories (main + non-free-firmware)..."
cat > "$CHROOT/etc/apt/sources.list" << 'EOF'
deb http://deb.debian.org/debian bookworm main contrib non-free non-free-firmware
deb http://deb.debian.org/debian bookworm-updates main contrib non-free non-free-firmware
deb http://security.debian.org/debian-security bookworm-security main contrib non-free non-free-firmware
EOF

echo "srikesh-os" > "$CHROOT/etc/hostname"
cat > "$CHROOT/etc/hosts" << 'EOF'
127.0.0.1 localhost srikesh-os
::1       localhost
EOF

# Mount virtual filesystems for chroot
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

echo "[3/7] Installing Linux 6.1 LTS, RTL8821CE firmware, NetworkManager, Xorg, Openbox, Chromium..."
chroot "$CHROOT" /bin/bash -c '
set -euo pipefail
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y --no-install-recommends \
    linux-image-amd64 \
    live-boot \
    systemd-sysv \
    dbus \
    udev \
    kmod \
    sudo \
    pciutils \
    usbutils \
    rfkill \
    iw \
    wireless-tools \
    firmware-realtek \
    firmware-misc-nonfree \
    firmware-sof-signed \
    network-manager \
    wpasupplicant \
    iproute2 \
    iputils-ping \
    ca-certificates \
    curl \
    xserver-xorg-core \
    xserver-xorg-video-fbdev \
    xserver-xorg-video-vesa \
    xserver-xorg-input-libinput \
    xinit \
    x11-xserver-utils \
    openbox \
    feh \
    fonts-dejavu-core \
    fonts-liberation \
    alsa-utils \
    pulseaudio \
    chromium \
    python3 \
    python3-gi \
    gir1.2-gtk-3.0 \
    pcmanfm \
    lxterminal \
    libgtk-3-0
'

echo "[4/7] Verifying Realtek RTL8821CE driver & firmware inside chroot..."
KVER=$(ls "$CHROOT/lib/modules" | head -n 1)
echo "  -> Installed Kernel: $KVER"
if ! test -f "$CHROOT/lib/firmware/rtw88/rtw8821c_fw.bin"; then
    echo "FATAL: /lib/firmware/rtw88/rtw8821c_fw.bin is MISSING!"
    exit 1
fi
if ! find "$CHROOT/lib/modules/$KVER" -name "*rtw88_8821ce*" | grep -q rtw88_8821ce; then
    echo "FATAL: rtw88_8821ce kernel module is MISSING!"
    exit 1
fi
echo "  -> VERIFIED: rtw88_8821ce.ko + /lib/firmware/rtw88/rtw8821c_fw.bin present!"

echo "[5/7] Installing SRIKESH OS Shell, Wallpaper, Dell Vostro 3520 tweaks, and Auto-Login..."
mkdir -p "$CHROOT/opt/srikesh"
cp "$WIN_PROJECT/shell/srikesh_shell.py" "$CHROOT/opt/srikesh/srikesh_shell.py"
cp "$WIN_PROJECT/shell/srikesh_sysmon.py" "$CHROOT/opt/srikesh/srikesh_sysmon.py"
chmod +x "$CHROOT/opt/srikesh/srikesh_shell.py" "$CHROOT/opt/srikesh/srikesh_sysmon.py"
python3 "$WIN_PROJECT/shell/generate_wallpaper.py" "$CHROOT/opt/srikesh/aurora_bloom.png"

# Realtek RTL8821CE stability fix for Dell Vostro 3520 (disable PCIe ASPM & deep LPS)
mkdir -p "$CHROOT/etc/modprobe.d"
cat > "$CHROOT/etc/modprobe.d/rtw88_8821ce.conf" << 'EOF'
options rtw88_pci disable_aspm=y
options rtw88_core disable_lps_deep=y
EOF

# NetworkManager full control
mkdir -p "$CHROOT/etc/NetworkManager"
cat > "$CHROOT/etc/NetworkManager/NetworkManager.conf" << 'EOF'
[main]
plugins=ifupdown,keyfile

[ifupdown]
managed=true
EOF

# Dell Vostro 3520 I2C Touchpad tap-to-click & natural scroll
mkdir -p "$CHROOT/etc/X11/xorg.conf.d"
cat > "$CHROOT/etc/X11/xorg.conf.d/40-libinput.conf" << 'EOF'
Section "InputClass"
    Identifier "libinput touchpad catchall"
    MatchIsTouchpad "on"
    MatchDevicePath "/dev/input/event*"
    Driver "libinput"
    Option "Tapping" "on"
    Option "NaturalScrolling" "true"
    Option "DisableWhileTyping" "true"
EndSection
EOF

# Create user srikesh + auto-login on tty1 + X11 session
chroot "$CHROOT" /bin/bash -c '
set -euo pipefail
useradd -m -s /bin/bash -G sudo,netdev,audio,video,plugdev srikesh || true
echo "srikesh:srikesh" | chpasswd
echo "root:srikesh" | chpasswd
echo "srikesh ALL=(ALL) NOPASSWD: ALL" > /etc/sudoers.d/srikesh
chmod 0440 /etc/sudoers.d/srikesh
systemctl enable NetworkManager || true
'

mkdir -p "$CHROOT/etc/systemd/system/getty@tty1.service.d"
cat > "$CHROOT/etc/systemd/system/getty@tty1.service.d/autologin.conf" << 'EOF'
[Service]
ExecStart=
ExecStart=-/sbin/agetty --autologin srikesh --noclear %I $TERM
EOF

# Openbox config with keyboard shortcuts
mkdir -p "$CHROOT/home/srikesh/.config/openbox"
cat > "$CHROOT/home/srikesh/.config/openbox/rc.xml" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<openbox_config xmlns="http://openbox.org/3.4/rc">
  <desktops><number>1</number></desktops>
  <theme>
    <name>Nightmare</name>
    <titleLayout>NLIMC</titleLayout>
    <keepBorder>yes</keepBorder>
  </theme>
  <keyboard>
    <keybind key="A-F4"><action name="Close"/></keybind>
    <keybind key="A-Tab"><action name="NextWindow"/></keybind>
    <keybind key="F1"><action name="Execute"><command>pcmanfm</command></action></keybind>
    <keybind key="F2"><action name="Execute"><command>lxterminal</command></action></keybind>
    <keybind key="F3"><action name="Execute"><command>chromium --no-sandbox --no-first-run https://www.google.com</command></action></keybind>
    <keybind key="F5"><action name="Execute"><command>python3 /opt/srikesh/srikesh_sysmon.py</command></action></keybind>
  </keyboard>
</openbox_config>
EOF

cat > "$CHROOT/home/srikesh/.xinitrc" << 'EOF'
#!/bin/sh
xsetroot -solid "#0F172A"
xrandr --auto 2>/dev/null || true
feh --bg-fill /opt/srikesh/aurora_bloom.png &
pulseaudio --start 2>/dev/null || true
openbox-session &
sleep 0.6
exec python3 /opt/srikesh/srikesh_shell.py
EOF
chmod +x "$CHROOT/home/srikesh/.xinitrc"

cat > "$CHROOT/home/srikesh/.bash_profile" << 'EOF'
if [ -z "$DISPLAY" ] && [ "$(tty)" = "/dev/tty1" ]; then
    exec startx -- -keeptty > /tmp/xorg.log 2>&1
fi
EOF
chroot "$CHROOT" chown -R srikesh:srikesh /home/srikesh

echo "[6/7] Stripping unnecessary docs/caches & compressing SquashFS (zstd)..."
chroot "$CHROOT" /bin/bash -c '
apt-get clean
rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/* \
       /usr/share/doc/* /usr/share/man/* /usr/share/info/* /usr/share/lintian/*
'

# Copy kernel & initrd before unmounting
cp "$CHROOT/boot/vmlinuz-"* "$ISO_DIR/live/vmlinuz"
cp "$CHROOT/boot/initrd.img-"* "$ISO_DIR/live/initrd.img"

cleanup_mounts
trap - EXIT

mksquashfs "$CHROOT" "$ISO_DIR/live/filesystem.squashfs" -comp zstd -Xcompression-level 15 -noappend -quiet

SQUASH_BYTES=$(stat -c%s "$ISO_DIR/live/filesystem.squashfs")
SQUASH_MB=$((SQUASH_BYTES / 1024 / 1024))
echo "  -> filesystem.squashfs size: ${SQUASH_MB} MB ($SQUASH_BYTES bytes)"

echo "[7/7] Building UEFI + BIOS Hybrid Bootable ISO & Standalone UEFI Bootloader..."
cat > "$ISO_DIR/boot/grub/grub.cfg" << 'EOF'
set default="0"
set timeout="2"

menuentry "SRIKESH OS Hybrid (Modern Desktop + Real Wi-Fi + Chromium)" {
    search --no-floppy --file --set=root /live/vmlinuz
    linux /live/vmlinuz boot=live components quiet
    initrd /live/initrd.img
}

menuentry "SRIKESH OS Hybrid (Safe Graphics / nomodeset)" {
    search --no-floppy --file --set=root /live/vmlinuz
    linux /live/vmlinuz boot=live components nomodeset
    initrd /live/initrd.img
}
EOF

# Build standalone UEFI BOOTX64.EFI for direct FAT32 USB deployment
mkdir -p "$OUT_DIR/usb_stage/EFI/BOOT" "$OUT_DIR/usb_stage/boot/grub" "$OUT_DIR/usb_stage/live"
grub-mkstandalone \
    --format=x86_64-efi \
    --output="$OUT_DIR/usb_stage/EFI/BOOT/BOOTX64.EFI" \
    --locales="" \
    --fonts="" \
    "boot/grub/grub.cfg=$ISO_DIR/boot/grub/grub.cfg"

cp "$ISO_DIR/boot/grub/grub.cfg" "$OUT_DIR/usb_stage/boot/grub/grub.cfg"
cp "$ISO_DIR/live/vmlinuz" "$OUT_DIR/usb_stage/live/vmlinuz"
cp "$ISO_DIR/live/initrd.img" "$OUT_DIR/usb_stage/live/initrd.img"
cp "$ISO_DIR/live/filesystem.squashfs" "$OUT_DIR/usb_stage/live/filesystem.squashfs"

# Build bootable ISO for Oracle VirtualBox testing
grub-mkrescue -o "$OUT_DIR/srikesh-os-hybrid.iso" "$ISO_DIR" -- -volid SRIKESH_OS

ISO_BYTES=$(stat -c%s "$OUT_DIR/srikesh-os-hybrid.iso")
ISO_MB=$((ISO_BYTES / 1024 / 1024))
echo "============================================================"
echo "  ✓ BUILD COMPLETE!"
echo "  ISO Size      : ${ISO_MB} MB ($ISO_BYTES bytes)"
echo "  SquashFS Size : ${SQUASH_MB} MB"
echo "  Output ISO    : $OUT_DIR/srikesh-os-hybrid.iso"
echo "  USB Stage     : $OUT_DIR/usb_stage/"
echo "============================================================"

if [ "$ISO_BYTES" -gt 2147483648 ]; then
    echo "FATAL: ISO exceeds strict 2 GB maximum limit!"
    exit 1
fi
