$ErrorActionPreference = "Continue"
$vbox = "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe"
$vmName = "SRIKESH OS"
$art = "C:\Users\shres\OneDrive\Desktop\RUST MICRO-OS KERNEL PROJECT\srikesh-os\build_artifacts"
$brain = "C:\Users\shres\.gemini\antigravity\brain\43e9f22a-f7d1-4d29-a088-e60efad3b868"
$serial = "$art\vbox_serial_uefi.log"

& $vbox controlvm $vmName poweroff 2>$null | Out-Null
Start-Sleep -Seconds 2
if (Test-Path $serial) { Clear-Content $serial -ErrorAction SilentlyContinue }

Write-Host "[1] Starting VM headless for REAL PS/2 Mouse Verification..."
& $vbox startvm $vmName --type headless
Start-Sleep -Seconds 18

$vb = New-Object -ComObject VirtualBox.VirtualBox
$sess = New-Object -ComObject VirtualBox.Session
$m = $vb.FindMachine($vmName)
$m.LockMachine($sess, 1) # LockType_Shared = 1
$mouse = $sess.Console.Mouse

# Screen is 1024x768, initial cursor is at (512, 384)
# 1. Move mouse smoothly down-left toward Start button at (180, 744)
Write-Host "[2] Moving mouse cursor toward Start/Launcher button..."
for ($i = 0; $i -lt 16; $i++) {
    $mouse.PutMouseEvent(-21, 22, 0, 0, 0)
    Start-Sleep -Milliseconds 40
}
Start-Sleep -Milliseconds 500

# 2. Left click Start button on Taskbar!
Write-Host "[3] Left-clicking Start button..."
$mouse.PutMouseEvent(0, 0, 0, 0, 1)
Start-Sleep -Milliseconds 120
$mouse.PutMouseEvent(0, 0, 0, 0, 0)
Start-Sleep -Seconds 2
& $vbox controlvm $vmName screenshotpng "$brain\vbox_mouse_01_launcher_click.png"

# 3. Move up-right onto Files icon inside Launcher (~400, 440) and Left-click!
Write-Host "[4] Moving cursor to Files icon in Launcher and clicking..."
for ($i = 0; $i -lt 12; $i++) {
    $mouse.PutMouseEvent(18, -25, 0, 0, 0)
    Start-Sleep -Milliseconds 40
}
Start-Sleep -Milliseconds 400
$mouse.PutMouseEvent(0, 0, 0, 0, 1)
Start-Sleep -Milliseconds 120
$mouse.PutMouseEvent(0, 0, 0, 0, 0)
Start-Sleep -Seconds 2
& $vbox controlvm $vmName screenshotpng "$brain\vbox_mouse_02_app_opened.png"

# 4. Move to empty right desktop (~900, 300) and Right-click for Context Menu!
Write-Host "[5] Moving cursor to empty desktop and Right-clicking..."
for ($i = 0; $i -lt 15; $i++) {
    $mouse.PutMouseEvent(33, -10, 0, 0, 0)
    Start-Sleep -Milliseconds 40
}
Start-Sleep -Milliseconds 400
$mouse.PutMouseEvent(0, 0, 0, 0, 2) # Right button = 2
Start-Sleep -Milliseconds 120
$mouse.PutMouseEvent(0, 0, 0, 0, 0)
Start-Sleep -Seconds 2
& $vbox controlvm $vmName screenshotpng "$brain\vbox_mouse_03_context_menu.png"

$sess.UnlockMachine()
& $vbox controlvm $vmName poweroff
Start-Sleep -Seconds 2

Write-Host "`n===== Mouse Events Logged by Kernel Serial Console ====="
if (Test-Path $serial) {
    Get-Content $serial | Select-String "\[MOUSE\]"
}
Write-Host "===== DONE ====="
