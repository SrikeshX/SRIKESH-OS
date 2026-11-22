$ErrorActionPreference = "Stop"

$VBox = "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe"
$VmName = "SRIKESH_OS_Hybrid_Test"
$IsoPath = "C:\Users\shres\OneDrive\Desktop\RUST MICRO-OS KERNEL PROJECT\srikesh-os-hybrid\out\srikesh-os-hybrid.iso"
$ArtDir = "C:\Users\shres\.gemini\antigravity\brain\43e9f22a-f7d1-4d29-a088-e60efad3b868"

if (-not (Test-Path $IsoPath)) {
    Write-Error "ISO not found at $IsoPath"
    exit 1
}

Write-Host "[VBOX] Cleaning up any existing VM $VmName..."
$ErrorActionPreference = "Continue"
& $VBox controlvm $VmName poweroff 2>&1 | Out-Null
Start-Sleep -Seconds 2
& $VBox unregistervm $VmName --delete 2>&1 | Out-Null
$ErrorActionPreference = "Stop"

Write-Host "[VBOX] Creating UEFI VM $VmName (2 GB RAM, 2 vCPU, NAT Internet)..."
& $VBox createvm --name $VmName --ostype Debian_64 --register | Out-Null
& $VBox modifyvm $VmName `
    --memory 2048 `
    --cpus 2 `
    --vram 64 `
    --graphicscontroller vmsvga `
    --firmware efi `
    --nic1 nat `
    --nictype1 82540EM `
    --audio-driver none `
    --mouse ps2 `
    --keyboard ps2 `
    --boot1 dvd --boot2 none --boot3 none --boot4 none

& $VBox storagectl $VmName --name "SATA" --add sata --controller IntelAhci | Out-Null
& $VBox storageattach $VmName --storagectl "SATA" --port 0 --device 0 --type dvddrive --medium $IsoPath

Write-Host "[VBOX] Booting SRIKESH OS Hybrid Live ISO (UEFI)..."
& $VBox startvm $VmName --type headless | Out-Null

# Wait for GRUB (2s) + Linux 6.1 LTS + systemd + Xorg + SRIKESH Shell
Start-Sleep -Seconds 55
& $VBox controlvm $VmName screenshotpng "$ArtDir\hybrid_01_desktop.png"
Write-Host "[VBOX] Captured hybrid_01_desktop.png"

# Press F4 -> Open Windows-11-style Wi-Fi Quick Panel
& $VBox controlvm $VmName keyboardputscancode 3e be
Start-Sleep -Seconds 4
& $VBox controlvm $VmName screenshotpng "$ArtDir\hybrid_02_wifi_panel.png"
Write-Host "[VBOX] Captured hybrid_02_wifi_panel.png"

# Press Escape to close Wi-Fi Panel
& $VBox controlvm $VmName keyboardputscancode 01 81
Start-Sleep -Seconds 1

# Press F5 -> Launch System & Hardware Monitor (verifies rtw88_8821ce + firmware + NetworkManager)
& $VBox controlvm $VmName keyboardputscancode 3f bf
Start-Sleep -Seconds 5
& $VBox controlvm $VmName screenshotpng "$ArtDir\hybrid_05_sysmon.png"
Write-Host "[VBOX] Captured hybrid_05_sysmon.png"

# Close SysMon (Alt+F4)
& $VBox controlvm $VmName keyboardputscancode 38 3e be b8
Start-Sleep -Seconds 2

# Press F3 -> Launch Chromium to https://www.google.com
& $VBox controlvm $VmName keyboardputscancode 3d bd
Start-Sleep -Seconds 18
& $VBox controlvm $VmName screenshotpng "$ArtDir\hybrid_03_google.png"
Write-Host "[VBOX] Captured hybrid_03_google.png"

# Focus Omnibox (Ctrl+L), type https://www.youtube.com and press Enter
& $VBox controlvm $VmName keyboardputscancode 1d 26 a6 9d
Start-Sleep -Milliseconds 600
& $VBox controlvm $VmName keyboardputstring "https://www.youtube.com"
Start-Sleep -Milliseconds 400
& $VBox controlvm $VmName keyboardputscancode 1c 9c
Start-Sleep -Seconds 18
& $VBox controlvm $VmName screenshotpng "$ArtDir\hybrid_04_youtube.png"
Write-Host "[VBOX] Captured hybrid_04_youtube.png"

# Power off cleanly
& $VBox controlvm $VmName poweroff | Out-Null
Start-Sleep -Seconds 2
Write-Host "[VBOX] Verification complete!"
