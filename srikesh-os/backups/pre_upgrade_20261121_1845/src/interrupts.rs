use core::arch::global_asm;
use core::sync::atomic::{AtomicU64, Ordering};

use x86_64::structures::idt::InterruptDescriptorTable;
use x86_64::VirtAddr;

pub const TIMER_INTERRUPT_ID: u8 = 32;
pub const KEYBOARD_INTERRUPT_ID: u8 = 33;
pub const MOUSE_INTERRUPT_ID: u8 = 44;

pub static TIMER_TICKS: AtomicU64 = AtomicU64::new(0);

unsafe extern "C" {
    fn srikesh_timer_interrupt_wrapper();
}

extern "C" fn timer_interrupt_rust(frame: usize) -> usize {
    TIMER_TICKS.fetch_add(1, Ordering::Relaxed);
    crate::pic::end_of_interrupt(TIMER_INTERRUPT_ID);
    crate::timer_schedule(frame)
}

extern "x86-interrupt" fn keyboard_interrupt_handler(
    _stack_frame: x86_64::structures::idt::InterruptStackFrame,
) {
    crate::keyboard::handle_keyboard_interrupt();
    crate::pic::end_of_interrupt(KEYBOARD_INTERRUPT_ID);
}

extern "x86-interrupt" fn mouse_interrupt_handler(
    _stack_frame: x86_64::structures::idt::InterruptStackFrame,
) {
    crate::mouse::handle_mouse_interrupt();
    crate::pic::end_of_interrupt(MOUSE_INTERRUPT_ID);
}

global_asm!(
    ".global srikesh_timer_interrupt_wrapper",
    ".type srikesh_timer_interrupt_wrapper,@function",
    "srikesh_timer_interrupt_wrapper:",
    "push rax",
    "push rbx",
    "push rcx",
    "push rdx",
    "push rsi",
    "push rdi",
    "push rbp",
    "push r8",
    "push r9",
    "push r10",
    "push r11",
    "push r12",
    "push r13",
    "push r14",
    "push r15",
    "mov rdi, rsp",
    "and rsp, -16",
    "cld",
    "call {handler}",
    "mov rsp, rax",
    "pop r15",
    "pop r14",
    "pop r13",
    "pop r12",
    "pop r11",
    "pop r10",
    "pop r9",
    "pop r8",
    "pop rbp",
    "pop rdi",
    "pop rsi",
    "pop rdx",
    "pop rcx",
    "pop rbx",
    "pop rax",
    "iretq",
    ".size srikesh_timer_interrupt_wrapper, .-srikesh_timer_interrupt_wrapper",
    handler = sym timer_interrupt_rust,
);

static IDT: spin::Lazy<InterruptDescriptorTable> = spin::Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(crate::gdt::DOUBLE_FAULT_IST_INDEX);
        idt.page_fault
            .set_handler_fn(page_fault_handler)
            .set_stack_index(crate::gdt::PAGE_FAULT_IST_INDEX);
        idt.general_protection_fault
            .set_handler_fn(general_protection_fault_handler)
            .set_stack_index(crate::gdt::PAGE_FAULT_IST_INDEX);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.divide_error.set_handler_fn(divide_error_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
    }

    idt[KEYBOARD_INTERRUPT_ID].set_handler_fn(keyboard_interrupt_handler);
    idt[MOUSE_INTERRUPT_ID].set_handler_fn(mouse_interrupt_handler);

    unsafe {
        idt[TIMER_INTERRUPT_ID]
            .set_handler_addr(VirtAddr::new(
                srikesh_timer_interrupt_wrapper as *const () as u64,
            ));
    }

    idt
});

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: x86_64::structures::idt::InterruptStackFrame,
) {
    crate::serial::write_str("[EXCEPTION] BREAKPOINT\n");
    crate::serial::write_str("  RIP: ");
    crate::serial::write_hex(stack_frame.instruction_pointer.as_u64());
    crate::serial::write_str("\n");
}

extern "x86-interrupt" fn divide_error_handler(
    stack_frame: x86_64::structures::idt::InterruptStackFrame,
) {
    crate::serial::write_str("\n[FATAL EXCEPTION] DIVIDE ERROR (#DE)\n");
    crate::serial::write_str("  RIP: ");
    crate::serial::write_hex(stack_frame.instruction_pointer.as_u64());
    crate::serial::write_str("\n");
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn invalid_opcode_handler(
    stack_frame: x86_64::structures::idt::InterruptStackFrame,
) {
    crate::serial::write_str("\n[FATAL EXCEPTION] INVALID OPCODE (#UD)\n");
    crate::serial::write_str("  RIP: ");
    crate::serial::write_hex(stack_frame.instruction_pointer.as_u64());
    crate::serial::write_str("\n");
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: x86_64::structures::idt::InterruptStackFrame,
    error_code: u64,
) {
    crate::serial::write_str("\n[FATAL EXCEPTION] GENERAL PROTECTION FAULT (#GP)\n");
    crate::serial::write_str("  Error Code: ");
    crate::serial::write_hex(error_code);
    crate::serial::write_str("\n  RIP: ");
    crate::serial::write_hex(stack_frame.instruction_pointer.as_u64());
    crate::serial::write_str("\n  CS: ");
    crate::serial::write_hex(stack_frame.code_segment.0 as u64);
    crate::serial::write_str("\n  RFLAGS: ");
    crate::serial::write_hex(stack_frame.cpu_flags.bits());
    crate::serial::write_str("\n");
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: x86_64::structures::idt::InterruptStackFrame,
    error_code: x86_64::structures::idt::PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;
    crate::serial::write_str("\n[FATAL EXCEPTION] PAGE FAULT (#PF)\n");
    crate::serial::write_str("  Accessed Address (CR2): ");
    if let Ok(addr) = Cr2::read() {
        crate::serial::write_hex(addr.as_u64());
    } else {
        crate::serial::write_str("<invalid>");
    }
    crate::serial::write_str("\n  Error Code: ");
    crate::serial::write_hex(error_code.bits());
    crate::serial::write_str("\n  RIP: ");
    crate::serial::write_hex(stack_frame.instruction_pointer.as_u64());
    crate::serial::write_str("\n");
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: x86_64::structures::idt::InterruptStackFrame,
    error_code: u64,
) -> ! {
    crate::serial::write_str("\n[FATAL EXCEPTION] DOUBLE FAULT (#DF)\n");
    crate::serial::write_str("  Error Code: ");
    crate::serial::write_hex(error_code);
    crate::serial::write_str("\n  RIP: ");
    crate::serial::write_hex(stack_frame.instruction_pointer.as_u64());
    crate::serial::write_str("\n");
    loop {
        x86_64::instructions::hlt();
    }
}
