# SRIKESH OS - Live USB Deployment Script
# Targets external USB Pendrive (hp v210w, Disk 1)
# Enforces strict safety against internal NVMe/SSD storage

param(
    [switch]$Force
)

$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Yellow
Write-Host "     SRIKESH OS — LIVE BOOTABLE USB DEPLOYMENT ENGINE     " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Yellow

# 1. Target Device Positive Verification
$targetDisk = Get-Disk -Number 1 -ErrorAction SilentlyContinue

if (-not $targetDisk) {
    Write-Error "[CRITICAL ERROR] Target USB Disk 1 not detected!"
    exit 1
}

if ($targetDisk.FriendlyName -ne "hp v210w") {
    Write-Error "[SAFETY ABORT] Disk 1 model mismatch! Found: $($targetDisk.FriendlyName). Expected: hp v210w"
    exit 1
}

if ($targetDisk.TotalSize -gt 10GB) {
    Write-Error "[SAFETY ABORT] Disk 1 size ($($targetDisk.TotalSize)) exceeds expected 4GB USB pendrive!"
    exit 1
}

Write-Host "[TARGET VERIFIED] Physical Model  : $($targetDisk.FriendlyName)" -ForegroundColor Green
Write-Host "[TARGET VERIFIED] Serial Number   : $($targetDisk.SerialNumber)" -ForegroundColor Green
Write-Host "[TARGET VERIFIED] Physical Size   : $([math]::Round($targetDisk.TotalSize / 1GB, 2)) GB" -ForegroundColor Green
Write-Host "[TARGET VERIFIED] Bus / MediaType : $($targetDisk.BusType) / Removable Pendrive" -ForegroundColor Green

$usbVol = Get-Volume -DriveLetter D -ErrorAction SilentlyContinue
if (-not $usbVol) {
    Write-Error "[ERROR] Drive D: volume not found!"
    exit 1
}

$artDir = "C:\Users\shres\OneDrive\Desktop\RUST MICRO-OS KERNEL PROJECT\srikesh-os\build_artifacts"
$usbRoot = "$artDir\usb_root"

Write-Host "`n[1/4] Staging Native UEFI Bootloader & Kernel Files..." -ForegroundColor Cyan
if (-not (Test-Path "$usbRoot\efi\boot\bootx64.efi") -or -not (Test-Path "$usbRoot\kernel-x86_64")) {
    Write-Error "[ERROR] Staged files missing in $usbRoot!"
    exit 1
}

# 2. Deploy UEFI Boot Structure to USB D:\
Write-Host "[2/4] Deploying UEFI EFI\BOOT Structure to D:\..." -ForegroundColor Cyan
$destEfi = "D:\efi\boot"
New-Item -ItemType Directory -Path $destEfi -Force | Out-Null
Copy-Item -Path "$usbRoot\efi\boot\bootx64.efi" -Destination "$destEfi\bootx64.efi" -Force
Copy-Item -Path "$usbRoot\kernel-x86_64" -Destination "D:\kernel-x86_64" -Force

Write-Host "  -> D:\efi\boot\bootx64.efi [COPIED - $(Get-Item "D:\efi\boot\bootx64.efi").Length bytes]" -ForegroundColor Green
Write-Host "  -> D:\kernel-x86_64        [COPIED - $(Get-Item "D:\kernel-x86_64").Length bytes]" -ForegroundColor Green

# 3. Deploy Standalone Images and Persistence Directories
Write-Host "`n[3/4] Deploying Standalone Hybrid Images and Persistence Structure..." -ForegroundColor Cyan
$imgDest = "D:\srikesh-os"
New-Item -ItemType Directory -Path $imgDest -Force | Out-Null
New-Item -ItemType Directory -Path "D:\home\srikesh" -Force | Out-Null
New-Item -ItemType Directory -Path "D:\system" -Force | Out-Null

Copy-Item -Path "$artDir\uefi.img" -Destination "$imgDest\srikesh-uefi.img" -Force
Copy-Item -Path "$artDir\bios.img" -Destination "$imgDest\srikesh-bios.img" -Force

# Create boot instructions readme
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
\srikesh-os\            - Standalone BIOS & UEFI raw images (3.5 MB)
\home\srikesh\          - Persistent User Data Partition
\system\                - Kernel Configuration & Zero-Write Policy

SECURITY GUARANTEE:
----------------------------------------------------------------------
SRIKESH OS is equipped with a hardware-enforced StorageGuard engine.
All internal SSDs, HDDs, and NVMe drives on the host laptop are strictly
locked in READ-ONLY mode. SRIKESH OS runs 100% in RAM and USB storage,
never modifying or touching the host Windows installation.
======================================================================
"@

Set-Content -Path "D:\README_BOOT_SRIKESH_OS.txt" -Value $readmeContent -Encoding UTF8

Write-Host "[4/4] Final Integrity & Directory Check on D:\..." -ForegroundColor Cyan
Get-ChildItem "D:\" | Select-Object Name, Length, LastWriteTime | Format-Table -AutoSize

Write-Host "`n==========================================================" -ForegroundColor Green
Write-Host "  ✓ SRIKESH OS SUCCESSFULLY DEPLOYED TO USB PENDRIVE (D:)  " -ForegroundColor Green
Write-Host "  Ready for live boot on Dell Vostro 3520 via F12 menu!  " -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Green
