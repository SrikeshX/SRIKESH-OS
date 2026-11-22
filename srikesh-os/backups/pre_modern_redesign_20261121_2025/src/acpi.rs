//! SRIKESH OS - ACPI & System Power Management
//!
//! Features:
//! - Standard QEMU/Bochs ACPI Power-off (Port 0x604 / 0xB004)
//! - System Reset via Keyboard Controller (Port 0x64 pulse 0xFE)
//! - PCI Fast Reset via Port 0xCF9 (value 0x06)

use x86_64::instructions::port::Port;
use crate::serial;

/// Performs a clean system reboot
pub fn reboot() -> ! {
    serial::write_str("\n[SYSTEM] Initiating Hardware Reboot Sequence...\n");

    unsafe {
        // Method 1: Fast PCI reset via 0xCF9
        let mut pci_reset = Port::<u8>::new(0xCF9);
        pci_reset.write(0x02);
        for _ in 0..10_000 { core::hint::spin_loop(); }
        pci_reset.write(0x06);

        // Method 2: 8042 Keyboard Controller Pulse Reset
        let mut kbd = Port::<u8>::new(0x64);
        for _ in 0..10_000 { core::hint::spin_loop(); }
        kbd.write(0xFE);

        // Fallback: Triple fault by loading empty IDT
        let idt_ptr = x86_64::structures::DescriptorTablePointer {
            limit: 0,
            base: x86_64::VirtAddr::new(0),
        };
        x86_64::instructions::tables::lidt(&idt_ptr);
        x86_64::instructions::interrupts::int3();
    }

    loop {
        core::hint::spin_loop();
    }
}

/// Performs a clean system shutdown
pub fn shutdown() -> ! {
    serial::write_str("\n[SYSTEM] SRIKESH OS Shutting Down Cleanly...\n");

    unsafe {
        // QEMU ACPI poweroff
        let mut qemu_acpi = Port::<u16>::new(0x604);
        qemu_acpi.write(0x2000);

        // VirtualBox poweroff
        let mut vbox_acpi = Port::<u16>::new(0x4004);
        vbox_acpi.write(0x3400);

        // Bochs / older QEMU poweroff
        let mut bochs_acpi = Port::<u16>::new(0xB004);
        bochs_acpi.write(0x2000);
    }

    // If emulator didn't exit, halt CPU safely
    loop {
        x86_64::instructions::hlt();
    }
}
