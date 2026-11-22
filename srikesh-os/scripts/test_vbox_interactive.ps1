$ErrorActionPreference = "Continue"
$vbox = "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe"
$vm = "SRIKESH OS"
$proj = "C:\Users\shres\OneDrive\Desktop\RUST MICRO-OS KERNEL PROJECT\srikesh-os"
$art = "$proj\build_artifacts"
$brain = "C:\Users\shres\.gemini\antigravity\brain\43e9f22a-f7d1-4d29-a088-e60efad3b868"
$vdi = "$art\srikesh-uefi.vdi"
$uefi = "$art\uefi.img"
$uuid = "25b4b139-4058-4f3f-9922-216ac39fe5f6"
$serial = "$art\vbox_serial_uefi.log"

Write-Host "=== SRIKESH OS — Full Interactive VirtualBox Verification ==="
& $vbox controlvm $vm poweroff 2>$null | Out-Null
Start-Sleep -Seconds 2

if (Test-Path $vdi) { Remove-Item $vdi -Force -ErrorAction SilentlyContinue }
& $vbox convertfromraw $uefi $vdi --format VDI
& $vbox internalcommands sethduuid $vdi $uuid
if (Test-Path $serial) { Clear-Content $serial -ErrorAction SilentlyContinue }

Write-Host "[1] Starting VM headless..."
& $vbox startvm $vm --type headless
Start-Sleep -Seconds 18

Write-Host "[2] Capturing Clean Desktop..."
& $vbox controlvm $vm screenshotpng "$brain\vbox_final_01_desktop.png"
Start-Sleep -Seconds 1

Write-Host "[3] Testing Keyboard F11 -> App Launcher..."
& $vbox controlvm $vm keyboardputscancode 57 d7
Start-Sleep -Seconds 2
& $vbox controlvm $vm screenshotpng "$brain\vbox_final_02_launcher.png"

Write-Host "[4] Testing Keyboard F1 -> File Manager..."
& $vbox controlvm $vm keyboardputscancode 3b bb
Start-Sleep -Seconds 2
& $vbox controlvm $vm screenshotpng "$brain\vbox_final_03_files.png"

Write-Host "[5] Testing Multi-Window: Opening Settings (F4) over File Manager..."
& $vbox controlvm $vm keyboardputscancode 3e be
Start-Sleep -Seconds 2
& $vbox controlvm $vm screenshotpng "$brain\vbox_final_04_multiwindow.png"

Write-Host "[6] Testing Calculator (F7) + Live Math (7 + 5 =)..."
& $vbox controlvm $vm keyboardputscancode 41 c1
Start-Sleep -Seconds 1
# '7'=0x08, '+'=shift+0x0D (2a 0d 8d aa), '5'=0x06, '='=0x0D
& $vbox controlvm $vm keyboardputscancode 08 88
Start-Sleep -Milliseconds 400
& $vbox controlvm $vm keyboardputscancode 2a 0d 8d aa
Start-Sleep -Milliseconds 400
& $vbox controlvm $vm keyboardputscancode 06 86
Start-Sleep -Milliseconds 400
& $vbox controlvm $vm keyboardputscancode 0d 8d
Start-Sleep -Seconds 1
& $vbox controlvm $vm screenshotpng "$brain\vbox_final_05_calculator.png"

Write-Host "[7] Testing Network Manager (F8) — Honest Hardware Status..."
& $vbox controlvm $vm keyboardputscancode 42 c2
Start-Sleep -Seconds 2
& $vbox controlvm $vm screenshotpng "$brain\vbox_final_06_network.png"

Write-Host "[8] Testing System Monitor (F5) — Live Scheduler & Context Switches..."
& $vbox controlvm $vm keyboardputscancode 3f bf
Start-Sleep -Seconds 2
& $vbox controlvm $vm screenshotpng "$brain\vbox_final_07_sysmon.png"

Write-Host "[9] Powering off VM..."
& $vbox controlvm $vm poweroff
Start-Sleep -Seconds 2

Write-Host "`n===== Serial Log Summary (last 35 lines) ====="
if (Test-Path $serial) {
    Get-Content $serial -Tail 35
}
Write-Host "===== DONE ====="
