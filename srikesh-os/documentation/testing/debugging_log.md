# SRIKESH OS — CHRONOLOGICAL ENGINEERING & DEBUGGING LOG

**Project:** SRIKESH OS (Preemptive x86-64 Rust Microkernel)  
**Student:** Srikesh S (25BTRCL258)  
**Hypervisor:** Oracle VirtualBox 7.2.22  
**Target:** Dell Vostro 3520 Laptop (Intel Core i3/i5, 8GB RAM, NVMe SSD)

---

### Incident 1: UEFI Multi-Core Crash (MSR 0x210 General Protection Fault)
- **Observation:** When booting the UEFI image in VirtualBox with `--firmware efi` and `--cpus 2`, the VM immediately aborted with `X64 Exception Type - 0D (#GP) in CpuDxe.pdb`.
- **Localization:** Crash occurred in VirtualBox's internal EDK2 firmware before kernel entry. Registers showed `RCX = 0x210` (`IA32_MTRR_PHYSBASE8`).
- **Hypothesis:** VirtualBox's MTRR emulation table only emulates variable MTRRs 0–7. Attempting to broadcast MTRR #8 across multiple guest APs triggers a `#GP(0)` fault inside EDK2 firmware.
- **Modification & Test:** Reconfigured the UEFI VM to `--cpus 1`.
- **Resolution:** UEFI boot succeeded instantly with 1024x768 GOP 32-bit framebuffer. Kernel entry reached cleanly. Legacy BIOS boot continues to support full multi-core (`--cpus 2`).

---

### Incident 2: Host Internal Drive Protection (Zero-Write Enforcement)
- **Observation:** Need absolute mathematical guarantee that the host computer's internal NVMe/SATA drives cannot be formatted or corrupted during live USB booting.
- **Hypothesis:** Implementing a kernel-level `StorageGuard` security abstraction with a default-deny policy will intercept all ATA/NVMe write operations before they reach hardware I/O ports.
- **Implementation:** Added `StorageGuard` checking bus type and device IDs. Sectors 0–2047 (bootloader/MBR) on all drives and all sectors on host internal drives are locked to `ReadOnlyProtected`.
- **Validation:** 5 automated security tests executed during boot; all 5 illegal write attempts were trapped and blocked with critical security alerts.

---

### Incident 3: Framebuffer Tearing and Screen Flicker
- **Observation:** Direct rendering of visual assets and text to physical video memory caused severe horizontal tearing and flickering during rapid redraws.
- **Hypothesis:** Direct video RAM writes over the PCI bus incur substantial bus latency and miss vertical blanking intervals.
- **Implementation:** Allocated a 4 MB double-buffer in the BSS section (`BACKBUFFER`). All geometry, scaling, and font rendering write to `BACKBUFFER`. A fast `swap_buffers()` blit transfers the complete frame in a single contiguous memory copy.
- **Validation:** Zero tearing, 60 FPS compositor fluidity achieved across all 11 applications and 9 offline games.

---

### Incident 4: Cross-Reboot Sector Persistence Verification
- **Observation:** Live OS USB environments often lose all user state upon poweroff.
- **Hypothesis:** Allocating a dedicated persistence block at sector LBA 2048 with magic `SRIK_VFS` allows persistent configuration files and boot counters to survive full hypervisor shutdowns.
- **Implementation:** Integrated ATA PIO sector read/write driver with checksum validation.
- **Validation:** Tested across two consecutive VirtualBox cold-restart sessions (`srikesh-bios.vdi`). Sector LBA 2048 was detected, file `/home/srikesh/persist_verify_2026.dat` matched 100%, and boot counter incremented to `#5`.

---

### Incident 5: CA-6 Academic Test Harness Verification
- **Observation:** Must fulfill all six mandatory CA-6 test cases (TC1 to TC6) with reproducible serial evidence.
- **Implementation:** Created `ca6_tests.rs` executing during bootloader initialization.
- **Validation:** Serial logs confirmed 6/6 tests passing:
  - TC1: Minimum 2 tasks dispatched in order.
  - TC2: 4 tasks scheduled deterministically in FIFO order.
  - TC3: 0-byte allocation and queue capacity overflow safely contained.
  - TC4: 1,000 rapid cycles completed without deadlocks.
  - TC5: Null pointers, invalid task states, and host drive writes rejected.
  - TC6: Two independent runs produced 100% identical matching dispatch vectors.
