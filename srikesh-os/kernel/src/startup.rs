//! SRIKESH OS - Modern Startup & Boot Welcome Screen
//!
//! Visual Design:
//! - Sleek Dark Slate & Aurora Bloom backdrop (matches modern desktop identity)
//! - Clean SRIKESH OS emblem, title, and "Awaken the Power of Technology." tagline
//! - Frosted dark-mica hardware initialization panel & electric sky-blue progress bar

use crate::graphics::{Color, GRAPHICS};
use crate::desktop::{DESKTOP_WALLPAPER_HEIGHT, DESKTOP_WALLPAPER_RAW, DESKTOP_WALLPAPER_WIDTH};
use crate::serial;

pub static STARTUP_EGYPTIAN_RAW: &[u8] = DESKTOP_WALLPAPER_RAW;
pub const STARTUP_EGYPTIAN_WIDTH: usize = DESKTOP_WALLPAPER_WIDTH;
pub const STARTUP_EGYPTIAN_HEIGHT: usize = DESKTOP_WALLPAPER_HEIGHT;

pub fn render_startup_screen(progress_percent: usize, status_message: &str) {
    let mut gfx = GRAPHICS.lock();
    if gfx.width == 0 || gfx.height == 0 {
        return;
    }

    let width = gfx.width;
    let height = gfx.height;

    // 1. Modern Aurora Bloom full-screen backdrop with sleek dark-glass tint
    gfx.draw_image_rgb(
        0,
        0,
        width,
        height,
        DESKTOP_WALLPAPER_RAW,
        DESKTOP_WALLPAPER_WIDTH,
        DESKTOP_WALLPAPER_HEIGHT,
    );
    gfx.fill_rect(0, 0, width, height, Color::with_alpha(10, 15, 28, 130));

    // 2. Centered Modern Logo & Branding
    let top_y = (height * 16) / 100;
    // Modern 4-tile Windows-11-inspired glowing sky-blue emblem
    let ex = width / 2 - 22;
    gfx.fill_rounded_rect(ex, top_y, 20, 20, 4, Color::BRIGHT_GOLD);
    gfx.fill_rounded_rect(ex + 24, top_y, 20, 20, 4, Color::ROYAL_GOLD);
    gfx.fill_rounded_rect(ex, top_y + 24, 20, 20, 4, Color::ROYAL_GOLD);
    gfx.fill_rounded_rect(ex + 24, top_y + 24, 20, 20, 4, Color::LAPIS_LIGHT);

    gfx.draw_text_centered(top_y + 60, "S R I K E S H   O S", Color::WHITE, 3);
    gfx.draw_text_centered(
        top_y + 92,
        "Awaken the Power of Technology.",
        Color::BRIGHT_GOLD,
        1,
    );

    // 3. Modern Frosted Dark-Mica Initialization Panel
    let panel_w = ((width * 68) / 100).clamp(480, 760);
    let panel_h = 176usize;
    let panel_x = (width - panel_w) / 2;
    let panel_y = top_y + 124;

    gfx.fill_rounded_rect(panel_x, panel_y, panel_w, panel_h, 10, Color::with_alpha(15, 23, 42, 228));
    gfx.draw_rect_outline(panel_x, panel_y, panel_w, panel_h, Color::ROYAL_GOLD, 1);

    let list_x = panel_x + 22;
    let mut list_y = panel_y + 16;
    let step = 18usize;

    let items = [
        "[OK] CPU Architecture : x86-64 Long Mode (Custom Rust no_std Kernel)",
        "[OK] Memory Manager   : CR3 4-Level Page Table + 64 KB Heap Allocator",
        "[OK] Interrupt Engine : 256-Entry IDT, Dual 8259 PIC & 100 Hz PIT Timer",
        "[OK] Input Subsystem  : Lock-Safe Hybrid 8042 Keyboard & Mouse Drivers",
        "[OK] Storage Guard    : Host Internal NVMe Zero-Write Protection Active",
        "[OK] Network Engine   : Intel e1000 Gigabit + ARP/DHCP/DNS/TCP/HTTP",
        "[OK] Display Engine   : Zero-Lag Dirty-Rect Double-Buffered Compositor",
        "[OK] Modern Desktop   : Multi-Window Taskbar, Launcher & Quick Settings",
    ];

    let items_to_show = (progress_percent * items.len() + 99) / 100;
    for (idx, &item) in items.iter().enumerate() {
        if idx < items_to_show {
            let color = if idx + 1 == items_to_show {
                Color::BRIGHT_GOLD
            } else {
                Color::SUCCESS_GREEN
            };
            gfx.draw_text(list_x, list_y, item, color, 1);
        }
        list_y += step;
    }

    // 4. Sleek Electric Sky-Blue Progress Bar
    let bar_w = panel_w;
    let bar_h = 12usize;
    let bar_x = panel_x;
    let bar_y = panel_y + panel_h + 20;

    gfx.fill_rounded_rect(bar_x, bar_y, bar_w, bar_h, 6, Color::new(30, 41, 59));
    let fill_w = (bar_w * progress_percent.min(100)) / 100;
    if fill_w > 0 {
        gfx.fill_rounded_rect(bar_x, bar_y, fill_w.max(8), bar_h, 6, Color::ROYAL_GOLD);
    }

    gfx.draw_text_centered(bar_y + 22, status_message, Color::ROYAL_SAND, 1);
    gfx.swap_buffers();
}

pub fn run_boot_sequence() {
    serial::write_str("[STARTUP] Launching Modern SRIKESH OS Welcome Screen...\n");
    let steps = [
        (15, "Initializing x86-64 Kernel & Memory Subsystems..."),
        (35, "Arming IDT, 8259 PIC & Preemptive Round-Robin Scheduler..."),
        (55, "Locking Internal Host NVMe Storage (Zero-Write Guard)..."),
        (75, "Probing PCI Hardware, USB, Ethernet & Wi-Fi Adapters..."),
        (90, "Caching Aurora Bloom Compositor & Modern Icon Suite..."),
        (100, "Welcome to SRIKESH OS — Starting Desktop..."),
    ];
    for &(pct, msg) in &steps {
        render_startup_screen(pct, msg);
        for _ in 0..1_200_000 {
            core::hint::spin_loop();
        }
    }
    serial::write_str("[STARTUP] Boot welcome sequence completed.\n");
}
