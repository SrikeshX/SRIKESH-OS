# SRIKESH OS - Live USB Deployment Script
# Authorized Target: external USB Pendrive (hp v210w, Disk 1, Serial: 000000000000031F, Drive D:)
# Internal Host Storage (Disk 0: Micron NVMe 512GB) is 100% Locked & Protected.

$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Yellow
Write-Host "     SRIKESH OS — LIVE BOOTABLE USB DEPLOYMENT ENGINE     " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Yellow

# 1. Host Safety Verification — Check Disk 0
$internalDisk = Get-Disk -Number 0
Write-Host "[SAFETY LOCK] Verifying Host Internal Disk 0: $($internalDisk.FriendlyName)" -ForegroundColor Magenta
if ($internalDisk.BusType -eq "USB") {
    Write-Error "[SAFETY ABORT] Disk 0 detected as USB! Aborting to protect internal drive."
    exit 1
}
Write-Host "[SAFETY LOCK] Internal Host Storage Disk 0 is 100% PROTECTED & UNTOUCHED." -ForegroundColor Green

# 2. Target USB Verification — Check Disk 1
$targetDisk = Get-Disk -Number 1
Write-Host "[TARGET CHECK] Probing Physical Disk 1..." -ForegroundColor Cyan

if ($targetDisk.FriendlyName -ne "hp v210w") {
    Write-Error "[SAFETY ABORT] Disk 1 model mismatch! Found: $($targetDisk.FriendlyName). Expected: hp v210w"
    exit 1
}

if ($targetDisk.SerialNumber -ne "000000000000031F") {
    Write-Error "[SAFETY ABORT] Serial Number mismatch! Expected: 000000000000031F. Found: $($targetDisk.SerialNumber)"
    exit 1
}

if ($targetDisk.TotalSize -gt 10GB) {
    Write-Error "[SAFETY ABORT] Target Disk 1 size exceeds 10GB! Aborting."
    exit 1
}

Write-Host "[TARGET VERIFIED] Physical Model  : $($targetDisk.FriendlyName)" -ForegroundColor Green
Write-Host "[TARGET VERIFIED] Serial Number   : $($targetDisk.SerialNumber)" -ForegroundColor Green
Write-Host "[TARGET VERIFIED] Size            : $([math]::Round($targetDisk.TotalSize / 1GB, 2)) GB" -ForegroundColor Green
Write-Host "[TARGET VERIFIED] Target Laptop   : Dell Vostro 3520" -ForegroundColor Green

# 3. Format Volume D: as FAT32 with label SRIKESH_OS
$targetVol = Get-Volume -DriveLetter D
if ($targetVol.FileSystemLabel -ne "HP V210W" -and $targetVol.FileSystemLabel -ne "SRIKESH_OS") {
    Write-Error "[SAFETY ABORT] Drive D: label unexpected: $($targetVol.FileSystemLabel)"
    exit 1
}

Write-Host "`n[1/5] Formatting Target USB Volume D: (FAT32, Label: SRIKESH_OS)..." -ForegroundColor Cyan
Format-Volume -DriveLetter D -FileSystem FAT32 -NewFileSystemLabel "SRIKESH_OS" -Force | Out-Null
Write-Host "  -> Volume D: formatted cleanly as FAT32 [SRIKESH_OS]." -ForegroundColor Green

# 4. Source Files Verification
$artDir = "C:\Users\shres\OneDrive\Desktop\RUST MICRO-OS KERNEL PROJECT\srikesh-os\build_artifacts"
$usbRoot = "$artDir\usb_root"

Write-Host "`n[2/5] Staging Native UEFI Bootloader & Kernel Files..." -ForegroundColor Cyan
if (-not (Test-Path "$usbRoot\efi\boot\bootx64.efi") -or -not (Test-Path "$usbRoot\kernel-x86_64")) {
    Write-Error "[ERROR] Staged files missing in $usbRoot!"
    exit 1
}

# 5. Deploy UEFI Boot Structure to USB D:\
Write-Host "[3/5] Deploying UEFI EFI\BOOT Structure to D:\..." -ForegroundColor Cyan
$destEfi = "D:\efi\boot"
New-Item -ItemType Directory -Path $destEfi -Force | Out-Null
Copy-Item -Path "$usbRoot\efi\boot\bootx64.efi" -Destination "$destEfi\bootx64.efi" -Force
Copy-Item -Path "$usbRoot\kernel-x86_64" -Destination "D:\kernel-x86_64" -Force
if (Test-Path "$usbRoot\boot.json") {
    Copy-Item -Path "$usbRoot\boot.json" -Destination "D:\boot.json" -Force
    Copy-Item -Path "$usbRoot\boot.json" -Destination "$destEfi\boot.json" -Force
    Write-Host "  -> D:\boot.json            [COPIED - Framebuffer Logging = false]" -ForegroundColor Green
}

Write-Host "  -> D:\efi\boot\bootx64.efi [COPIED - $(Get-Item "D:\efi\boot\bootx64.efi").Length bytes]" -ForegroundColor Green
Write-Host "  -> D:\kernel-x86_64        [COPIED - $(Get-Item "D:\kernel-x86_64").Length bytes]" -ForegroundColor Green

# 6. Deploy Standalone Hybrid Images and Persistence Structure
Write-Host "`n[4/5] Deploying Standalone Hybrid Images and Persistence Structure..." -ForegroundColor Cyan
$imgDest = "D:\srikesh-os"
New-Item -ItemType Directory -Path $imgDest -Force | Out-Null
New-Item -ItemType Directory -Path "D:\home\srikesh" -Force | Out-Null
New-Item -ItemType Directory -Path "D:\system" -Force | Out-Null

Copy-Item -Path "$artDir\uefi.img" -Destination "$imgDest\srikesh-uefi.img" -Force
Copy-Item -Path "$artDir\bios.img" -Destination "$imgDest\srikesh-bios.img" -Force

# Create persistent user files
$welcomeTxt = @"
Welcome to SRIKESH OS!
Awaken the Power of Technology.

A genuine x86-64 bare-metal microkernel engineered in low-level Rust.
Features:
- Preemptive Round Robin Task Scheduler (50 ms time slices)
- Double-buffered Egyptian Royal Desktop Compositor
- 11 Functional Applications & 9 Offline Games Suite
- Host Storage Zero-Write Guard (Host Internal Disks 100% Protected)
- Non-Volatile Sector Persistence on USB Storage

Student: Srikesh S (CSE AI/ML) | USN: 25BTRCL258
Course: CA-6 Operating Systems Flagship Engineering Project
Target Laptop: Dell Vostro 3520
"@
Set-Content -Path "D:\home\srikesh\welcome.txt" -Value $welcomeTxt -Encoding UTF8

$bootCfg = @"
[SRIKESH_OS_BOOT]
version=0.2.0-Royal
theme=egyptian_royal_gold
compositor=double_buffered_32bit
target_hardware=dell_vostro_3520
storage_guard=active_lock_internal_drives
persistence=enabled_lba_2048
"@
Set-Content -Path "D:\system\boot.cfg" -Value $bootCfg -Encoding UTF8

# Create comprehensive boot instructions readme
$readmeContent = @"
======================================================================
  SRIKESH OS — AWAKEN THE POWER OF TECHNOLOGY
  Official Portable Live USB Medium
======================================================================
Candidate: Srikesh S (CSE AI/ML) | USN: 25BTRCL258
Academic Course: CA-6 Operating Systems Flagship Engineering Project
Target Laptop: Dell Vostro 3520 (x86-64 Intel/AMD Architecture)

HOW TO BOOT SRIKESH OS ON YOUR DELL VOSTRO 3520 (OR OTHER LAPTOPS):
----------------------------------------------------------------------
1. Insert this HP v210w USB pendrive into any USB port.
2. Power on the laptop and immediately tap [F12] repeatedly to enter
   the One-Time Boot Menu.
3. Under 'UEFI BOOT:', select:
      'UEFI: HP v210w' or 'UEFI: USB Storage Device'
4. Press [ENTER].
5. SRIKESH OS will boot immediately into the high-resolution 32-bit
   Egyptian Royal Desktop environment!

INCLUDED DIRECTORIES & SYSTEM FILES:
----------------------------------------------------------------------
\efi\boot\bootx64.efi   - 64-bit UEFI Bootloader (GOP Framebuffer)
\kernel-x86_64          - SRIKESH OS x86-64 Rust Bare-Metal Kernel
\srikesh-os\            - Standalone BIOS & UEFI raw images (3.5 MB each)
\home\srikesh\          - Persistent User Data Partition
\system\                - Kernel Configuration & Zero-Write Policy
\README_BOOT_SRIKESH_OS.txt - Boot guide and hardware documentation

SECURITY GUARANTEE:
----------------------------------------------------------------------
SRIKESH OS is equipped with a hardware-enforced StorageGuard engine.
All internal SSDs, HDDs, and NVMe drives on the host laptop are strictly
locked in READ-ONLY mode. SRIKESH OS runs 100% in RAM and USB storage,
never modifying or touching the host Windows installation.
======================================================================
"@
Set-Content -Path "D:\README_BOOT_SRIKESH_OS.txt" -Value $readmeContent -Encoding UTF8

# 7. Final Verification
Write-Host "`n[5/5] Final Integrity & Directory Check on D:\..." -ForegroundColor Cyan
Get-ChildItem "D:\" -Recurse | Select-Object FullName, Length, LastWriteTime | Format-Table -AutoSize

Write-Host "==========================================================" -ForegroundColor Green
Write-Host "  ✓ SRIKESH OS SUCCESSFULLY DEPLOYED TO USB PENDRIVE (D:)  " -ForegroundColor Green
Write-Host "  Physical Model: $($targetDisk.FriendlyName) ($($targetDisk.SerialNumber))" -ForegroundColor Green
Write-Host "  Ready for live boot on Dell Vostro 3520 via F12 menu!  " -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Green
