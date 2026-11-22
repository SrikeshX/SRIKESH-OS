# VirtualBox Full Run to Desktop Verification
$vbox = "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe"
$artDir = "C:\Users\shres\OneDrive\Desktop\RUST MICRO-OS KERNEL PROJECT\srikesh-os\build_artifacts"
$vmName = "SRIKESH OS"

Write-Host "=========================================================="
Write-Host "  ORACLE VIRTUALBOX 7.2.22 — FULL DESKTOP RUN VERIFICATION "
Write-Host "=========================================================="

Write-Host "[1/3] Starting '$vmName' in VirtualBox (headless)..."
& $vbox startvm $vmName --type headless

# Sleep 15 seconds to allow full startup animation + PIT calibration + desktop transition
Write-Host "[2/3] Waiting 15 seconds for startup sequence and desktop compositor..."
Start-Sleep -Seconds 15

# Capture Desktop Screenshot
$screenDesktop = "$artDir\vbox_desktop_verified.png"
Write-Host "[3/3] Capturing VirtualBox desktop screenshot: $screenDesktop..."
& $vbox controlvm $vmName screenshotpng $screenDesktop
Start-Sleep -Seconds 1
& $vbox controlvm $vmName poweroff
Start-Sleep -Seconds 2

Write-Host "Verification Complete. Recent Serial Log Output:"
Get-Content "$artDir\vbox_serial_uefi.log" -Tail 35
