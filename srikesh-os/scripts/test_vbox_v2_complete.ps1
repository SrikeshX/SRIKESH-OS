$ErrorActionPreference = "Continue"
$vbox = "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe"
$vmName = "SRIKESH OS"
$proj = "C:\Users\shres\OneDrive\Desktop\RUST MICRO-OS KERNEL PROJECT\srikesh-os"
$art = "$proj\build_artifacts"
$brain = "C:\Users\shres\.gemini\antigravity\brain\43e9f22a-f7d1-4d29-a088-e60efad3b868"
$vdi = "$art\srikesh-uefi.vdi"
$uefi = "$art\uefi.img"
$uuid = "25b4b139-4058-4f3f-9922-216ac39fe5f6"
$serial = "$art\vbox_serial_uefi.log"

& $vbox controlvm $vmName poweroff 2>$null | Out-Null
Start-Sleep -Seconds 2
if (Test-Path $vdi) { Remove-Item $vdi -Force -ErrorAction SilentlyContinue }
& $vbox convertfromraw $uefi $vdi --format VDI
& $vbox internalcommands sethduuid $vdi $uuid
if (Test-Path $serial) { Clear-Content $serial -ErrorAction SilentlyContinue }

Write-Host "[1] Starting VM headless..."
& $vbox startvm $vmName --type headless
Start-Sleep -Seconds 18

$vb = New-Object -ComObject VirtualBox.VirtualBox
$sess = New-Object -ComObject VirtualBox.Session
$m = $vb.FindMachine($vmName)
$m.LockMachine($sess, 1)
$mouse = $sess.Console.Mouse

# 1. Move mouse smoothly to System Tray (880, 744) from (512, 384) -> exercises zero-lag cursor blit!
Write-Host "[2] Moving cursor to System Tray (exercising fast dirty-rect cursor blit)..."
for ($i = 0; $i -lt 18; $i++) {
    $mouse.PutMouseEvent(20, 20, 0, 0, 0)
    Start-Sleep -Milliseconds 30
}
Start-Sleep -Milliseconds 400
# Click System Tray -> opens Quick Settings!
$mouse.PutMouseEvent(0, 0, 0, 0, 1)
Start-Sleep -Milliseconds 120
$mouse.PutMouseEvent(0, 0, 0, 0, 0)
Start-Sleep -Seconds 2
& $vbox controlvm $vmName screenshotpng "$brain\vbox_v2_01_quick_settings.png"

# 2. Close Quick Settings (ESC), Open Terminal (F2), type "info" + Enter!
Write-Host "[3] Testing Terminal: F2 -> type 'info' + Enter..."
& $vbox controlvm $vmName keyboardputscancode 01 81
Start-Sleep -Milliseconds 400
& $vbox controlvm $vmName keyboardputscancode 3c bc
Start-Sleep -Seconds 1
# i=0x17, n=0x31, f=0x21, o=0x18, Enter=0x1C
& $vbox controlvm $vmName keyboardputscancode 17 97 31 b1 21 a1 18 98 1c 9c
Start-Sleep -Seconds 2
& $vbox controlvm $vmName screenshotpng "$brain\vbox_v2_02_terminal_cmd.png"

# 3. Open Browser (F3), Ctrl+A, type "youtube.com" + Enter!
Write-Host "[4] Testing Browser: F3 -> Ctrl+A -> type 'youtube.com' + Enter..."
& $vbox controlvm $vmName keyboardputscancode 3d bd
Start-Sleep -Seconds 1
# Ctrl+A: 1d 1e 9e 9d
& $vbox controlvm $vmName keyboardputscancode 1d 1e 9e 9d
Start-Sleep -Milliseconds 300
# y=15, o=18, u=16, t=14, u=16, b=30, e=12, .=34, c=2e, o=18, m=32, Enter=1c
& $vbox controlvm $vmName keyboardputscancode 15 95 18 98 16 96 14 94 16 96 30 b0 12 92 34 b4 2e ae 18 98 32 b2 1c 9c
Start-Sleep -Seconds 2
& $vbox controlvm $vmName screenshotpng "$brain\vbox_v2_03_browser_yt.png"

# 4. Open System Monitor (F5) -> capture live Full Frame Cycles vs Cursor Blit Cycles!
Write-Host "[5] Capturing System Monitor with live Full-Frame vs Cursor-Blit TSC telemetry..."
& $vbox controlvm $vmName keyboardputscancode 3f bf
Start-Sleep -Seconds 2
& $vbox controlvm $vmName screenshotpng "$brain\vbox_v2_04_sysmon_perf.png"

$sess.UnlockMachine()
& $vbox controlvm $vmName poweroff
Start-Sleep -Seconds 2
Write-Host "===== COMPLETE ====="
