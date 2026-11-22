//! SRIKESH OS - Global Descriptor Table (GDT) & Task State Segment (TSS)
//!
//! Provides:
//! - Kernel 64-bit Code Segment (0x08)
//! - Kernel 64-bit Data Segment (0x10)
//! - Dedicated IST1 stack for Double Fault exception handling (prevents Triple Faults)
//! - Dedicated IST2 stack for Page Fault / General Protection Fault handling

use spin::Lazy;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const PAGE_FAULT_IST_INDEX: u16 = 1;

const IST_STACK_SIZE: usize = 4096 * 5; // 20 KB dedicated exception stacks

#[repr(align(16))]
struct ExceptionStack([u8; IST_STACK_SIZE]);

static mut DOUBLE_FAULT_STACK: ExceptionStack = ExceptionStack([0; IST_STACK_SIZE]);
static mut PAGE_FAULT_STACK: ExceptionStack = ExceptionStack([0; IST_STACK_SIZE]);

pub struct Selectors {
    pub code_selector: SegmentSelector,
    pub data_selector: SegmentSelector,
    pub tss_selector: SegmentSelector,
}

static TSS: Lazy<TaskStateSegment> = Lazy::new(|| {
    let mut tss = TaskStateSegment::new();

    let df_top = unsafe {
        let ptr = DOUBLE_FAULT_STACK.0.as_mut_ptr() as u64;
        VirtAddr::new(ptr + IST_STACK_SIZE as u64)
    };
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = df_top;

    let pf_top = unsafe {
        let ptr = PAGE_FAULT_STACK.0.as_mut_ptr() as u64;
        VirtAddr::new(ptr + IST_STACK_SIZE as u64)
    };
    tss.interrupt_stack_table[PAGE_FAULT_IST_INDEX as usize] = pf_top;

    tss
});

static GDT: Lazy<(GlobalDescriptorTable, Selectors)> = Lazy::new(|| {
    let mut gdt = GlobalDescriptorTable::new();
    let code_selector = gdt.append(Descriptor::kernel_code_segment());
    let data_selector = gdt.append(Descriptor::kernel_data_segment());
    let tss_selector = gdt.append(Descriptor::tss_segment(&TSS));

    (
        gdt,
        Selectors {
            code_selector,
            data_selector,
            tss_selector,
        },
    )
});

pub fn init() {
    use x86_64::instructions::segmentation::{Segment, CS, DS, ES, SS};
    use x86_64::instructions::tables::load_tss;

    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        SS::set_reg(GDT.1.data_selector);
        DS::set_reg(GDT.1.data_selector);
        ES::set_reg(GDT.1.data_selector);
        load_tss(GDT.1.tss_selector);
    }
}

pub fn get_selectors() -> &'static Selectors {
    &GDT.1
}
