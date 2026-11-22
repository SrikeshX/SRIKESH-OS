//! SRIKESH OS - Professional Graphical Application & System Tray Icons
//!
//! Renders recognizable, Windows-11-inspired vector/geometric icons styled in
//! Egyptian Royal Black, Gold, Lapis Blue, and Emerald accents — zero plain-text boxes.

use crate::graphics::{Color, GraphicsEngine};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppIconKind {
    StartLauncher,
    Files,
    Terminal,
    Browser,
    Settings,
    SystemMonitor,
    NetworkManager,
    TextEditor,
    Calculator,
    ImageViewer,
    Games,
    SoftwareStore,
    About,
    Power,
}

fn fill_circle(gfx: &mut GraphicsEngine, cx: isize, cy: isize, r: isize, color: Color) {
    let r2 = r * r;
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy <= r2 {
                let px = cx + dx;
                let py = cy + dy;
                if px >= 0 && py >= 0 {
                    gfx.set_pixel_blend(px as usize, py as usize, color);
                }
            }
        }
    }
}

fn draw_circle_ring(gfx: &mut GraphicsEngine, cx: isize, cy: isize, r_outer: isize, r_inner: isize, color: Color) {
    let ro2 = r_outer * r_outer;
    let ri2 = r_inner * r_inner;
    for dy in -r_outer..=r_outer {
        for dx in -r_outer..=r_outer {
            let d2 = dx * dx + dy * dy;
            if d2 <= ro2 && d2 >= ri2 {
                let px = cx + dx;
                let py = cy + dy;
                if px >= 0 && py >= 0 {
                    gfx.set_pixel_blend(px as usize, py as usize, color);
                }
            }
        }
    }
}

fn draw_line(gfx: &mut GraphicsEngine, x0: isize, y0: isize, x1: isize, y1: isize, color: Color, thickness: isize) {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut cx = x0;
    let mut cy = y0;

    for _ in 0..256 {
        for ty in 0..thickness {
            for tx in 0..thickness {
                let px = cx + tx;
                let py = cy + ty;
                if px >= 0 && py >= 0 {
                    gfx.set_pixel_blend(px as usize, py as usize, color);
                }
            }
        }
        if cx == x1 && cy == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            cx += sx;
        }
        if e2 <= dx {
            err += dx;
            cy += sy;
        }
    }
}

/// Renders a crisp graphical application icon inside a `size x size` square at `(x, y)`.
/// Designed for sizes 24..=48 (standard: 28 for taskbar, 36 for desktop/launcher).
pub fn draw_app_icon(gfx: &mut GraphicsEngine, kind: AppIconKind, x: usize, y: usize, size: usize) {
    let s = size.max(16);
    let cx = (x + s / 2) as isize;
    let cy = (y + s / 2) as isize;

    match kind {
        AppIconKind::StartLauncher => {
            // Windows-11 inspired 4-tile royal gold grid
            let pad = s / 6;
            let gap = (s / 14).max(2);
            let tile = (s - pad * 2 - gap) / 2;
            let x0 = x + pad;
            let y0 = y + pad;
            gfx.fill_rounded_rect(x0, y0, tile, tile, 2, Color::BRIGHT_GOLD);
            gfx.fill_rounded_rect(x0 + tile + gap, y0, tile, tile, 2, Color::ROYAL_GOLD);
            gfx.fill_rounded_rect(x0, y0 + tile + gap, tile, tile, 2, Color::ROYAL_GOLD);
            gfx.fill_rounded_rect(x0 + tile + gap, y0 + tile + gap, tile, tile, 2, Color::ROYAL_AMBER);
        }

        AppIconKind::Files => {
            // Golden 3D folder icon with tab and highlight
            let pad = s / 10;
            let fw = s - pad * 2;
            let fh = (s * 68) / 100;
            let fx = x + pad;
            let fy = y + s - pad - fh;
            // Folder back tab
            let tab_w = (fw * 44) / 100;
            let tab_h = (s * 18) / 100;
            gfx.fill_rounded_rect(fx, fy.saturating_sub(tab_h / 2), tab_w, tab_h + 4, 3, Color::DARK_GOLD);
            // Back plate
            gfx.fill_rounded_rect(fx, fy, fw, fh, 4, Color::new(190, 145, 30));
            // Inner paper sheet peek
            gfx.fill_rect(fx + 3, fy + 2, fw - 6, 3, Color::ROYAL_SAND);
            // Front folder flap
            let flap_y = fy + fh / 4;
            let flap_h = fh - fh / 4;
            gfx.fill_rounded_rect(fx, flap_y, fw, flap_h, 4, Color::BRIGHT_GOLD);
            // Subtle bottom shadow stripe
            gfx.fill_rect(fx + 2, fy + fh - 3, fw - 4, 2, Color::DARK_GOLD);
        }

        AppIconKind::Terminal => {
            // Dark rounded terminal frame with gold title bar and crisp ">_" prompt
            let pad = s / 12;
            let w = s - pad * 2;
            let h = (s * 80) / 100;
            let rx = x + pad;
            let ry = y + (s - h) / 2;
            gfx.fill_rounded_rect(rx, ry, w, h, 4, Color::new(18, 18, 22));
            gfx.draw_rect_outline(rx, ry, w, h, Color::ROYAL_GOLD, 1);
            // Top bar
            gfx.fill_rect(rx + 1, ry + 1, w - 2, (h / 5).max(4), Color::new(48, 38, 22));
            // Prompt chevron '>'
            let px = (rx + w / 5) as isize;
            let py = (ry + h / 2 + 2) as isize;
            let d = (s / 8).max(3) as isize;
            draw_line(gfx, px, py - d, px + d, py, Color::BRIGHT_GOLD, 2);
            draw_line(gfx, px + d, py, px, py + d, Color::BRIGHT_GOLD, 2);
            // Underscore '_'
            gfx.fill_rect((px + d + 4) as usize, (py + d - 1) as usize, (d as usize * 2).max(5), 2, Color::SUCCESS_GREEN);
        }

        AppIconKind::Browser => {
            // Royal Lapis Blue globe with golden latitude & longitude rings
            let r = (s as isize * 42) / 100;
            fill_circle(gfx, cx, cy, r, Color::new(26, 86, 178));
            fill_circle(gfx, cx - r / 4, cy - r / 4, r / 2, Color::new(50, 130, 220));
            draw_circle_ring(gfx, cx, cy, r, r - 2, Color::BRIGHT_GOLD);
            // Equator & meridian lines
            gfx.fill_rect((cx - r + 2) as usize, cy as usize, (r * 2 - 4).max(1) as usize, 2, Color::ROYAL_GOLD);
            gfx.fill_rect(cx as usize, (cy - r + 2) as usize, 2, (r * 2 - 4).max(1) as usize, Color::ROYAL_GOLD);
            // Upper & lower latitude lines
            let lat_r = (r * 7) / 10;
            gfx.fill_rect((cx - lat_r) as usize, (cy - r / 2) as usize, (lat_r * 2).max(1) as usize, 1, Color::ROYAL_SAND);
            gfx.fill_rect((cx - lat_r) as usize, (cy + r / 2) as usize, (lat_r * 2).max(1) as usize, 1, Color::ROYAL_SAND);
        }

        AppIconKind::Settings => {
            // Precision metallic gold gear with 8 teeth and center axle hole
            let r_outer = (s as isize * 38) / 100;
            let r_inner = (s as isize * 16) / 100;
            let tooth = (s / 7).max(4);
            let half_t = (tooth / 2) as isize;
            // 4 cardinal teeth
            gfx.fill_rect((cx - half_t) as usize, (cy - r_outer - 3) as usize, tooth, (r_outer * 2 + 6) as usize, Color::ROYAL_GOLD);
            gfx.fill_rect((cx - r_outer - 3) as usize, (cy - half_t) as usize, (r_outer * 2 + 6) as usize, tooth, Color::ROYAL_GOLD);
            // 4 diagonal teeth
            let diag = (r_outer * 72) / 100;
            for &(dx, dy) in &[(-diag, -diag), (diag, -diag), (-diag, diag), (diag, diag)] {
                fill_circle(gfx, cx + dx, cy + dy, (tooth as isize / 2).max(2), Color::ROYAL_GOLD);
            }
            // Gear body & dark center hole
            fill_circle(gfx, cx, cy, r_outer - 1, Color::BRIGHT_GOLD);
            draw_circle_ring(gfx, cx, cy, r_outer - 1, r_outer - 3, Color::DARK_GOLD);
            fill_circle(gfx, cx, cy, r_inner, Color::new(22, 18, 14));
        }

        AppIconKind::SystemMonitor => {
            // Dark telemetry screen with glowing emerald ECG pulse waveform
            let pad = s / 10;
            let w = s - pad * 2;
            let h = (s * 72) / 100;
            let rx = x + pad;
            let ry = y + pad;
            gfx.fill_rounded_rect(rx, ry, w, h, 4, Color::new(14, 24, 22));
            gfx.draw_rect_outline(rx, ry, w, h, Color::ROYAL_GOLD, 1);
            // Monitor stand
            gfx.fill_rect(x + s / 2 - 2, ry + h, 4, 4, Color::DARK_GOLD);
            gfx.fill_rect(x + s / 3, ry + h + 3, s / 3, 2, Color::ROYAL_GOLD);
            // Pulse waveform
            let mid_y = (ry + h / 2 + 1) as isize;
            let x1 = (rx + 3) as isize;
            let x2 = (rx + w / 3) as isize;
            let x3 = (rx + w / 2 - 2) as isize;
            let x4 = (rx + w / 2 + 3) as isize;
            let x5 = (rx + (w * 2) / 3) as isize;
            let x6 = (rx + w - 4) as isize;
            let amp = (h / 3) as isize;
            draw_line(gfx, x1, mid_y, x2, mid_y, Color::SUCCESS_GREEN, 2);
            draw_line(gfx, x2, mid_y, x3, mid_y - amp, Color::BRIGHT_GOLD, 2);
            draw_line(gfx, x3, mid_y - amp, x4, mid_y + amp - 1, Color::SUCCESS_GREEN, 2);
            draw_line(gfx, x4, mid_y + amp - 1, x5, mid_y, Color::SUCCESS_GREEN, 2);
            draw_line(gfx, x5, mid_y, x6, mid_y, Color::SUCCESS_GREEN, 2);
        }

        AppIconKind::NetworkManager => {
            // Crisp Wi-Fi / Signal waves over royal dark medallion
            let r = (s as isize * 44) / 100;
            fill_circle(gfx, cx, cy, r, Color::new(18, 28, 48));
            draw_circle_ring(gfx, cx, cy, r, r - 1, Color::ROYAL_GOLD);
            let base_y = cy + r / 2;
            fill_circle(gfx, cx, base_y, 2, Color::BRIGHT_GOLD);
            // 3 upper arcs
            for &(arc_r, col) in &[
                (r / 3, Color::BRIGHT_GOLD),
                ((r * 55) / 100, Color::SUCCESS_GREEN),
                ((r * 78) / 100, Color::ROYAL_SAND),
            ] {
                for dy in -arc_r..=0 {
                    for dx in -arc_r..=arc_r {
                        let d2 = dx * dx + dy * dy;
                        if d2 <= arc_r * arc_r && d2 >= (arc_r - 2) * (arc_r - 2) && dx.abs() <= -dy + 2 {
                            let px = cx + dx;
                            let py = base_y + dy;
                            if px >= 0 && py >= 0 {
                                gfx.set_pixel(px as usize, py as usize, col);
                            }
                        }
                    }
                }
            }
        }

        AppIconKind::TextEditor => {
            // Warm sandstone/ivory document page with ruled lines & golden pen
            let pad = s / 8;
            let w = (s * 68) / 100;
            let h = s - pad * 2;
            let rx = x + (s - w) / 2;
            let ry = y + pad;
            gfx.fill_rounded_rect(rx, ry, w, h, 3, Color::new(242, 232, 208));
            gfx.draw_rect_outline(rx, ry, w, h, Color::ROYAL_GOLD, 1);
            // Ruled text lines
            for i in 0..4 {
                let ly = ry + 6 + i * ((h - 10) / 4).max(4);
                let lw = if i == 3 { (w * 5) / 10 } else { w - 8 };
                gfx.fill_rect(rx + 4, ly, lw, 2, Color::new(80, 68, 52));
            }
            // Golden stylus diagonal accent at bottom-right
            draw_line(
                gfx,
                (rx + w - 8) as isize,
                (ry + h - 4) as isize,
                (rx + w + 2) as isize,
                (ry + h / 2) as isize,
                Color::BRIGHT_GOLD,
                3,
            );
        }

        AppIconKind::Calculator => {
            // Obsidian calculator body with glowing screen and 3x3 keypad
            let pad = s / 8;
            let w = (s * 72) / 100;
            let h = s - pad * 2;
            let rx = x + (s - w) / 2;
            let ry = y + pad;
            gfx.fill_rounded_rect(rx, ry, w, h, 4, Color::new(34, 30, 26));
            gfx.draw_rect_outline(rx, ry, w, h, Color::ROYAL_GOLD, 1);
            // LCD display
            let lcd_h = (h * 24) / 100;
            gfx.fill_rect(rx + 3, ry + 3, w - 6, lcd_h, Color::new(24, 92, 68));
            // 3x3 keypad buttons
            let grid_y = ry + lcd_h + 6;
            let btn_s = ((w - 10) / 3).max(3);
            for row in 0..3 {
                for col in 0..3 {
                    let bx = rx + 3 + col * (btn_s + 2);
                    let by = grid_y + row * (btn_s + 2);
                    let col_btn = if row == 2 && col == 2 {
                        Color::BRIGHT_GOLD
                    } else {
                        Color::new(95, 86, 72)
                    };
                    if by + btn_s < ry + h - 1 {
                        gfx.fill_rect(bx, by, btn_s, btn_s, col_btn);
                    }
                }
            }
        }

        AppIconKind::ImageViewer => {
            // Golden gallery frame with Egyptian night sky, golden sun, and twin pyramids
            let pad = s / 10;
            let w = s - pad * 2;
            let h = (s * 78) / 100;
            let rx = x + pad;
            let ry = y + (s - h) / 2;
            gfx.fill_rounded_rect(rx, ry, w, h, 3, Color::LAPIS_LAZULI);
            gfx.draw_rect_outline(rx, ry, w, h, Color::BRIGHT_GOLD, 1);
            // Sun
            fill_circle(gfx, (rx + w - w / 4) as isize, (ry + h / 4 + 1) as isize, (s / 10).max(2) as isize, Color::BRIGHT_GOLD);
            // Twin golden pyramids
            let base_y = ry + h - 3;
            let pyr_h = h / 2;
            for row in 0..pyr_h {
                let half = (row * (w / 3)) / pyr_h.max(1);
                let px1 = rx + w / 3;
                gfx.fill_rect(px1.saturating_sub(half), base_y - pyr_h + row, half * 2 + 1, 1, Color::ROYAL_GOLD);
                let px2 = rx + (w * 2) / 3;
                let half2 = (row * (w / 4)) / pyr_h.max(1);
                if row > 2 {
                    gfx.fill_rect(px2.saturating_sub(half2), base_y - pyr_h + row, half2 * 2 + 1, 1, Color::DARK_GOLD);
                }
            }
        }

        AppIconKind::Games => {
            // Royal Gamepad icon with D-pad & colored action buttons
            let pad = s / 10;
            let w = s - pad * 2;
            let h = (s * 56) / 100;
            let rx = x + pad;
            let ry = y + (s - h) / 2;
            gfx.fill_rounded_rect(rx, ry, w, h, 7, Color::new(56, 36, 84));
            gfx.draw_rect_outline(rx, ry, w, h, Color::BRIGHT_GOLD, 1);
            // D-pad on left
            let dx = rx + w / 4;
            let dy = ry + h / 2;
            gfx.fill_rect(dx.saturating_sub(4), dy.saturating_sub(1), 9, 3, Color::BRIGHT_GOLD);
            gfx.fill_rect(dx.saturating_sub(1), dy.saturating_sub(4), 3, 9, Color::BRIGHT_GOLD);
            // Action buttons on right
            let ax = rx + (w * 3) / 4;
            fill_circle(gfx, ax as isize, (dy - 3) as isize, 2, Color::BRIGHT_GOLD);
            fill_circle(gfx, ax as isize, (dy + 3) as isize, 2, Color::SUCCESS_GREEN);
            fill_circle(gfx, (ax - 3) as isize, dy as isize, 2, Color::ROYAL_AMBER);
            fill_circle(gfx, (ax + 3) as isize, dy as isize, 2, Color::DANGER_RED);
        }

        AppIconKind::SoftwareStore => {
            // Golden shopping bag with top handle
            let pad = s / 8;
            let w = (s * 72) / 100;
            let h = (s * 64) / 100;
            let rx = x + (s - w) / 2;
            let ry = y + s - pad - h;
            // Bag handle
            let hw = w / 2;
            let hx = rx + (w - hw) / 2;
            gfx.draw_rect_outline(hx, ry.saturating_sub(5), hw, 7, Color::BRIGHT_GOLD, 2);
            // Bag body
            gfx.fill_rounded_rect(rx, ry, w, h, 4, Color::new(42, 76, 142));
            gfx.draw_rect_outline(rx, ry, w, h, Color::ROYAL_GOLD, 1);
            // Golden royal emblem stripe
            gfx.fill_rect(rx + w / 4, ry + h / 2 - 2, w / 2, 4, Color::BRIGHT_GOLD);
        }

        AppIconKind::About => {
            // Royal gold medallion with crisp 'i'
            let r = (s as isize * 42) / 100;
            fill_circle(gfx, cx, cy, r, Color::ROYAL_GOLD);
            fill_circle(gfx, cx, cy, r - 2, Color::new(32, 24, 14));
            // Dot and stem of 'i'
            fill_circle(gfx, cx, cy - r / 2 + 2, 2, Color::BRIGHT_GOLD);
            gfx.fill_rect((cx - 1) as usize, (cy - 2) as usize, 3, (r as usize).max(6) - 2, Color::BRIGHT_GOLD);
        }

        AppIconKind::Power => {
            let r = (s as isize * 38) / 100;
            draw_circle_ring(gfx, cx, cy + 1, r, r - 2, Color::DANGER_RED);
            gfx.fill_rect((cx - 1) as usize, (cy - r - 1) as usize, 3, (r as usize).max(5), Color::BRIGHT_GOLD);
        }
    }
}

/// System Tray 16x16 Wi-Fi / Network indicator icon (honest connected vs offline state)
pub fn draw_tray_network(gfx: &mut GraphicsEngine, x: usize, y: usize, connected: bool) {
    let cx = (x + 8) as isize;
    let base_y = (y + 13) as isize;
    let col = if connected { Color::SUCCESS_GREEN } else { Color::TEXT_MUTED };
    fill_circle(gfx, cx, base_y, 1, col);
    for &arc_r in &[4isize, 7, 10] {
        for dy in -arc_r..=0 {
            for dx in -arc_r..=arc_r {
                let d2 = dx * dx + dy * dy;
                if d2 <= arc_r * arc_r && d2 >= (arc_r - 1) * (arc_r - 1) && dx.abs() <= -dy + 1 {
                    let px = cx + dx;
                    let py = base_y + dy;
                    if px >= 0 && py >= 0 {
                        gfx.set_pixel(px as usize, py as usize, col);
                    }
                }
            }
        }
    }
    if !connected {
        // Honest red diagonal slash when offline
        draw_line(gfx, (x + 2) as isize, (y + 2) as isize, (x + 14) as isize, (y + 14) as isize, Color::DANGER_RED, 1);
    }
}

/// System Tray 16x16 Volume speaker icon
pub fn draw_tray_volume(gfx: &mut GraphicsEngine, x: usize, y: usize, muted: bool) {
    let col = if muted { Color::TEXT_MUTED } else { Color::ROYAL_SAND };
    // Speaker box
    gfx.fill_rect(x + 2, y + 6, 3, 5, col);
    // Speaker cone
    gfx.fill_rect(x + 5, y + 5, 2, 7, col);
    gfx.fill_rect(x + 7, y + 3, 2, 11, col);
    if muted {
        draw_line(gfx, (x + 11) as isize, (y + 5) as isize, (x + 15) as isize, (y + 11) as isize, Color::DANGER_RED, 1);
        draw_line(gfx, (x + 15) as isize, (y + 5) as isize, (x + 11) as isize, (y + 11) as isize, Color::DANGER_RED, 1);
    } else {
        gfx.fill_rect(x + 11, y + 6, 1, 5, Color::BRIGHT_GOLD);
        gfx.fill_rect(x + 13, y + 4, 1, 9, Color::ROYAL_GOLD);
    }
}

/// System Tray 22x14 Battery icon with fill level
pub fn draw_tray_battery(gfx: &mut GraphicsEngine, x: usize, y: usize, pct: usize) {
    gfx.draw_rect_outline(x, y + 2, 18, 11, Color::ROYAL_SAND, 1);
    // Positive terminal cap
    gfx.fill_rect(x + 18, y + 5, 2, 5, Color::ROYAL_SAND);
    let fill_w = ((pct.min(100) * 14) / 100).max(2);
    let fill_col = if pct > 30 { Color::SUCCESS_GREEN } else { Color::ROYAL_AMBER };
    gfx.fill_rect(x + 2, y + 4, fill_w, 7, fill_col);
}
