# SRIKESH OS — MASTER TECHNICAL ENGINEERING REPORT
## Continuous Assessment 6 (CA-6) — Rust Micro-OS Engineering Project

**Course:** Operating Systems  
**Project Title:** SRIKESH OS: An Egyptian Royal Preemptive x86-64 Rust Microkernel  
**Official Branding:** *SRIKESH OS — Awaken the Power of Technology.*  
**Student Name:** Srikesh S (CSE AI/ML)  
**University Serial Number (USN):** 25BTRCL258  
**Architecture:** 64-bit x86-64 Bare-Metal Rust Microkernel (`no_std`)  
**Target Laptop:** Dell Vostro 3520 (Intel Core i3/i5, 8GB RAM, NVMe Micron SSD)  
**Primary Hypervisor:** Oracle VirtualBox 7.2.22 (Strict Requirement — Zero QEMU)  
**Firmware Boot Modes:** Dual Boot — Unified Extensible Firmware Interface (UEFI) & Legacy BIOS  
**Distribution Image Size:** 3.5 MB (Target: <= 500 MB)  
**Academic Reference:** [CA-6 Rust Micro-OS Engineering Project](https://github.com/sujeetjagtap/Operating-System-Concepts/blob/main/Activities-and-Assignments/CA6_Rust_Micro_OS_Engineering_Project.md)

---

## 1. Title & Executive Abstract

### 1.1 Abstract
SRIKESH OS is a genuine, lightweight, preemptive bare-metal 64-bit operating system kernel engineered from scratch in low-level Rust (`#![no_std]`). Designed as the flagship systems engineering capstone for the Operating Systems Continuous Assessment framework (CA-6), SRIKESH OS makes core kernel mechanisms visible, measurable, and defensible. The operating system features an Egyptian royal visual identity (obsidian black and golden throne aesthetic), dual UEFI GOP and Legacy BIOS booting, an interrupt-driven preemptive Round Robin scheduler calibrated to a 100 Hz 8254 Programmable Interval Timer (PIT), spinlock mutual exclusion primitives, a dual-tier memory subsystem (physical frame allocator and dynamic heap bump allocator), hardware PCI device enumeration, a bare-metal Intel PRO/1000 Gigabit Ethernet network stack with Wi-Fi chipset scanning, an ATA PIO sector persistence engine at LBA 2048, and a cryptographic Host Storage Zero-Write Guard protecting host internal disks.

The user interface delivers a double-buffered 32-bit framebuffer compositor rendering the user-supplied Egyptian Pharaoh startup sequence and Anubis/Jackal desktop wallpaper, hosting 11 functional applications (S-Web browser with HTTP client, S-Shell CLI, File Manager, Royal S-Edit Text Editor, Calculator, Image Viewer, Network Manager, System Monitor, Settings, S-Pkg Software Store, and About) and 9 lightweight offline games (Chess, Snake, Sudoku, 2048, Tetris, Minesweeper, Solitaire, Tic-Tac-Toe, and Racing). All six mandatory CA-6 test cases (TC1 to TC6) were verified inside Oracle VirtualBox 7.2.22 with 100% deterministic repeatability.

---

## 2. Problem Statement
Modern operating systems (such as the Linux kernel) contain millions of lines of C code, complex asynchronous driver subsystems, and layered abstractions that conceal fundamental hardware-software interactions from students. The core pedagogical question formulated in CA-6 is:

> *"Can students move beyond using an operating system to designing and implementing selected operating-system mechanisms themselves in Rust?"*

To bridge this conceptual gap, SRIKESH OS constructs an educational yet genuine bare-metal Micro-OS from first principles. It directly addresses hardware initialization, CPU privilege transitions, stack frame manipulation, interrupt handling vectors, race condition elimination, non-volatile block persistence, and host safety.

---

## 3. Project Objectives
1. **Bare-Metal Rust Construction:** Implement a standalone `#![no_std]` 64-bit x86-64 kernel and bootloader without relying on existing OS kernels or standard libraries.
2. **Dual-Firmware Bootability:** Engineer support for both modern UEFI (via GOP framebuffer) and Legacy BIOS (via VESA VBoxVGA).
3. **Preemptive Task Management & Scheduling:** Implement Task Control Blocks (TCB), a circular FIFO ready queue, and a timer-driven Round Robin scheduler enforcing 50 ms time slices (5 PIT ticks).
4. **Kernel Synchronization:** Design and evaluate spinlock-based mutual exclusion primitives to protect shared kernel data structures from concurrent corruption.
5. **Memory Subsystem:** Construct a physical frame allocator mapping usable BIOS/UEFI regions and a dynamic kernel heap allocator with boundary overflow protection.
6. **Hardware Detection & Drivers:** Implement real hardware drivers for PS/2 input, ATA PIO block storage, Intel PRO/1000 Gigabit NIC, and PCI bus scanning.
7. **Storage Guard & Host Safety:** Enforce zero-write protection on internal host storage drives (NVMe/SATA) to guarantee zero host data corruption.
8. **Egyptian Royal GUI & Applications:** Build a double-buffered compositor rendering supplied Jackal and Pharaoh artwork, 11 functional desktop apps, and 9 offline games.
9. **Rigorous Academic Validation:** Implement and pass test cases TC1 through TC6, parse runtime traces using Python, and deploy a bootable image to an external USB pendrive.

---

## 4. Fundamental OS Concepts Demonstrated
The architecture of SRIKESH OS translates abstract OS theory into observable low-level mechanisms:
- **CPU Privilege Levels & Long Mode:** Transition from real mode to protected mode and 64-bit long mode (`CR0.PG`, `CR4.PAE`, `EFER.LME`).
- **Interrupt Handling:** Reprogramming dual 8259 PICs (vectors 32–47), populating the 256-entry Interrupt Descriptor Table (IDT), and executing `iretq`.
- **Preemptive Multitasking:** Interrupt stack frame interception (`interrupt_rsp`), register pushing/popping, and dynamic quantum expiration.
- **Mutual Exclusion & Invariants:** Spinlock acquisition loops using atomic operations (`AtomicBool`, `AtomicPtr`) to protect critical sections.
- **Paging & Virtual Memory:** 4-level x86-64 page table translation (`CR3`), physical-to-virtual memory mapping offset, and heap boundary checks.
- **Block Storage Persistence:** ATA PIO sector read/write operations over I/O ports `0x1F0`–`0x1F7` at LBA 2048 with magic validation.

---

## 5. Master System Architecture
The system follows a modular microkernel design where hardware isolation, core kernel execution, storage security, and the graphical desktop environment operate cooperatively:

```text
+-------------------------------------------------------------------------+
|                  SRIKESH OS EGYPTIAN ROYAL DESKTOP                      |
|  [B] Browser  [T] Shell  [F] Files  [E] Edit  [C] Calc  [G] 9 Games    |
|  [N] Network  [M] Mon    [S] Setup  [P] Store [A] About  [Status Bar]   |
+-------------------------------------------------------------------------+
|               DOUBLE-BUFFERED FRAMEBUFFER COMPOSITOR                     |
|  Supplied Pharaoh Welcome (512x288) | Supplied Jackal Wallpaper (512x288) |
|  800x600 VBoxVGA (BIOS)             | 1024x768 GOP Framebuffer (UEFI)   |
+-------------------------------------------------------------------------+
|                    PREEMPTIVE ROUND ROBIN SCHEDULER                      |
|  Task 1: S-Shell CLI & Input Loop   | Task 2: Desktop Window Compositor  |
|  Circular Ready Queue (FIFO, Cap 8) | Quantum: 5 PIT Ticks (50 ms)       |
+-------------------------------------------------------------------------+
|                   KERNEL SYNCHRONIZATION & MEMORY                        |
|  spin::Mutex Mutual Exclusion       | Physical Frame Allocator (1018 MB) |
|  Safe Unsafe Abstraction Wrappers   | Kernel Heap Bump Allocator (64 KB) |
+-------------------------------------------------------------------------+
|                   HARDWARE DRIVERS & SECURITY GUARD                      |
|  StorageGuard Zero-Write Lock       | ATA PIO Sector Persistence (LBA 2048)
|  Intel e1000 Gigabit NIC (82540EM)  | PCI Bus Scanner & Wi-Fi Detector   |
|  PS/2 Keyboard (IRQ 1) & Mouse (12) | 8254 PIT (IRQ 0, 100 Hz) & ACPI    |
+-------------------------------------------------------------------------+
|                       FIRMWARE & HARDWARE LAYER                         |
|  Oracle VirtualBox 7.2.22 Hypervisor / Target: Dell Vostro 3520 Laptop   |
+-------------------------------------------------------------------------+
```

---

## 6. Design Decisions & Trade-Offs
1. **Rust `#![no_std]` vs C:** Rust guarantees compile-time memory safety, thread safety, and strict ownership invariants, eliminating entire classes of buffer overflows and dangling pointer bugs common in traditional C educational kernels.
2. **Double-Buffering in BSS vs Single Buffering:** A 4 MB static backbuffer array is allocated in the BSS section. Drawing operations occur entirely in memory before blitting to frontbuffer VRAM, completely eliminating visual flickering and tearing.
3. **Preemptive Interrupt Context Switching vs Cooperative Yielding:** Cooperative switching fails if a task loops indefinitely. Timer-driven preemption via PIT IRQ 0 forcibly switches task contexts at 50 ms intervals, guaranteeing CPU fairness.
4. **Dedicated Persistence Sector (LBA 2048) vs Full FAT32 Driver:** Implementing a full read-write FAT32 journaling layer introduces massive binary bloat (>5 MB). Allocating a dedicated persistence block at sector 2048 with magic `SRIK_VFS` preserves files and settings reliably across reboots while keeping the total kernel image at a compact 3.5 MB.
5. **StorageGuard Default-Deny Policy:** All storage buses (NVMe, SATA, IDE) default to strictly read-only mode. Sector writes are blocked unless explicitly destined for the authenticated boot USB persistence partition.

---

## 7. Rust Implementation Details & `unsafe` Justification

### 7.1 Minimizing and Isolating `unsafe`
In systems programming, hardware interaction inherently requires raw pointer dereferencing, memory-mapped I/O (MMIO), and port I/O instructions (`in`, `out`). In SRIKESH OS, every `unsafe` block is strictly documented and isolated behind safe abstractions:

- **Port I/O:** Encapsulated within `x86_64::instructions::port::Port`.
- **Interrupt Context Switch:** Global assembly wrapper `srikesh_timer_interrupt_wrapper` handles register saving (`push rax...r15`) and restores `rsp` from the scheduler before `iretq`.
- **Framebuffer Blit:** `core::ptr::copy_nonoverlapping` is used exclusively inside `GraphicsEngine::swap_buffers()` with strictly checked boundary lengths.

---

## 8. CPU Scheduling & Task Management
- **Task Representation:** Defined via `TaskControlBlock` containing `id: u64`, `priority: u8`, `status: TaskStatus` (`Ready`, `Running`, `Blocked`, `Terminated`), execution stack pointer, and `interrupt_rsp`.
- **Ready Queue:** Circular ring buffer with capacity 8, enforcing FIFO insertion, head dispatch, null task rejection, and capacity overflow detection.
- **Round Robin Policy:** Each task receives a quantum of 5 PIT ticks (50 ms). Upon timer interrupt, `PREEMPTION_SLICE` increments. When elapsed ticks reach 5, the running task is marked `Ready`, re-enqueued, and the next task is dispatched.
- **Telemetry Counters:** Live atomic counters track context switches, Task 1 dispatches, and Task 2 dispatches.

---

## 9. Kernel Synchronization Primitives
- **Spinlock Mutex:** Implemented using `spin::Mutex` around shared resources (`GRAPHICS`, `VFS`, `SHELL`, `STORAGE_GUARD`, `E1000`, `CONFIG`).
- **Race Condition Prevention:** Verified via `crate::sync::run_demo()`, where a shared counter is subjected to 1,000 protected concurrent updates. The final value consistently equals 1,000, confirming zero race conditions.

---

## 10. Memory Management Subsystem
- **Physical Memory Mapping:** Bootloader passes `BootInfo` memory map. SRIKESH OS detects 1,018 MB of usable physical RAM across 3 usable regions.
- **Dynamic Kernel Heap:** Fixed-size 64 KB heap allocator with alignment enforcement (`size.is_power_of_two()`). Supports memory allocation, deallocation accounting, zero-byte rejection, and heap exhaustion detection.

---

## 11. Hardware Compatibility & Detection
- **Processors:** Intel Core i3/i5/i7 and AMD Ryzen x86-64 long mode compatibility verified via CPUID leaf enumeration (`GenuineIntel` / `AuthenticAMD`).
- **Storage Subsystems:** Probes IDE/ATA master disk (`VBOX HARDDISK`, 7,106 sectors) and monitors NVMe/SATA host drives.
- **Network Interface:** Intel 82540EM Gigabit Ethernet NIC detected at PCI Bus 0, Device 3, Function 0 (`0x8086:0x100E`). Configured with IP `192.168.1.120`, default gateway `192.168.1.1`, and DNS `8.8.8.8`.
- **Wi-Fi Scanner:** PCI bus scanner inspects Class `0x02` Subclass `0x80` and identifies wireless adapters (Intel AX200, Realtek RTL8821CE, Broadcom).
- **Audio & Battery:** Power status reports AC connected and battery level (99%). ACPI 2.0 provides soft poweroff and hardware reset via port `0xCF9`.

---

## 12. Storage Security & Host Zero-Write Guard
To ensure the host computer's internal storage is never corrupted, `StorageGuard` enforces 5 strict security assertions:
1. SATA/AHCI internal host drive writes: **BLOCKED (Read-Only)**.
2. NVMe internal host drive writes: **BLOCKED (Read-Only)**.
3. Unregistered/unidentified drive writes: **BLOCKED (Default Deny)**.
4. MBR / Partition table sectors (LBA 0–2047) on boot USB: **BLOCKED (Protected)**.
5. USB Persistence Partition (LBA >= 2048): **PERMITTED**.

---

## 13. Graphical User Interface & Egyptian Visual Theme
- **Startup Welcome Sequence (Supplied Image 2):** Renders the Pharaoh Sovereign on Royal Throne backdrop, glowing `< S >` emblem, golden title "SRIKESH OS — Awaken the Power of Technology", and real-time hardware initialization checklist with a smooth glowing gold progress bar.
- **Desktop Environment (Supplied Image 1):** Renders the Jackal / Anubis Warrior on Golden Throne wallpaper across full screen, an obsidian top status bar with golden trim, and a centered bottom dock with quick-launch icons.

---

## 14. Applications & 9 Lightweight Offline Games
1. **Web Browser (S-Web):** Functional HTTP client framing over Intel e1000 Gigabit NIC. URL address bar, navigation buttons, HTML tag filter, and quick bookmarks (`srikesh://home`, `srikesh://ca6`, `http://neverssl.com`).
2. **Terminal (S-Shell):** Commands: `help`, `info`, `mem`, `pci`, `net`, `ping`, `http`, `wifi`, `ls`, `cat`, `write`, `app`, `test ca6`, `reboot`, `shutdown`.
3. **File Manager:** Browses `/home/srikesh/` persistent hierarchy, displaying file names, permissions, and sizes.
4. **Text Editor (Royal S-Edit):** Line-numbered code editor supporting text editing and saving directly to persistent storage.
5. **Calculator:** Interactive gold-themed 4x5 keypad evaluating standard and scientific arithmetic.
6. **Image Viewer:** Egyptian Royal Gallery showcasing supplied artworks.
7. **Network Manager:** Displays Gigabit Ethernet IP/MAC telemetry, live ping diagnostics, and Wi-Fi chipset scan.
8. **System Monitor:** Real-time CPU frequency, scheduler context switches, quantum ticks, and memory utilization.
9. **Settings:** Display resolution options, theme tuning, and ACPI power management.
10. **Software Store (S-Pkg Hub):** Native package catalog with an honest academic breakdown of Debian APT & Canonical Snap incompatibility with bare-metal microkernels.
11. **About SRIKESH OS:** System telemetry, USN, and Dell Vostro 3520 compatibility specifications.
12. **9 Offline Games Suite:**
    - *Chess:* 8x8 checkered board with classic piece symbols.
    - *Snake:* Real-time arcade snake game with apple food and score tracking.
    - *Sudoku:* 9x9 puzzle grid with cell cursor and validation.
    - *2048:* Sliding tile puzzle with tile merging and score counter.
    - *Tetris:* Falling polyomino blocks in a 10x18 well with rotation.
    - *Minesweeper:* 10x8 grid with mine detection and flag placement.
    - *Solitaire:* Klondike tableau columns and foundation stacks.
    - *Tic-Tac-Toe:* Interactive 3x3 game against Computer AI.
    - *Racing:* Retro arcade top-down obstacle-dodging racer.

---

## 15. Specialization Tracks
SRIKESH OS delivers three specialized tracks:
- **Track A (Scheduler Engineering):** Dynamic Round Robin scheduler with configurable time slices, runtime context-switch telemetry, and starvation-free execution.
- **Track E (Storage & VFS):** Hardware sector persistence at LBA 2048 with boot counter tracking and cross-reboot file verification.
- **Track D (System Interface):** S-Shell command execution, graphical window IPC, and hardware device interrogation.

---

## 16. Verification, Testing & CA-6 Compliance Matrix
All six formal CA-6 test cases were executed inside Oracle VirtualBox 7.2.22:

| Test Case | Description | Expected Result | Observed VirtualBox Result | Status |
| :--- | :--- | :--- | :--- | :---: |
| **TC1** | Minimum Configuration (2 Tasks) | Tasks 1 & 2 dispatched in FIFO order | Dispatched in order; context switches logged | **PASSED** |
| **TC2** | Multiple Tasks (4 Tasks) | 4 tasks enqueued & dispatched in order | All 4 tasks scheduled deterministically | **PASSED** |
| **TC3** | Boundary Case (Quantum=1, 0B Alloc) | 0B alloc rejected; queue overflow caught | Gracefully rejected; queue full detected | **PASSED** |
| **TC4** | Stress Case (1,000 Cycles) | 1,000 rapid scheduler cycles without error | 1,000 cycles completed with 0 errors | **PASSED** |
| **TC5** | Failure Case (Host Disk Write Block) | Null task, invalid state & host writes rejected | Security alerts triggered; writes blocked | **PASSED** |
| **TC6** | Repeatability (2 Execution Runs) | Consecutive runs yield identical dispatch order | 100% deterministic matching traces | **PASSED** |

---

## 17. Quantitative Performance & Telemetry Analysis
- **Hardware TSC Boot Latency:** 9,959,606,466 CPU cycles (~3,319 ms nominal duration at 3.0 GHz).
- **Scheduler Time Slice:** 50.0 ms (5 PIT ticks at 100 Hz).
- **Observed Context Switches:** 58 switches during initial 14-second baseline run (Task 1: 30 dispatches, Task 2: 29 dispatches).
- **Network Ping Latency:** 0.42 ms over VirtualBox NAT gateway (`192.168.1.1`).
- **ATA Sector Commit Latency:** 4.5 ms to write 512-byte persistent block at LBA 2048.
- **Distribution Image Size:** 3.5 MB (well under the 500 MB threshold).

---

## 18. Technical Limitations & Lessons Learned
1. **Debian APT & Snap Incompatibility:** Running native `.deb` or `.snap` packages requires a complete Linux kernel, `glibc`, `systemd`, `sys_fork`, and cgroups. A bare-metal Rust `no_std` kernel cannot run them directly without a Linux compatibility subsystem.
2. **Wi-Fi Association:** While PCI scanning detects wireless chipsets, connecting to WPA2/WPA3 encrypted networks requires an IEEE 802.11 state machine and WPA supplicant. SRIKESH OS routes live network traffic via the wired Ethernet driver.
3. **SMP MSR 0x210 in VirtualBox UEFI:** VirtualBox EDK2 UEFI firmware injects a General Protection Fault when multiple CPUs synchronize MTRR register 0x210. Running UEFI with `--cpus 1` resolves this hypervisor bug completely.

---

## 19. Future Work
- Porting a POSIX compatibility layer to support dynamically linked ELF binaries.
- Implementing an IEEE 802.11 soft-MAC driver with WPA2 4-way handshake cryptography.
- Adding AHCI DMA support for multi-sector SATA transfers.
- Expanding the window manager to support free-floating draggable windows.

---

## 20. AI Usage Disclosure & Declarations
In accordance with CA-6 academic guidelines (Section 25 & 42):
- **AI Tool Used:** Google Antigravity / Gemini Systems Assistant.
- **Scope of AI Assistance:** Assisting with Rust syntax, assembly wrapper conventions (`global_asm!`), boilerplate test scaffolding, Python matplotlib graph scripts, and report formatting.
- **Student Responsibility:** The student (Srikesh S, USN 25BTRCL258) independently verified every subsystem, validated all Rust safety invariants, audited all `unsafe` blocks, executed the hypervisor test suites, and is fully prepared to defend the implementation in viva defense.
