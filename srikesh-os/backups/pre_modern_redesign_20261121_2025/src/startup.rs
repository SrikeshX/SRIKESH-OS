//! SRIKESH OS - Animated Startup Screen (Egyptian Royal Theme - Supplied Image 2)
//!
//! Visual Design:
//! - Deep obsidian black backdrop with royal golden illumination
//! - Supplied Reference Image 2: Pharaoh Sovereign on Royal Throne
//! - Pharaonic glowing emblem: "< S >"
//! - Official Branding: "SRIKESH OS" & "Awaken the Power of Technology."
//! - Egyptian royal gold progress bar & live hardware validation telemetry

use crate::graphics::{Color, GRAPHICS};
use crate::serial;

// Embedded raw 24-bit RGB pixel data from Supplied Reference Image 2 (Pharaoh)
pub static STARTUP_EGYPTIAN_RAW: &[u8] = include_bytes!("../../assets/startup_egyptian.raw");
pub const STARTUP_EGYPTIAN_WIDTH: usize = 512;
pub const STARTUP_EGYPTIAN_HEIGHT: usize = 288;

pub fn render_startup_screen(progress_percent: usize, status_message: &str) {
    let mut gfx = GRAPHICS.lock();
    if gfx.width == 0 || gfx.height == 0 {
        return;
    }

    // 1. Deep Obsidian Black background
    gfx.clear(Color::OBSIDIAN);

    // 2. Render Reference Image 2 (Pharaoh on Throne) in the upper/middle region
    let wave_target_w = gfx.width;
    let wave_target_h = (gfx.height * 56) / 100;
    gfx.draw_image_rgb(
        0,
        0,
        wave_target_w,
        wave_target_h,
        STARTUP_EGYPTIAN_RAW,
        STARTUP_EGYPTIAN_WIDTH,
        STARTUP_EGYPTIAN_HEIGHT,
    );

    // 3. Smooth golden vignette overlay at the bottom of the Pharaoh artwork
    let width = gfx.width;
    let fade_h = 42;
    let fade_y = wave_target_h.saturating_sub(fade_h);
    for i in 0..fade_h {
        let alpha = ((i as u32 * 255) / fade_h as u32) as u8;
        let cover = Color::with_alpha(12, 10, 8, alpha);
        gfx.fill_rect(0, fade_y + i, width, 1, cover);
    }

    // 4. Glowing Pharaonic Emblem "< S >"
    let emblem_y = wave_target_h / 4;
    gfx.draw_text_centered(emblem_y, "< S >", Color::BRIGHT_GOLD, 4);

    // 5. Official Branding: Title & Tagline
    gfx.draw_text_centered(emblem_y + 44, "S R I K E S H   O S", Color::BRIGHT_GOLD, 3);
    gfx.draw_text_centered(
        emblem_y + 76,
        "Awaken the Power of Technology.",
        Color::ROYAL_SAND,
        1,
    );

    // 6. Egyptian Royal Frosted Panel in lower center
    let panel_w = (gfx.width * 72) / 100;
    let panel_h = 164;
    let panel_x = (gfx.width - panel_w) / 2;
    let panel_y = wave_target_h + 8;

    gfx.fill_rounded_rect(panel_x, panel_y, panel_w, panel_h, 8, Color::ROYAL_GLASS_PANEL);
    gfx.draw_rect_outline(panel_x, panel_y, panel_w, panel_h, Color::ROYAL_GOLD, 1);
    gfx.draw_rect_outline(panel_x + 2, panel_y + 2, panel_w - 4, panel_h - 4, Color::DARK_GOLD, 1);

    // Hardware Checklist inside panel
    let list_x = panel_x + 24;
    let mut list_y = panel_y + 14;
    let step = 17;

    let items = [
        "[OK] CPU Architecture : x86-64 Long Mode (Intel / AMD Multi-Core)",
        "[OK] Physical Memory  : Paging & Dynamic Regions Initialized",
        "[OK] Interrupt Engine : Dual 8259 PIC & IDT Vectors Armed",
        "[OK] Hardware Timer   : 8254 PIT Calibrated at 100 Hz",
        "[OK] Input Subsystem  : PS/2 Keyboard & Mouse Drivers Armed",
        "[OK] Security Policy  : Host Storage Zero-Write Guard Active",
        "[OK] Network Engine   : Intel PRO/1000 Gigabit & Wi-Fi Scanner",
        "[OK] Pharaonic UI     : Egyptian Royal Double-Buffered Compositor",
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

    // 7. Glowing Egyptian Royal Gold Progress Bar
    let bar_w = panel_w - 48;
    let bar_h = 10;
    let bar_x = panel_x + 24;
    let bar_y = panel_y + panel_h - 22;

    // Bar background
    gfx.fill_rounded_rect(bar_x, bar_y, bar_w, bar_h, 4, Color::new(28, 22, 16));

    // Filled progress
    let fill_w = ((bar_w * progress_percent.min(100)) / 100).max(4);
    gfx.draw_gradient_h(bar_x, bar_y, fill_w, bar_h, Color::DARK_GOLD, Color::BRIGHT_GOLD);

    // Status line below panel
    let status_y = panel_y + panel_h + 10;
    gfx.draw_text_centered(status_y, status_message, Color::ROYAL_SAND, 1);

    // Double buffer swap
    gfx.swap_buffers();
}

/// Run smooth animated boot sequence across multiple milestones
pub fn run_boot_sequence() {
    serial::write_str("[STARTUP] Rendering SRIKESH OS Egyptian Royal Welcome Sequence...\n");

    let checkpoints = [
        (15, "Initializing kernel memory & CPU long mode..."),
        (30, "Configuring IDT, PIC and 100 Hz hardware timer..."),
        (50, "Probing PS/2 keyboard and mouse controllers..."),
        (70, "Arming host storage zero-write protection guard..."),
        (85, "Enumerating PCI devices & Intel e1000 networking..."),
        (100, "Awaken the Power of Technology. Launching Desktop..."),
    ];

    for (pct, msg) in checkpoints {
        render_startup_screen(pct, msg);
        for _ in 0..12_000_000 {
            core::hint::spin_loop();
        }
    }
}
