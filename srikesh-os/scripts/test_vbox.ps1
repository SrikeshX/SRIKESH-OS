$vbox = "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe"
$artDir = "C:\Users\shres\OneDrive\Desktop\RUST MICRO-OS KERNEL PROJECT\srikesh-os\build_artifacts"
$vmName = "SRIKESH OS"
$vdiPath = "$artDir\srikesh-uefi.vdi"
$REGISTERED_UUID = "25b4b139-4058-4f3f-9922-216ac39fe5f6"

Write-Host "=== SRIKESH OS — VirtualBox Verification v4 ===" -ForegroundColor Cyan

# Step 1: Kill any running VM
Write-Host "[1] Stopping VM..."
& $vbox controlvm $vmName poweroff 2>$null; Start-Sleep 2

# Step 2: Detach medium
Write-Host "[2] Detaching old medium..."
& $vbox storageattach $vmName --storagectl "SATA" --port 0 --device 0 --medium none 2>$null; Start-Sleep 1

# Step 3: Close old registration
Write-Host "[3] Closing old UUID from registry..."
& $vbox closemedium disk "{$REGISTERED_UUID}" 2>$null; Start-Sleep 1

# Step 4: Delete and recreate VDI
Write-Host "[4] Recreating VDI from latest uefi.img..."
if (Test-Path $vdiPath) { Remove-Item $vdiPath -Force }
& $vbox convertfromraw "$artDir\uefi.img" $vdiPath --format VDI
Write-Host "  VDI created." -ForegroundColor Green

# Step 5: Stamp the registered UUID using sethduuid
Write-Host "[5] Stamping registered UUID ($REGISTERED_UUID) using sethduuid..."
& $vbox internalcommands sethduuid $vdiPath $REGISTERED_UUID
Write-Host "  UUID stamped." -ForegroundColor Green

# Step 6: Register (open) the medium explicitly
Write-Host "[6] Opening/registering medium..."
& $vbox openmedium disk $vdiPath 2>$null; Start-Sleep 1

# Step 7: Re-attach
Write-Host "[7] Attaching VDI to VM..."
& $vbox storageattach $vmName --storagectl "SATA" --port 0 --device 0 --type hdd --medium $vdiPath

# Step 8: Clear serial log
Write-Host "[8] Clearing serial log..."
"[SRIKESH OS Serial Log]" | Set-Content "$artDir\vbox_serial_uefi.log" -Encoding UTF8

# Step 9: Boot
Write-Host "[9] Starting VM headless..." -ForegroundColor Green
& $vbox startvm $vmName --type headless
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: VM failed to start. Full VM config below:" -ForegroundColor Red
    & $vbox showvminfo $vmName
    exit 1
}

# Step 10: Wait
Write-Host "[10] Waiting 22 seconds for boot sequence and desktop..."
Start-Sleep -Seconds 22

# Step 11: Screenshot
Write-Host "[11] Capturing screenshots..."
& $vbox controlvm $vmName screenshotpng "$artDir\vbox_desktop_verified.png"
Start-Sleep -Seconds 2
& $vbox controlvm $vmName screenshotpng "$artDir\vbox_desktop_verified_2.png"

# Step 12: Power off
Write-Host "[12] Powering off..."
& $vbox controlvm $vmName poweroff; Start-Sleep 2

# Step 13: Show log
Write-Host ""
Write-Host "===== Serial Boot Log (last 40 lines) =====" -ForegroundColor Cyan
if (Test-Path "$artDir\vbox_serial_uefi.log") {
    Get-Content "$artDir\vbox_serial_uefi.log" -Tail 40
} else {
    Write-Host "(no serial log found)"
}

Write-Host ""
Write-Host "===== Done! Desktop screenshots saved to build_artifacts/ =====" -ForegroundColor Green
