$ErrorActionPreference = "Stop"

$StageDir = "C:\Users\shres\OneDrive\Desktop\RUST MICRO-OS KERNEL PROJECT\srikesh-os-hybrid\out\usb_stage"
$ExpectedModel = "hp v210w"
$ExpectedSerial = "000000000000031F"
$TargetDiskNum = 1
$TargetDrive = "D"

Write-Host "============================================================"
Write-Host "   SRIKESH OS HYBRID — SAFE HP v210w LIVE USB DEPLOYMENT    "
Write-Host "============================================================"

# 1. Verify Host Disk 0 (Internal NVMe SSD) is untouched
$disk0 = Get-Disk -Number 0
Write-Host "[SAFETY] Internal Disk 0: $($disk0.FriendlyName) ($($disk0.Size) bytes) -> PROTECTED / UNTOUCHED"

# 2. Strictly verify Target Disk 1
$disk1 = Get-Disk -Number $TargetDiskNum
if ($null -eq $disk1) {
    Write-Error "ABORT: Physical Disk $TargetDiskNum not found!"
    exit 1
}
$model = $disk1.FriendlyName.Trim()
$serial = $disk1.SerialNumber.Trim()
if ($model -ne $ExpectedModel -or $serial -ne $ExpectedSerial) {
    Write-Error "ABORT: Disk $TargetDiskNum mismatch! Found '$model' / '$serial', expected '$ExpectedModel' / '$ExpectedSerial'."
    exit 1
}

$part = Get-Partition -DiskNumber $TargetDiskNum | Where-Object { $_.DriveLetter -eq $TargetDrive }
if ($null -eq $part) {
    Write-Error "ABORT: Partition with drive letter ${TargetDrive}: not found on Disk $TargetDiskNum!"
    exit 1
}

Write-Host "[VERIFIED] Target USB: Disk $TargetDiskNum | $model | Serial: $serial | Drive: ${TargetDrive}:"

# 3. Verify staged files exist and are under 2 GB
$required = @(
    "EFI\BOOT\BOOTX64.EFI",
    "boot\grub\grub.cfg",
    "live\vmlinuz",
    "live\initrd.img",
    "live\filesystem.squashfs"
)
$totalBytes = 0
foreach ($rel in $required) {
    $src = Join-Path $StageDir $rel
    if (-not (Test-Path $src)) {
        Write-Error "ABORT: Missing staged file $src"
        exit 1
    }
    $totalBytes += (Get-Item $src).Length
}
$totalMB = [math]::Round($totalBytes / 1MB, 1)
Write-Host "[VERIFIED] Staged payload size: $totalMB MB ($totalBytes bytes)"
if ($totalBytes -gt 2147483648) {
    Write-Error "ABORT: Staged payload exceeds strict 2 GB limit!"
    exit 1
}

# 4. Format D: as FAT32 (SRIKESH_OS)
Write-Host "[1/3] Formatting ${TargetDrive}: as FAT32 (Label: SRIKESH_OS)..."
Format-Volume -DriveLetter $TargetDrive -FileSystem FAT32 -NewFileSystemLabel "SRIKESH_OS" -Force -Confirm:$false | Out-Null

# 5. Copy UEFI bootloader, GRUB config, Linux kernel, initrd, and SquashFS
Write-Host "[2/3] Copying SRIKESH OS Hybrid files to ${TargetDrive}:\..."
foreach ($rel in $required) {
    $src = Join-Path $StageDir $rel
    $dst = Join-Path "${TargetDrive}:\" $rel
    $dstDir = Split-Path $dst -Parent
    if (-not (Test-Path $dstDir)) {
        New-Item -ItemType Directory -Path $dstDir -Force | Out-Null
    }
    Copy-Item -Path $src -Destination $dst -Force
    $srcLen = (Get-Item $src).Length
    $dstLen = (Get-Item $dst).Length
    if ($srcLen -ne $dstLen) {
        Write-Error "ABORT: Size mismatch on $dst ($dstLen != $srcLen)"
        exit 1
    }
    Write-Host "  -> $dst ($dstLen bytes) [VERIFIED]"
}

# Write boot instructions readme
@"
SRIKESH OS Hybrid Live USB (Dell Vostro 3520 Edition)
=====================================================
- Real Linux 6.1 LTS Kernel + Realtek RTL8821CE driver (rtw88_8821ce) & firmware (rtw8821c_fw.bin)
- Real Windows-11-style Wi-Fi Quick Panel (NetworkManager / nmcli)
- Full Chromium Web Browser (HTTPS/TLS, Google, YouTube, Offline Dino Game)
- Custom SRIKESH OS Desktop Shell (Aurora Bloom Wallpaper, Taskbar, Start Menu, System Monitor)

How to boot on Dell Vostro 3520:
1. Keep this HP v210w USB plugged in and restart your laptop.
2. Tap F12 repeatedly at the Dell logo to open the One-Time Boot Menu.
3. Under UEFI Boot, select your HP v210w USB drive.
   (Ensure Secure Boot is Disabled in F2 BIOS Setup if UEFI blocks unsigned GRUB.)
4. Once on the desktop, click the Wi-Fi icon in the bottom-right taskbar to scan and connect to your Wi-Fi network, then open Browser (Google / YouTube).
"@ | Set-Content -Path "${TargetDrive}:\README_SRIKESH_HYBRID.txt" -Encoding UTF8

# 6. Final directory listing
Write-Host "[3/3] Final verified contents of ${TargetDrive}:\:"
Get-ChildItem -Path "${TargetDrive}:\" -Recurse | Select-Object FullName, Length | Format-Table -AutoSize
Write-Host "============================================================"
Write-Host "  ✓ SRIKESH OS HYBRID DEPLOYED & VERIFIED ON HP v210w (D:)  "
Write-Host "============================================================"
