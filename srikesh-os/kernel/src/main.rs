#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(unused_unsafe)]
#![allow(static_mut_refs)]
mod cpu;
mod gdt;
mod task;
mod ready_queue;
mod scheduler;
mod sync;
mod memory;
mod logger;
mod serial;
mod interrupts;
mod pic;
mod timer;
mod context;
mod interrupt_context;
mod task_control;
mod scheduler_tests;
mod graphics;
mod icons;
mod startup;
mod keyboard;
mod text_input;
mod mouse;
mod storage_guard;
mod ata;
mod vfs;
mod persistence;
mod pci;
mod e1000;
mod net;
mod acpi;
mod spkg;
mod shell;
mod desktop;
mod games;
mod browser;
mod ca6_tests;

// Permanently allocated task control blocks.
// Their addresses remain stable for the lifetime of the kernel.
static mut TASK1_TCB: task_control::TaskControlBlock =
    task_control::TaskControlBlock::uninitialized(
        1, 5, preempt_task1_entry
    );

static mut TASK2_TCB: task_control::TaskControlBlock =
    task_control::TaskControlBlock::uninitialized(
        2, 5, preempt_task2_entry
    );


use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use bootloader_api::config::Mapping;
use bootloader_api::info::PixelFormat;
use font8x8::{UnicodeFonts, BASIC_FONTS};

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

#[allow(dead_code)]
fn draw_text(
    buffer: &mut [u8],
    width: usize,
    height: usize,
    stride: usize,
    bpp: usize,
    format: PixelFormat,
    text: &str,
    y: usize,
    scale: usize,
) {
    let text_width = text.chars().count() * 8 * scale;
    let start_x = width.saturating_sub(text_width) / 2;

    for (i, ch) in text.chars().enumerate() {
        if let Some(glyph) = BASIC_FONTS.get(ch) {
            for (row, bits) in glyph.iter().enumerate() {
                for col in 0..8 {
                    if (*bits & (1 << col)) != 0 {
                        for sy in 0..scale {
                            for sx in 0..scale {
                                let px = start_x + i * 8 * scale + col * scale + sx;
                                let py = y + row * scale + sy;

                                if px < width && py < height {
                                    let offset =
                                        (py * stride + px) * bpp;

                                    if offset + bpp <= buffer.len() {
                                        match format {
                                            PixelFormat::Bgr | PixelFormat::Rgb => {
                                                if bpp >= 3 {
                                                    buffer[offset] = 255;
                                                    buffer[offset + 1] = 255;
                                                    buffer[offset + 2] = 255;
                                                }
                                            }
                                            PixelFormat::U8 => {
                                                buffer[offset] = 255;
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}




// The boot context stores the kernel's execution state.
static mut BOOT_CONTEXT: context::CpuContext =
    context::CpuContext { rsp: 0 };

// Shared ready queue containing pointers to permanently allocated TCBs.
static mut READY_QUEUE: ready_queue::ReadyQueue =
    ready_queue::ReadyQueue::new();

static CURRENT_TASK: core::sync::atomic::AtomicPtr<task_control::TaskControlBlock> =
    core::sync::atomic::AtomicPtr::new(core::ptr::null_mut());

static PREEMPTION_SLICE: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);

// Scheduler specialization: configurable quantum and runtime statistics.
const SCHEDULER_QUANTUM_TICKS: u64 = 5;

pub(crate) static CONTEXT_SWITCH_COUNT: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);

pub(crate) static TASK1_DISPATCH_COUNT: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);

pub(crate) static TASK2_DISPATCH_COUNT: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);


pub(crate) fn timer_schedule(frame: usize) -> usize {
    use core::sync::atomic::Ordering;

    let current = CURRENT_TASK.load(Ordering::Relaxed);

    if current.is_null() {
        return frame;
    }

    unsafe {
        (*current).interrupt_rsp = frame;
    }

    let elapsed = PREEMPTION_SLICE.fetch_add(1, Ordering::Relaxed) + 1;

    if elapsed < SCHEDULER_QUANTUM_TICKS {
        return frame;
    }

    PREEMPTION_SLICE.store(0, Ordering::Relaxed);

    unsafe {
        let queue = &mut *core::ptr::addr_of_mut!(READY_QUEUE);

        if (*current).status != task_control::TaskStatus::Running {
            return frame;
        }

        (*current).set_status(task_control::TaskStatus::Ready);

        if queue.enqueue(current).is_err() {
            (*current).set_status(task_control::TaskStatus::Running);
            return frame;
        }

        if let Some(next) = scheduler::dispatch_next(queue) {
            if next != current {
                CONTEXT_SWITCH_COUNT.fetch_add(1, Ordering::Relaxed);

                match (*next).id {
                    1 => { TASK1_DISPATCH_COUNT.fetch_add(1, Ordering::Relaxed); }
                    2 => { TASK2_DISPATCH_COUNT.fetch_add(1, Ordering::Relaxed); }
                    _ => {}
                }
            }

            CURRENT_TASK.store(next, Ordering::Relaxed);
            return (*next).interrupt_rsp;
        }

        (*current).set_status(task_control::TaskStatus::Running);
        CURRENT_TASK.store(current, Ordering::Relaxed);
        frame
    }
}

fn halt_forever(message: &str) -> ! {
    serial::write_str(message);

    loop {
        x86_64::instructions::hlt();
    }
}


extern "C" fn preempt_task1_entry() -> ! {
    use core::sync::atomic::Ordering;

    serial::write_str("\n[PREEMPT] TASK 1 STARTED\n");

    let mut progress: u64 = 0;
    let mut performance_reported = false;

    loop {
        progress = progress.wrapping_add(1);

        if progress >= 10_000_000 && !performance_reported {
            performance_reported = true;

            serial::write_str("\n[PERFORMANCE] Scheduler runtime snapshot\n");

            serial::write_str("[PERFORMANCE] Context switches: ");
            serial::write_u64(CONTEXT_SWITCH_COUNT.load(Ordering::Relaxed));
            serial::write_str("\n");

            serial::write_str("[PERFORMANCE] Task 1 dispatches: ");
            serial::write_u64(TASK1_DISPATCH_COUNT.load(Ordering::Relaxed));
            serial::write_str("\n");

            serial::write_str("[PERFORMANCE] Task 2 dispatches: ");
            serial::write_u64(TASK2_DISPATCH_COUNT.load(Ordering::Relaxed));
            serial::write_str("\n");

            serial::write_str("[PERFORMANCE] Scheduler quantum: ");
            serial::write_u64(SCHEDULER_QUANTUM_TICKS);
            serial::write_str(" ticks\n");
        }

        if let Some(ev) = crate::keyboard::pop_key_event() {
            use crate::keyboard::Key;
            serial::write_str("[KEYBOARD EVENT] KeyEvent dispatched\n");

            // Alt+Tab -> switch focused window
            if ev.alt && ev.key == Key::Tab {
                crate::desktop::DESKTOP.lock().cycle_window_focus();
                crate::desktop::render_desktop_frame();
                continue;
            }
            // Alt+F4 -> close focused window
            if ev.alt && ev.key == Key::F4 {
                crate::desktop::DESKTOP.lock().close_focused_or_overlay();
                crate::desktop::render_desktop_frame();
                continue;
            }
            // Windows / Super key -> toggle App Launcher
            if ev.key == Key::Super {
                crate::desktop::launch_app(crate::desktop::ActiveApp::AppLauncher);
                crate::desktop::render_desktop_frame();
                continue;
            }

            // Optional Function Key Shortcuts
            let f_app = match ev.key {
                Key::F1 => Some(crate::desktop::ActiveApp::FileManager),
                Key::F2 => Some(crate::desktop::ActiveApp::Terminal),
                Key::F3 => Some(crate::desktop::ActiveApp::Browser),
                Key::F4 => Some(crate::desktop::ActiveApp::Settings),
                Key::F5 => Some(crate::desktop::ActiveApp::SystemMonitor),
                Key::F6 => Some(crate::desktop::ActiveApp::Games),
                Key::F7 => Some(crate::desktop::ActiveApp::Calculator),
                Key::F8 => Some(crate::desktop::ActiveApp::NetworkManager),
                Key::F9 => Some(crate::desktop::ActiveApp::SoftwareStore),
                Key::F10 => Some(crate::desktop::ActiveApp::About),
                Key::F11 => Some(crate::desktop::ActiveApp::AppLauncher),
                Key::F12 => Some(crate::desktop::ActiveApp::DesktopOnly),
                _ => None,
            };
            if let Some(target) = f_app {
                crate::desktop::launch_app(target);
                crate::desktop::render_desktop_frame();
                continue;
            }

            // ESC closes focused window, launcher, quick settings, or context menu
            if ev.key == Key::Escape {
                crate::desktop::DESKTOP.lock().close_focused_or_overlay();
                crate::desktop::render_desktop_frame();
                continue;
            }

            let (active_app, qs_wifi) = {
                let d = crate::desktop::DESKTOP.lock();
                (d.active_app, d.quick_settings_open && d.qs_wifi_view)
            };

            // Quick Settings Wi-Fi passphrase input
            if qs_wifi {
                crate::desktop::DESKTOP.lock().handle_wifi_pass_key_event(ev);
                crate::desktop::render_desktop_frame();
                continue;
            }

            // Live Search & Enter-to-launch when App Launcher is open
            if active_app == crate::desktop::ActiveApp::AppLauncher {
                let (sw, sh) = {
                    let g = crate::graphics::GRAPHICS.lock();
                    (g.width, g.height)
                };
                crate::desktop::DESKTOP.lock().handle_launcher_key_event(ev, sw, sh);
                crate::desktop::render_desktop_frame();
                continue;
            }

            // Browser address bar input (full typing, Ctrl+C/V/A, arrows, Enter)
            if active_app == crate::desktop::ActiveApp::Browser {
                crate::browser::BROWSER.lock().handle_key(ev);
                crate::desktop::render_desktop_frame();
                continue;
            }

            // Text Editor input (full typing + Ctrl+S save to VFS)
            if active_app == crate::desktop::ActiveApp::TextEditor {
                crate::desktop::DESKTOP.lock().handle_editor_key_event(ev);
                crate::desktop::render_desktop_frame();
                continue;
            }

            // Network Manager Wi-Fi passphrase input
            if active_app == crate::desktop::ActiveApp::NetworkManager {
                crate::desktop::DESKTOP.lock().handle_wifi_pass_key_event(ev);
                crate::desktop::render_desktop_frame();
                continue;
            }

            // File Manager keyboard navigation (Up/Down/Enter)
            if active_app == crate::desktop::ActiveApp::FileManager {
                let mut d = crate::desktop::DESKTOP.lock();
                match ev.key {
                    Key::Up => { if d.selected_file_idx > 0 { d.selected_file_idx -= 1; } }
                    Key::Down => { if d.selected_file_idx < 7 { d.selected_file_idx += 1; } }
                    Key::Home => { d.selected_file_idx = 0; }
                    Key::Enter => {
                        drop(d);
                        crate::desktop::launch_app(crate::desktop::ActiveApp::TextEditor);
                        crate::desktop::render_desktop_frame();
                        continue;
                    }
                    _ => {}
                }
                drop(d);
                crate::desktop::render_desktop_frame();
                continue;
            }

            // Games input
            if active_app == crate::desktop::ActiveApp::Games {
                let mapped = match ev.key {
                    Key::Up => Some('\x11'),
                    Key::Down => Some('\x12'),
                    Key::Left => Some('\x13'),
                    Key::Right => Some('\x14'),
                    Key::Enter => Some('\n'),
                    Key::Char(c) => Some(c),
                    _ => None,
                };
                if let Some(ch) = mapped {
                    crate::games::GAMES.lock().handle_key(ch);
                    crate::desktop::render_desktop_frame();
                }
                continue;
            }

            // Calculator keyboard input
            if active_app == crate::desktop::ActiveApp::Calculator {
                let s = match ev.key {
                    Key::Char('0') => Some("0"), Key::Char('1') => Some("1"),
                    Key::Char('2') => Some("2"), Key::Char('3') => Some("3"),
                    Key::Char('4') => Some("4"), Key::Char('5') => Some("5"),
                    Key::Char('6') => Some("6"), Key::Char('7') => Some("7"),
                    Key::Char('8') => Some("8"), Key::Char('9') => Some("9"),
                    Key::Char('+') => Some("+"), Key::Char('-') => Some("-"),
                    Key::Char('*') => Some("*"), Key::Char('/') => Some("/"),
                    Key::Char('=') | Key::Enter => Some("="),
                    Key::Char('c') | Key::Char('C') | Key::Backspace | Key::Delete => Some("C"),
                    _ => None,
                };
                if let Some(lbl) = s {
                    crate::desktop::DESKTOP.lock().calc_press(lbl);
                    crate::desktop::render_desktop_frame();
                }
                continue;
            }

            // Terminal input (only when Terminal window is focused)
            if active_app == crate::desktop::ActiveApp::Terminal {
                let mut shell = crate::shell::SHELL.lock();
                match ev.key {
                    Key::Enter => {
                        let len = shell.input_len;
                        let mut cmd_copy = [0u8; 128];
                        for i in 0..len {
                            cmd_copy[i] = shell.input_buf[i];
                        }
                        shell.input_len = 0;
                        if let Ok(safe_str) = core::str::from_utf8(&cmd_copy[..len]) {
                            shell.execute_command(safe_str);
                        }
                    }
                    Key::Backspace | Key::Delete => {
                        if shell.input_len > 0 {
                            shell.input_len -= 1;
                        }
                    }
                    Key::Char(ch) => {
                        let b = ch as u8;
                        if b >= 0x20 && b <= 0x7E && shell.input_len < 120 {
                            let len = shell.input_len;
                            shell.input_buf[len] = b;
                            shell.input_len += 1;
                        }
                    }
                    _ => {}
                }
                drop(shell);
                crate::desktop::render_desktop_frame();
            }
        }

        if progress % 10_000_000 == 0 {
            serial::write_str("[PREEMPT] TASK 1 RUNNING\n");
        }

        core::hint::spin_loop();
    }
}

extern "C" fn preempt_task2_entry() -> ! {
    serial::write_str("\n[PREEMPT] TASK 2 STARTED\n");

    let mut last_click = false;
    let mut last_mouse_x: usize = 9999;
    let mut last_mouse_y: usize = 9999;
    let mut right_click_handled = false;
    let mut render_ticks: u64 = 0;

    loop {
        // Hybrid IRQ + polling fallback for 8042 PS/2 & Legacy-USB HID
        crate::mouse::poll_8042();

        let mouse = crate::mouse::get_mouse_state();
        let mut needs_render = false;

        // --- Left click ---
        if mouse.left_button && !last_click {
            last_click = true;
            serial::write_str("[MOUSE] Left Click at (");
            serial::write_u64(mouse.x as u64);
            serial::write_str(", ");
            serial::write_u64(mouse.y as u64);
            serial::write_str(")\n");
            crate::desktop::handle_mouse_click(mouse.x, mouse.y);
            needs_render = true;
        } else if !mouse.left_button && last_click {
            last_click = false;
            // Release any active window drag
            crate::desktop::handle_mouse_move(mouse.x, mouse.y, false);
        }

        // --- Right click: desktop context menu ---
        if mouse.right_button && !right_click_handled {
            right_click_handled = true;
            serial::write_str("[MOUSE] Right Click at (");
            serial::write_u64(mouse.x as u64);
            serial::write_str(", ");
            serial::write_u64(mouse.y as u64);
            serial::write_str(")\n");
            crate::desktop::handle_right_click(mouse.x, mouse.y);
            needs_render = true;
        } else if !mouse.right_button {
            right_click_handled = false;
        }

        // --- Cursor movement & window dragging/resizing ---
        if mouse.x != last_mouse_x || mouse.y != last_mouse_y {
            last_mouse_x = mouse.x;
            last_mouse_y = mouse.y;
            let dragged_or_resized = crate::desktop::handle_mouse_move(mouse.x, mouse.y, mouse.left_button);
            if dragged_or_resized {
                needs_render = true;
            } else if !needs_render {
                // ULTRA-FAST dirty-rect cursor blit (~864 bytes instead of 8.3 MB -> zero lag!)
                crate::desktop::update_cursor_fast(mouse.x, mouse.y);
            }
        }

        // --- Scroll wheel ---
        let scroll = crate::mouse::take_scroll_delta();
        if scroll != 0 {
            needs_render = true;
        }

        if needs_render {
            crate::desktop::render_desktop_frame();
        }

        // --- Periodic refresh for live RTC clock & System Monitor (low overhead) ---
        render_ticks = render_ticks.wrapping_add(1);
        if render_ticks % 2_500_000 == 0 {
            crate::desktop::render_desktop_frame();
        }

        if render_ticks % 10_000_000 == 0 {
            serial::write_str("[PREEMPT] TASK 2 RUNNING\n");
        }

        core::hint::spin_loop();
    }
}

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let tsc_boot_start = unsafe { core::arch::x86_64::_rdtsc() };
    serial::init();
    x86_64::instructions::interrupts::disable();
    gdt::init();
    interrupts::init_idt();
    pic::initialize_pics();
    timer::init();
    logger::info("SRIKESH OS kernel started");
    memory::report(boot_info);
    cpu::report();
    let mut first_task = task::Task::new(1, 5);

    logger::info("Task created successfully");
    task::report(&first_task);

    first_task.set_state(task::TaskState::RUNNING);
    task::report(&first_task);

    first_task.set_state(task::TaskState::BLOCKED);
    task::report(&first_task);

    first_task.set_state(task::TaskState::TERMINATED);
    task::report(&first_task);

    logger::info("Running Host Storage Zero-Write Guard verification...");
    storage_guard::run_security_tests();

    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        let info = framebuffer.info();
        let buffer = framebuffer.buffer_mut();

        graphics::GRAPHICS.lock().init(buffer, &info);
        mouse::set_mouse_bounds(info.width, info.height);
        mouse::init_mouse();

        logger::info("Double-Buffered Graphics & PS/2 Input Subsystems initialized");
        startup::run_boot_sequence();
    }

    persistence::init_persistence();
    persistence::test_persistence_subsystem();

    pci::scan_pci_bus();
    e1000::init_e1000();

    logger::info("Starting PIT timer test...");
    x86_64::instructions::interrupts::enable();

    while interrupts::TIMER_TICKS.load(
        core::sync::atomic::Ordering::Relaxed
    ) < 100 {
        x86_64::instructions::hlt();
    }

    x86_64::instructions::interrupts::disable();
    logger::info("PIT timer test passed: received 100 ticks");
    net::test_network_subsystem();
    crate::sync::run_demo();
    scheduler_tests::run();
    ca6_tests::run_all_tests();
    memory::allocator_demo();

    spkg::init_packages();
    crate::shell::SHELL.lock().init();
    desktop::render_desktop_frame();

    // Flush boot-time spurious keyboard events before starting tasks
    crate::keyboard::flush_key_buffer();
    serial::write_str("[KEYBOARD] Boot-time key buffer flushed\n");

    let tsc_boot_end = unsafe { core::arch::x86_64::_rdtsc() };
    let tsc_delta = tsc_boot_end.saturating_sub(tsc_boot_start);
    serial::write_str("\n--- [SRIKESH OS HARDWARE BOOT-TIME TELEMETRY] ---\n");
    serial::write_str("  [TSC] Cycles from Kernel Entry to Desktop: ");
    serial::write_u64(tsc_delta);
    serial::write_str(" cycles\n");
    serial::write_str("  [PIT] Hardware Timer Tick Frequency     : 100 Hz (10 ms per tick)\n");
    serial::write_str("  [TSC] Approx Duration at nominal 3.0 GHz : ");
    serial::write_u64(tsc_delta / 3_000_000);
    serial::write_str(" ms\n");
    serial::write_str("---------------------------------------------------\n\n");

    logger::info("Initializing interrupt-driven task contexts");

// Initialize Task 1's interrupt stack.
let tcb1_ptr = core::ptr::addr_of_mut!(TASK1_TCB);
let tcb1_pin = unsafe {
    core::pin::Pin::new_unchecked(&mut *tcb1_ptr)
};

if tcb1_pin.initialize_interrupt_context().is_err() {
    halt_forever("[ERROR] Task 1 interrupt context initialization failed\n");
}

// Initialize Task 2's interrupt stack.
let tcb2_ptr = core::ptr::addr_of_mut!(TASK2_TCB);
let tcb2_pin = unsafe {
    core::pin::Pin::new_unchecked(&mut *tcb2_ptr)
};

if tcb2_pin.initialize_interrupt_context().is_err() {
    halt_forever("[ERROR] Task 2 interrupt context initialization failed\n");
}

logger::info("Both interrupt task contexts initialized");

unsafe {
    let frame = (*tcb1_ptr).interrupt_rsp as *const usize;

    serial::write_str("[DEBUG] TASK 1 FRAME WORDS:\n");

    for index in 15..20 {
        serial::write_str("[DEBUG] FRAME[");
        serial::write_hex(index as u64);
        serial::write_str("] = ");
        serial::write_hex(*frame.add(index) as u64);
        serial::write_str("\n");
    }

    use x86_64::instructions::segmentation::{Segment, CS};
    serial::write_str("[DEBUG] CURRENT CS = ");
    serial::write_hex(CS::get_reg().0 as u64);
    serial::write_str("\n");
}

unsafe {
    let task1_rsp = (*tcb1_ptr).interrupt_rsp;
    let task2_rsp = (*tcb2_ptr).interrupt_rsp;

    serial::write_str("[DEBUG] TASK 1 initial frame: ");
    serial::write_hex(task1_rsp as u64);
    serial::write_str("\n");

    serial::write_str("[DEBUG] TASK 1 expected post-IRET RSP: ");
    serial::write_hex(
        (task1_rsp + 20 * core::mem::size_of::<usize>()) as u64
    );
    serial::write_str("\n");

    serial::write_str("[DEBUG] TASK 2 initial frame: ");
    serial::write_hex(task2_rsp as u64);
    serial::write_str("\n");

    serial::write_str("[DEBUG] TASK 2 expected post-IRET RSP: ");
    serial::write_hex(
        (task2_rsp + 20 * core::mem::size_of::<usize>()) as u64
    );
    serial::write_str("\n");
}

// Prepare the ready queue and dispatch the first task.
let first_task = unsafe {
    let queue = &mut *core::ptr::addr_of_mut!(READY_QUEUE);

    let task1 = core::ptr::addr_of_mut!(TASK1_TCB);
    let task2 = core::ptr::addr_of_mut!(TASK2_TCB);

    if queue.enqueue(task1).is_err() {
        core::ptr::null_mut()
    } else if queue.enqueue(task2).is_err() {
        core::ptr::null_mut()
    } else {
        scheduler::dispatch_next(queue)
            .unwrap_or(core::ptr::null_mut())
    }
};

if first_task.is_null() {
    halt_forever("[ERROR] Initial preemptive dispatch failed\n");
}

PREEMPTION_SLICE.store(
    0,
    core::sync::atomic::Ordering::Relaxed
);

CURRENT_TASK.store(
    first_task,
    core::sync::atomic::Ordering::Relaxed
);

// Count the initial dispatch of the first task.
unsafe {
    match (*first_task).id {
        1 => { TASK1_DISPATCH_COUNT.fetch_add(1, core::sync::atomic::Ordering::Relaxed); }
        2 => { TASK2_DISPATCH_COUNT.fetch_add(1, core::sync::atomic::Ordering::Relaxed); }
        _ => {}
    }
}

logger::info("Initial task dispatched");
logger::info("Starting timer-driven Round Robin scheduler");

// The initial synthetic RFLAGS enables interrupts through IRETQ.
unsafe {
    context::start_interrupt_context((*first_task).interrupt_rsp);
}

}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
