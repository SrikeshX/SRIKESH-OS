$ErrorActionPreference = "Continue"
$vbox = "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe"
$vmName = "SRIKESH OS"
$proj = "C:\Users\shres\OneDrive\Desktop\RUST MICRO-OS KERNEL PROJECT\srikesh-os"
$art = "$proj\build_artifacts"
$brain = "C:\Users\shres\.gemini\antigravity\brain\43e9f22a-f7d1-4d29-a088-e60efad3b868"
$vdi = "$art\srikesh-uefi.vdi"
$uefi = "$art\uefi.img"
$uuid = "25b4b139-4058-4f3f-9922-216ac39fe5f6"

& $vbox controlvm $vmName poweroff 2>$null | Out-Null
Start-Sleep -Seconds 2
if (Test-Path $vdi) { Remove-Item $vdi -Force -ErrorAction SilentlyContinue }
& $vbox convertfromraw $uefi $vdi --format VDI
& $vbox internalcommands sethduuid $vdi $uuid

Write-Host "[1] Starting VM headless..."
& $vbox startvm $vmName --type headless
Start-Sleep -Seconds 19

# 1. Open Browser (F3) -> Chrome-inspired New Tab Page
Write-Host "[2] Capturing Chrome-inspired Browser New Tab Page..."
& $vbox controlvm $vmName keyboardputscancode 3d bd
Start-Sleep -Seconds 2
& $vbox controlvm $vmName screenshotpng "$brain\vbox_chrome_01_newtab.png"

# 2. Press Enter on http://example.com -> Live HTTP/1.1 200 OK
Write-Host "[3] Fetching live http://example.com..."
& $vbox controlvm $vmName keyboardputscancode 1c 9c
Start-Sleep -Seconds 4
& $vbox controlvm $vmName screenshotpng "$brain\vbox_chrome_02_live_example.png"

# 3. Ctrl+A, type "chrome://dino", Enter, Space to start & jump!
Write-Host "[4] Launching Playable Offline Dinosaur Runner (chrome://dino)..."
& $vbox controlvm $vmName keyboardputscancode 1d 1e 9e 9d
Start-Sleep -Milliseconds 300
# c=2e h=23 r=13 o=18 m=32 e=12 :=2a+27 / /=35 35 d=20 i=17 n=31 o=18 Enter=1c
& $vbox controlvm $vmName keyboardputscancode 2e ae 23 a3 13 93 18 98 32 b2 12 92 2a 27 a7 aa 35 b5 35 b5 20 a0 17 97 31 b1 18 98 1c 9c
Start-Sleep -Seconds 1
# Press Space to start & jump!
& $vbox controlvm $vmName keyboardputscancode 39 b9
Start-Sleep -Seconds 2
& $vbox controlvm $vmName screenshotpng "$brain\vbox_chrome_03_dino_game.png"

# 4. Move real PS/2 mouse to Wi-Fi tray icon (836, 744) and click -> Windows-11 Wi-Fi Quick Panel!
Write-Host "[5] Clicking Wi-Fi tray icon -> Windows-11 Wi-Fi Quick Panel..."
$vb = New-Object -ComObject VirtualBox.VirtualBox
$sess = New-Object -ComObject VirtualBox.Session
$m = $vb.FindMachine($vmName)
$m.LockMachine($sess, 1)
$mouse = $sess.Console.Mouse
for ($i = 0; $i -lt 16; $i++) {
    $mouse.PutMouseEvent(20, 22, 0, 0, 0)
    Start-Sleep -Milliseconds 30
}
Start-Sleep -Milliseconds 300
$mouse.PutMouseEvent(0, 0, 0, 0, 1)
Start-Sleep -Milliseconds 120
$mouse.PutMouseEvent(0, 0, 0, 0, 0)
Start-Sleep -Seconds 2
& $vbox controlvm $vmName screenshotpng "$brain\vbox_chrome_04_wifi_panel.png"

$sess.UnlockMachine()
& $vbox controlvm $vmName poweroff
Start-Sleep -Seconds 2
Write-Host "===== COMPLETE ====="
