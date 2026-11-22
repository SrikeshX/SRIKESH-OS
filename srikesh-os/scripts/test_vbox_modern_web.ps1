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
Start-Sleep -Seconds 20

Write-Host "[2] Capturing Modern Aurora Bloom Desktop + DHCP Status..."
& $vbox controlvm $vmName screenshotpng "$brain\vbox_modern_01_desktop.png"
Start-Sleep -Seconds 1

Write-Host "[3] Opening Browser (F3) and fetching LIVE http://example.com over real DNS+TCP+HTTP..."
& $vbox controlvm $vmName keyboardputscancode 3d bd
Start-Sleep -Seconds 1
& $vbox controlvm $vmName keyboardputscancode 1c 9c
Start-Sleep -Seconds 4
& $vbox controlvm $vmName screenshotpng "$brain\vbox_modern_02_live_web.png"

Write-Host "[4] Clicking System Tray with real PS/2 mouse -> Modern Quick Settings Panel..."
$vb = New-Object -ComObject VirtualBox.VirtualBox
$sess = New-Object -ComObject VirtualBox.Session
$m = $vb.FindMachine($vmName)
$m.LockMachine($sess, 1)
$mouse = $sess.Console.Mouse
for ($i = 0; $i -lt 18; $i++) {
    $mouse.PutMouseEvent(20, 20, 0, 0, 0)
    Start-Sleep -Milliseconds 30
}
Start-Sleep -Milliseconds 300
$mouse.PutMouseEvent(0, 0, 0, 0, 1)
Start-Sleep -Milliseconds 120
$mouse.PutMouseEvent(0, 0, 0, 0, 0)
Start-Sleep -Seconds 2
& $vbox controlvm $vmName screenshotpng "$brain\vbox_modern_03_quick_settings.png"

Write-Host "[5] Opening Network Manager (F8)..."
& $vbox controlvm $vmName keyboardputscancode 42 c2
Start-Sleep -Seconds 2
& $vbox controlvm $vmName screenshotpng "$brain\vbox_modern_04_network.png"

$sess.UnlockMachine()
& $vbox controlvm $vmName poweroff
Start-Sleep -Seconds 2

Write-Host "`n===== Network & Browser Serial Log ====="
if (Test-Path $serial) {
    Get-Content $serial | Select-String "DHCP|NETWORK|BROWSER"
}
Write-Host "===== COMPLETE ====="
