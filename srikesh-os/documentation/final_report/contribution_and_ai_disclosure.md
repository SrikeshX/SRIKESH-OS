# SRIKESH OS — INDIVIDUAL CONTRIBUTION & AI USAGE DISCLOSURE

**Candidate Name:** Srikesh S (CSE AI/ML)  
**University Serial Number (USN):** 25BTRCL258  
**Course:** Operating Systems (Continuous Assessment 6)  
**Project:** SRIKESH OS (Preemptive x86-64 Rust Microkernel)  
**Academic Reference:** CA-6 Rust Micro-OS Engineering Project

---

## 1. Subsystem Ownership & Individual Contribution

As the sole developer of this capstone engineering project, I designed, developed, debugged, and validated all components of SRIKESH OS:

1. **Kernel Architecture & Entry:** Built the `x86_64-unknown-none` freestanding Rust microkernel, configured dual-firmware (UEFI & BIOS) boot targets, and established the serial diagnostic logger.
2. **Task Representation & Preemptive Scheduler:** Designed the `TaskControlBlock`, circular `ReadyQueue`, 5-tick quantum Round Robin scheduling algorithm, and interrupt stack context switching logic.
3. **Interrupts & Synchronization:** Configured the 256-entry IDT, reprogrammed dual 8259 PICs, calibrated the 8254 PIT to 100 Hz, and implemented `spin::Mutex` synchronization.
4. **Memory Management:** Implemented the physical memory frame scanner and dynamic 64 KB kernel heap bump allocator with boundary checks.
5. **Hardware Drivers & Networking:** Implemented PS/2 keyboard/mouse handlers, ATA PIO block persistence (LBA 2048), Intel PRO/1000 Gigabit NIC driver (ARP, IPv4, ICMP Echo ping), and PCI bus scanner.
6. **Storage Security Guard:** Implemented and audited the `StorageGuard` engine with 5 automated security tests to guarantee zero writes to host internal SSDs.
7. **Egyptian Royal Desktop & Applications:** Engineered the double-buffered 32-bit framebuffer compositor using supplied Pharaoh and Jackal images, built 11 functional applications, and created the 9 offline games suite.
8. **Testing & Telemetry:** Built and executed test cases TC1 through TC6, collected serial telemetry, wrote Python parsing scripts, generated performance plots, and authored this documentation.

---

## 2. Generative AI Usage Disclosure

In compliance with Sections 25 and 42 of the CA-6 Course Specification:

- **AI Tools Consulted:** Google Antigravity / Gemini Systems Assistant.
- **Permissible Scope of Use:**
  1. Brainstorming syntax for low-level Rust assembly macros (`core::arch::global_asm!`).
  2. Synthesizing boilerplate test scaffolding for boundary and stress cases.
  3. Formatting Python matplotlib scripts for scheduler trace plotting.
  4. Reviewing compiler warnings and error diagnostics.
- **Academic Integrity Statement:** All kernel mechanisms, hardware interactions, architectural trade-offs, and `unsafe` blocks were independently verified, tested inside Oracle VirtualBox 7.2.22, and understood by the candidate. No AI-generated code was adopted blindly. The candidate is fully prepared to defend every line of kernel code during the viva defense.

Signed,  
**Srikesh S**  
USN: 25BTRCL258  
Department of Computer Science & Engineering (AI & ML)
