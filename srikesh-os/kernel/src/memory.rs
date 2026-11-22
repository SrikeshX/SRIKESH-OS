use bootloader_api::{
    info::{MemoryRegionKind},
    BootInfo,
};

use crate::serial;

fn write_number(mut value: u64) {
    if value == 0 {
        serial::write_str("0");
        return;
    }

    let mut digits = [0u8; 20];
    let mut index = 0;

    while value > 0 {
        digits[index] = b'0' + (value % 10) as u8;
        value /= 10;
        index += 1;
    }

    while index > 0 {
        index -= 1;
        serial::write_byte(digits[index]);
    }
}

pub static PHYSICAL_MEMORY_OFFSET: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

pub fn phys_to_virt(phys: u64) -> u64 {
    PHYSICAL_MEMORY_OFFSET.load(core::sync::atomic::Ordering::Relaxed) + phys
}

/// Translates ANY x86-64 virtual address (including kernel .bss DMA buffers)
/// to its true physical address by walking the active CR3 4-level page table.
pub fn virt_to_phys(virt: u64) -> u64 {
    let offset = PHYSICAL_MEMORY_OFFSET.load(core::sync::atomic::Ordering::Relaxed);
    if offset == 0 {
        return virt;
    }
    if virt >= offset && virt < offset + 0x1000_0000_0000 {
        return virt - offset;
    }

    let (cr3_frame, _) = x86_64::registers::control::Cr3::read();
    let mut table_phys = cr3_frame.start_address().as_u64();

    let indices = [
        ((virt >> 39) & 0x1FF) as usize, // PML4
        ((virt >> 30) & 0x1FF) as usize, // PDPT
        ((virt >> 21) & 0x1FF) as usize, // PD
        ((virt >> 12) & 0x1FF) as usize, // PT
    ];

    for (level, &idx) in indices.iter().enumerate() {
        let table_virt = (offset + table_phys) as *const u64;
        let entry = unsafe { core::ptr::read_volatile(table_virt.add(idx)) };
        if (entry & 1) == 0 {
            return virt; // Not present
        }
        let next_phys = entry & 0x000F_FFFF_FFFF_F000;
        // Check Huge Page bit (PS = bit 7) on PDPT (1 GB) or PD (2 MB)
        if level == 1 && (entry & (1 << 7)) != 0 {
            return (next_phys & !0x3FFF_FFFF) | (virt & 0x3FFF_FFFF);
        }
        if level == 2 && (entry & (1 << 7)) != 0 {
            return (next_phys & !0x1F_FFFF) | (virt & 0x1F_FFFF);
        }
        table_phys = next_phys;
    }

    table_phys | (virt & 0xFFF)
}

pub fn report(boot_info: &BootInfo) {
    if let Some(offset) = boot_info.physical_memory_offset.into_option() {
        PHYSICAL_MEMORY_OFFSET.store(offset, core::sync::atomic::Ordering::Relaxed);
    }
    let mut usable_bytes = 0u64;
    let mut usable_regions = 0u64;

    for region in boot_info.memory_regions.iter() {
        if region.kind == MemoryRegionKind::Usable {
            usable_bytes += region.end - region.start;
            usable_regions += 1;
        }
    }

    serial::write_str("[MEMORY] Usable RAM: ");
    write_number(usable_bytes / (1024 * 1024));
    serial::write_str(" MB\n");

    serial::write_str("[MEMORY] Usable regions: ");
    write_number(usable_regions);
    serial::write_str("\n");

    serial::write_str("[MEMORY] Kernel size: ");
    write_number(boot_info.kernel_len / 1024);
    serial::write_str(" KB\n");
}


// Simple fixed-size kernel bump allocator.
use core::sync::atomic::{AtomicUsize, Ordering};

const HEAP_SIZE: usize = 64 * 1024;

#[repr(align(4096))]
struct Heap([u8; HEAP_SIZE]);

static mut HEAP: Heap = Heap([0; HEAP_SIZE]);
static HEAP_USED: AtomicUsize = AtomicUsize::new(0);

pub fn allocate(size: usize, align: usize) -> Option<core::ptr::NonNull<u8>> {
    if size == 0 || align == 0 || !align.is_power_of_two() {
        return None;
    }

    let base = core::ptr::addr_of_mut!(HEAP).cast::<u8>() as usize;

    loop {
        let current = HEAP_USED.load(Ordering::Relaxed);
        let address = base.checked_add(current)?;
        let aligned = address.checked_add(align - 1)? & !(align - 1);
        let end = aligned.checked_add(size)?;
        let next = end.checked_sub(base)?;

        if next > HEAP_SIZE {
            return None;
        }

        if HEAP_USED.compare_exchange_weak(
            current, next, Ordering::SeqCst, Ordering::Relaxed
        ).is_ok() {
            return core::ptr::NonNull::new(aligned as *mut u8);
        }
    }
}

pub fn heap_used() -> usize {
    HEAP_USED.load(Ordering::SeqCst)
}

pub fn heap_capacity() -> usize {
    HEAP_SIZE
}

pub fn heap_remaining() -> usize {
    HEAP_SIZE.saturating_sub(heap_used())
}

pub fn allocator_demo() {
    crate::logger::info("Starting memory allocator tests");

    // Test 1: Reject invalid allocation requests.
    if allocate(0, 16).is_some()
        || allocate(32, 0).is_some()
        || allocate(32, 3).is_some()
    {
        crate::logger::info("Allocator test failed: invalid request accepted");
        return;
    }
    crate::logger::info("Allocator test passed: invalid requests rejected");

    // Test 2: Allocate memory with different alignments.
    let first = match allocate(128, 16) {
        Some(ptr) => ptr,
        None => {
            crate::logger::info("Allocator test failed: first allocation");
            return;
        }
    };

    let second = match allocate(256, 64) {
        Some(ptr) => ptr,
        None => {
            crate::logger::info("Allocator test failed: second allocation");
            return;
        }
    };

    if first.as_ptr() as usize % 16 != 0
        || second.as_ptr() as usize % 64 != 0
    {
        crate::logger::info("Allocator test failed: alignment");
        return;
    }
    crate::logger::info("Allocator test passed: memory alignment");

    // Test 3: Ensure the allocations do not overlap.
    let first_start = first.as_ptr() as usize;
    let second_start = second.as_ptr() as usize;

    let first_end = first_start + 128;
    let second_end = second_start + 256;

    if first_start < second_end && second_start < first_end {
        crate::logger::info("Allocator test failed: overlapping allocations");
        return;
    }
    crate::logger::info("Allocator test passed: separate allocations");

    // Test 4: Check capacity and remaining heap space.
    if heap_used() > heap_capacity()
        || heap_remaining() != heap_capacity() - heap_used()
    {
        crate::logger::info("Allocator test failed: heap accounting");
        return;
    }
    crate::logger::info("Allocator test passed: heap accounting");

    // Test 5: Consume the remaining heap and verify exhaustion.
    let remaining = heap_remaining();

    if remaining == 0 || allocate(remaining, 1).is_none() {
        crate::logger::info("Allocator test failed: heap exhaustion");
        return;
    }

    if heap_remaining() != 0 || allocate(1, 1).is_some() {
        crate::logger::info("Allocator test failed: exhaustion boundary");
        return;
    }

    crate::logger::info("Allocator test passed: heap exhaustion");
    crate::logger::info("All memory allocator tests passed");
}
