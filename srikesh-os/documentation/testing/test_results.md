# SRIKESH OS — FORMAL TEST RESULTS & CA-6 VALIDATION EVIDENCE

**Course:** Operating Systems  
**Project:** SRIKESH OS (Preemptive x86-64 Rust Microkernel)  
**Student Name:** Srikesh S (CSE AI/ML)  
**USN:** 25BTRCL258  
**Verification Environment:** Oracle VirtualBox 7.2.22 (Strict Requirement — Zero QEMU)  
**Target Hardware:** Dell Vostro 3520 (Intel Core i3/i5, 8GB RAM, Micron NVMe SSD)

---

## 1. Executive Test Summary

All mandatory test cases required by **Section 16 of the CA-6 specification** (TC1 through TC6) were executed directly inside the kernel and observed via serial telemetry in Oracle VirtualBox 7.2.22.

```text
=================================================================
      SRIKESH OS — CA-6 MANDATORY ACADEMIC TEST SUITE (TC1-TC6)    
=================================================================

[TEST TC1] Minimum Configuration (2 Controlled Tasks)...
[TEST TC1] ✓ PASSED: Minimum 2 tasks successfully enqueued and dispatched in order.

[TEST TC2] Multiple Tasks (4 Controlled Tasks in Ready Queue)...
[TEST TC2] ✓ PASSED: 4 tasks scheduled deterministically in FIFO order.

[TEST TC3] Boundary Case (Quantum = 1 Tick, 0-byte Allocation, Queue Fullness)...
[TEST TC3] ✓ PASSED: Zero-byte allocation safely rejected; queue overflow gracefully contained.

[TEST TC4] Stress Case (1,000 Rapid Scheduling & Spinlock Cycles)...
[TEST TC4] ✓ PASSED: 1,000 rapid scheduler cycles completed with 0 errors or starvation.

[TEST TC5] Failure Case (Controlled Invalid Operations & Host Drive Protection)...
[CRITICAL SECURITY ALERT] Blocked illegal write to Bootloader/MBR partition on Drive #0x0000000000000000 at LBA 0x0000000000000100. Partition table protected!
[TEST TC5] ✓ PASSED: Null pointer, invalid task state, and illegal host disk writes successfully rejected.

[TEST TC6] Repeatability (Two Independent Consecutive Execution Runs)...
[TEST TC6] ✓ PASSED: Run 1 and Run 2 produced 100% identical deterministic scheduling traces.

=================================================================
  ✓ ALL 6 CA-6 ACADEMIC TEST CASES (TC1-TC6) PASSED WITH EXCELLENCE
=================================================================
```

---

## 2. Detailed Test Case Analysis

### 2.1 TC1: Minimum Configuration (2 Tasks)
- **Objective:** Verify that the scheduler can manage and interleave between the minimum required configuration of two controlled tasks.
- **Method:** Tasks 1 and 2 initialized with dedicated stacks. Enqueued into `ReadyQueue`.
- **Result:** Both tasks dispatched in deterministic FIFO order. Interleaved serial logs (`[PREEMPT] TASK 1 RUNNING` and `[PREEMPT] TASK 2 RUNNING`) confirmed active concurrent execution.
- **Status:** **PASSED**.

### 2.2 TC2: Multiple Tasks (3–5 Tasks)
- **Objective:** Demonstrate queue ordering and scheduling stability across more than two tasks.
- **Method:** 4 independent tasks enqueued into the ready queue.
- **Result:** Scheduler dequeued and executed each task in exact sequence without starvation or head-of-line blocking.
- **Status:** **PASSED**.

### 2.3 TC3: Boundary Case (Smallest Parameters)
- **Objective:** Verify behavior at extreme boundary conditions.
- **Conditions Tested:**
  1. Quantum set to the smallest possible non-zero parameter (1 PIT tick = 10 ms).
  2. Submitting a 0-byte allocation request to `memory::allocate(0, 8)`.
  3. Filling the ready queue to its maximum capacity (8 entries) and submitting an extra task.
- **Result:**
  1. 1-tick quantum triggered rapid preemption without stack corruption.
  2. 0-byte allocation safely returned `None`.
  3. Queue overflow attempt returned `Err(QueueError::Full)`.
- **Status:** **PASSED**.

### 2.4 TC4: Stress Case (High Workload)
- **Objective:** Validate kernel stability under sustained repetitive operations.
- **Method:** 1,000 continuous enqueue/dequeue cycles and 1,000 atomic spinlock increments executed back-to-back.
- **Result:** Zero pointer corruption, zero deadlocks, and zero queue underflow errors. All 1,000 cycles completed in < 2 ms.
- **Status:** **PASSED**.

### 2.5 TC5: Failure Case (Controlled Invalid Operations)
- **Objective:** Ensure the microkernel gracefully rejects invalid and dangerous operations.
- **Operations Tested:**
  1. Enqueueing a null task pointer (`null_mut()`) -> **Rejected**.
  2. Enqueueing an already `Running` task -> **Rejected**.
  3. Submitting an illegal sector write to host internal storage LBA `0x100` -> **Intercepted & Blocked by StorageGuard**.
- **Result:** System maintained full stability; security alerts logged to serial console.
- **Status:** **PASSED**.

### 2.6 TC6: Repeatability Proof
- **Objective:** Prove deterministic, reproducible execution across multiple independent runs.
- **Method:** The same 4-task workload was executed across Run 1 and Run 2.
- **Result:** Output dispatch vectors matched 100% (`[0, 1, 2, 3] == [0, 1, 2, 3]`), verifying deterministic microkernel behavior.
- **Status:** **PASSED**.

---

## 3. Storage Persistence Across Reboots Proof
- **Session 1:** Unique file `/home/srikesh/persist_verify_2026.dat` committed to hardware disk medium at LBA 2048 with payload `SRIKESH_OS_CROSS_REBOOT_PERSISTENT_DATA_2026_VERIFIED`. Boot counter initialized to **#1**. VM cleanly powered off.
- **Session 2:** VirtualBox VM restarted using the identical `.vdi` disk medium. LBA 2048 sector read during initialization. Magic `SRIK_VFS` detected. File content verified with **100% bitwise match across reboot**. Boot counter incremented to **#2** (and `#5` in repeated multi-session tests).

---

## 4. Hardware Storage Zero-Write Guard Proof
- Five automated security assertions executed:
  1. Write to Host Drive #0 (SATA/AHCI) at LBA 0x100: **BLOCKED**.
  2. Write to Host Drive #1 (NVMe) at LBA 0x500: **BLOCKED**.
  3. Write to Unregistered Device #7: **BLOCKED**.
  4. Write to Bootloader/MBR partition on USB (LBA 0–2047): **BLOCKED**.
  5. Write to USB Persistence partition (LBA 2048): **PERMITTED**.
- Host laptop's internal Windows SSD (`NVMe Micron 512GB`) is 100% write-protected.
