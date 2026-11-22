//! SRIKESH OS - Egyptian Royal Windows-Inspired Desktop Environment & Multi-Window Compositor
//!
//! Architecture:
//! - Fullscreen Supplied Reference Image 1 (Anubis / Jackal on Golden Throne)
//! - Top Royal Status Bar: Branding, Honest Network State, RAM, Boot Counter, Power Trigger
//! - Multi-Window Manager (up to 6 simultaneous windows with z-order, focus, dragging,
//!   Minimize, Maximize/Restore, and Close)
//! - Windows-11-Inspired Bottom Taskbar: Start/App Launcher button, Pinned Graphical Icons,
//!   Running App Indicators, and System Tray (Network, Volume, Battery, Live RTC Time & Date)
//! - Windows-11 / Slax-Inspired Application Launcher with Categories, Live Search & Graphical Grid
//! - Desktop Graphical Icons with Single-Click Select & Double-Click Launch
//! - Right-Click Desktop Context Menu

use crate::graphics::{Color, GraphicsEngine, GRAPHICS};
use crate::icons::{self, AppIconKind};
use crate::mouse::get_mouse_state;
use crate::shell::SHELL;
use crate::vfs::VFS;
use crate::persistence::CONFIG;
use crate::browser::BROWSER;
use crate::games::{GameType, GAMES};
use crate::pci::PCI;
use crate::e1000;
use crate::net;
use crate::acpi;
use crate::startup::{STARTUP_EGYPTIAN_HEIGHT, STARTUP_EGYPTIAN_RAW, STARTUP_EGYPTIAN_WIDTH};
use core::sync::atomic::Ordering;

// Supplied Reference Image 1: Anubis / Jackal Warrior on Golden Throne
pub static DESKTOP_WALLPAPER_RAW: &[u8] = include_bytes!("../../assets/desktop_egyptian.raw");
pub const DESKTOP_WALLPAPER_WIDTH: usize = 512;
pub const DESKTOP_WALLPAPER_HEIGHT: usize = 288;

pub const TOP_BAR_H: usize = 28;
pub const TASKBAR_H: usize = 48;
pub const MAX_WINDOWS: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveApp {
    DesktopOnly,
    AppLauncher,
    Browser,
    Terminal,
    FileManager,
    TextEditor,
    Calculator,
    ImageViewer,
    NetworkManager,
    Games,
    SystemMonitor,
    Settings,
    SoftwareStore,
    About,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LauncherCategory {
    All,
    Productivity,
    System,
    Games,
}

#[derive(Clone, Copy)]
pub struct WindowSlot {
    pub open: bool,
    pub app: ActiveApp,
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
    pub saved_x: usize,
    pub saved_y: usize,
    pub saved_w: usize,
    pub saved_h: usize,
    pub minimized: bool,
    pub maximized: bool,
    pub z: u32,
}

impl WindowSlot {
    pub const fn empty() -> Self {
        Self {
            open: false,
            app: ActiveApp::DesktopOnly,
            x: 80,
            y: 48,
            w: 720,
            h: 460,
            saved_x: 80,
            saved_y: 48,
            saved_w: 720,
            saved_h: 460,
            minimized: false,
            maximized: false,
            z: 0,
        }
    }
}

pub struct DesktopManager {
    pub active_app: ActiveApp,
    pub windows: [WindowSlot; MAX_WINDOWS],
    pub next_z: u32,
    pub launcher_open: bool,
    pub launcher_category: LauncherCategory,
    pub search_buf: [u8; 32],
    pub search_len: usize,
    // Desktop icon selection & double-click tracking
    pub selected_desktop_icon: Option<usize>,
    pub last_icon_click_idx: Option<usize>,
    pub last_icon_click_tick: u64,
    // Window dragging state
    pub dragging_slot: Option<usize>,
    pub drag_offset_x: usize,
    pub drag_offset_y: usize,
    // Right-click desktop context menu
    pub ctx_menu_open: bool,
    pub ctx_menu_x: usize,
    pub ctx_menu_y: usize,
    // System tray states
    pub volume_muted: bool,
    // File Manager selection
    pub selected_file_idx: usize,
    // Image Viewer selection (0 = Anubis, 1 = Pharaoh)
    pub viewer_image_idx: usize,
    // Network Manager diagnostic message
    pub net_ping_ok: Option<bool>,
    // Interactive Calculator state
    pub calc_accum: i64,
    pub calc_current: i64,
    pub calc_op: u8, // 0=none, b'+', b'-', b'*', b'/'
    pub calc_new_entry: bool,
}

impl DesktopManager {
    pub const fn new() -> Self {
        Self {
            active_app: ActiveApp::DesktopOnly,
            windows: [WindowSlot::empty(); MAX_WINDOWS],
            next_z: 1,
            launcher_open: false,
            launcher_category: LauncherCategory::All,
            search_buf: [0; 32],
            search_len: 0,
            selected_desktop_icon: None,
            last_icon_click_idx: None,
            last_icon_click_tick: 0,
            dragging_slot: None,
            drag_offset_x: 0,
            drag_offset_y: 0,
            ctx_menu_open: false,
            ctx_menu_x: 0,
            ctx_menu_y: 0,
            volume_muted: false,
            selected_file_idx: 0,
            viewer_image_idx: 0,
            net_ping_ok: None,
            calc_accum: 0,
            calc_current: 0,
            calc_op: 0,
            calc_new_entry: true,
        }
    }

    pub fn focused_window_slot(&self) -> Option<usize> {
        let mut best: Option<(usize, u32)> = None;
        for i in 0..MAX_WINDOWS {
            let w = self.windows[i];
            if w.open && !w.minimized {
                if best.map_or(true, |(_, bz)| w.z > bz) {
                    best = Some((i, w.z));
                }
            }
        }
        best.map(|(idx, _)| idx)
    }

    fn sync_active_app(&mut self) {
        if self.launcher_open {
            self.active_app = ActiveApp::AppLauncher;
        } else if let Some(slot) = self.focused_window_slot() {
            self.active_app = self.windows[slot].app;
        } else {
            self.active_app = ActiveApp::DesktopOnly;
        }
    }

    pub fn open_or_focus_app(&mut self, app: ActiveApp, screen_w: usize, screen_h: usize) {
        if app == ActiveApp::DesktopOnly {
            self.minimize_all();
            return;
        }
        if app == ActiveApp::AppLauncher {
            self.launcher_open = !self.launcher_open;
            self.ctx_menu_open = false;
            self.sync_active_app();
            return;
        }

        self.launcher_open = false;
        self.ctx_menu_open = false;

        // Check if already open
        for i in 0..MAX_WINDOWS {
            if self.windows[i].open && self.windows[i].app == app {
                self.windows[i].minimized = false;
                self.next_z += 1;
                self.windows[i].z = self.next_z;
                self.sync_active_app();
                return;
            }
        }

        // Find free slot (or evict lowest-z slot)
        let mut target_slot = None;
        for i in 0..MAX_WINDOWS {
            if !self.windows[i].open {
                target_slot = Some(i);
                break;
            }
        }
        let slot = target_slot.unwrap_or_else(|| {
            let mut min_i = 0;
            let mut min_z = u32::MAX;
            for i in 0..MAX_WINDOWS {
                if self.windows[i].z < min_z {
                    min_z = self.windows[i].z;
                    min_i = i;
                }
            }
            min_i
        });

        let avail_h = screen_h.saturating_sub(TOP_BAR_H + TASKBAR_H);
        let default_w = ((screen_w * 74) / 100).clamp(560, 1180).min(screen_w.saturating_sub(24));
        let default_h = ((avail_h * 78) / 100).clamp(360, 740).min(avail_h.saturating_sub(20));

        // Cascade offset so multiple open windows are clearly visible
        let open_count = self.windows.iter().filter(|w| w.open).count();
        let cascade = (open_count % 4) * 26;
        let base_x = screen_w.saturating_sub(default_w) / 2;
        let base_y = TOP_BAR_H + avail_h.saturating_sub(default_h) / 2;
        let wx = (base_x + cascade).min(screen_w.saturating_sub(default_w + 8));
        let wy = (base_y + cascade).min((TOP_BAR_H + avail_h).saturating_sub(default_h + 6));

        self.next_z += 1;
        self.windows[slot] = WindowSlot {
            open: true,
            app,
            x: wx,
            y: wy,
            w: default_w,
            h: default_h,
            saved_x: wx,
            saved_y: wy,
            saved_w: default_w,
            saved_h: default_h,
            minimized: false,
            maximized: false,
            z: self.next_z,
        };
        self.sync_active_app();
    }

    pub fn close_slot(&mut self, slot: usize) {
        if slot < MAX_WINDOWS {
            self.windows[slot].open = false;
            self.windows[slot].minimized = false;
            self.windows[slot].maximized = false;
            if self.dragging_slot == Some(slot) {
                self.dragging_slot = None;
            }
            self.sync_active_app();
        }
    }

    pub fn minimize_slot(&mut self, slot: usize) {
        if slot < MAX_WINDOWS && self.windows[slot].open {
            self.windows[slot].minimized = true;
            if self.dragging_slot == Some(slot) {
                self.dragging_slot = None;
            }
            self.sync_active_app();
        }
    }

    pub fn toggle_maximize_slot(&mut self, slot: usize, screen_w: usize, screen_h: usize) {
        if slot < MAX_WINDOWS && self.windows[slot].open {
            let w = &mut self.windows[slot];
            if w.maximized {
                w.x = w.saved_x;
                w.y = w.saved_y;
                w.w = w.saved_w;
                w.h = w.saved_h;
                w.maximized = false;
            } else {
                w.saved_x = w.x;
                w.saved_y = w.y;
                w.saved_w = w.w;
                w.saved_h = w.h;
                w.x = 4;
                w.y = TOP_BAR_H + 2;
                w.w = screen_w.saturating_sub(8);
                w.h = screen_h.saturating_sub(TOP_BAR_H + TASKBAR_H + 4);
                w.maximized = true;
            }
        }
    }

    pub fn minimize_all(&mut self) {
        for i in 0..MAX_WINDOWS {
            if self.windows[i].open {
                self.windows[i].minimized = true;
            }
        }
        self.launcher_open = false;
        self.ctx_menu_open = false;
        self.dragging_slot = None;
        self.sync_active_app();
    }

    pub fn close_focused_or_overlay(&mut self) {
        if self.ctx_menu_open {
            self.ctx_menu_open = false;
            return;
        }
        if self.launcher_open {
            self.launcher_open = false;
            self.search_len = 0;
            self.sync_active_app();
            return;
        }
        if let Some(slot) = self.focused_window_slot() {
            self.close_slot(slot);
        }
    }

    pub fn handle_launcher_key(&mut self, ch: char) {
        if ch == '\x08' {
            if self.search_len > 0 {
                self.search_len -= 1;
            }
        } else if (ch as u8) >= 0x20 && (ch as u8) <= 0x7E {
            if self.search_len < 28 {
                self.search_buf[self.search_len] = ch as u8;
                self.search_len += 1;
            }
        }
    }

    pub fn calc_press(&mut self, label: &str) {
        match label {
            "C" => {
                self.calc_accum = 0;
                self.calc_current = 0;
                self.calc_op = 0;
                self.calc_new_entry = true;
            }
            "+" | "-" | "*" | "/" => {
                if self.calc_op != 0 && !self.calc_new_entry {
                    self.calc_eval();
                } else {
                    self.calc_accum = self.calc_current;
                }
                self.calc_op = label.as_bytes()[0];
                self.calc_new_entry = true;
            }
            "=" => {
                self.calc_eval();
                self.calc_op = 0;
                self.calc_new_entry = true;
            }
            d if d.len() == 1 && d.as_bytes()[0].is_ascii_digit() => {
                let digit = (d.as_bytes()[0] - b'0') as i64;
                if self.calc_new_entry {
                    self.calc_current = digit;
                    self.calc_new_entry = false;
                } else {
                    self.calc_current = self.calc_current.saturating_mul(10).saturating_add(digit);
                }
            }
            _ => {}
        }
    }

    fn calc_eval(&mut self) {
        match self.calc_op {
            b'+' => self.calc_current = self.calc_accum.saturating_add(self.calc_current),
            b'-' => self.calc_current = self.calc_accum.saturating_sub(self.calc_current),
            b'*' => self.calc_current = self.calc_accum.saturating_mul(self.calc_current),
            b'/' => {
                if self.calc_current != 0 {
                    self.calc_current = self.calc_accum / self.calc_current;
                } else {
                    self.calc_current = 0;
                }
            }
            _ => {}
        }
        self.calc_accum = self.calc_current;
    }
}

pub static DESKTOP: spin::Mutex<DesktopManager> = spin::Mutex::new(DesktopManager::new());

/// Safe helper to launch an app from anywhere (e.g., Task 1 F-keys or Shell)
pub fn launch_app(app: ActiveApp) {
    let (sw, sh) = {
        let g = GRAPHICS.lock();
        (g.width, g.height)
    };
    if sw == 0 || sh == 0 {
        return;
    }
    DESKTOP.lock().open_or_focus_app(app, sw, sh);
}

/// Read CMOS RTC — returns (hour, minute, second, day, month, year2digit)
fn read_rtc_time() -> (u8, u8, u8, u8, u8, u8) {
    use x86_64::instructions::port::Port;
    let mut addr_port = Port::<u8>::new(0x70);
    let mut data_port = Port::<u8>::new(0x71);

    let bcd_to_bin = |bcd: u8| -> u8 { ((bcd >> 4) * 10).saturating_add(bcd & 0x0F) };

    let (second, minute, hour, day, month, year) = unsafe {
        addr_port.write(0x00); let s = data_port.read();
        addr_port.write(0x02); let m = data_port.read();
        addr_port.write(0x04); let h = data_port.read();
        addr_port.write(0x07); let d = data_port.read();
        addr_port.write(0x08); let mo = data_port.read();
        addr_port.write(0x09); let y = data_port.read();
        addr_port.write(0x0B); let regb = data_port.read();
        if regb & 0x04 != 0 {
            (s, m, h & 0x3F, d, mo, y)
        } else {
            (
                bcd_to_bin(s),
                bcd_to_bin(m),
                bcd_to_bin(h & 0x7F),
                bcd_to_bin(d),
                bcd_to_bin(mo),
                bcd_to_bin(y),
            )
        }
    };
    (hour, minute, second, day, month, year)
}

fn format_u64_into(mut n: u64, buf: &mut [u8; 24]) -> &str {
    if n == 0 {
        buf[0] = b'0';
        return core::str::from_utf8(&buf[..1]).unwrap_or("0");
    }
    let mut tmp = [0u8; 24];
    let mut c = 0;
    while n > 0 && c < 24 {
        tmp[c] = b'0' + (n % 10) as u8;
        n /= 10;
        c += 1;
    }
    for i in 0..c {
        buf[i] = tmp[c - 1 - i];
    }
    core::str::from_utf8(&buf[..c]).unwrap_or("0")
}

fn format_i64_into(val: i64, buf: &mut [u8; 24]) -> &str {
    if val < 0 {
        buf[0] = b'-';
        let mut sub = [0u8; 24];
        let s = format_u64_into(val.unsigned_abs(), &mut sub);
        let len = s.len().min(22);
        for i in 0..len {
            buf[1 + i] = s.as_bytes()[i];
        }
        core::str::from_utf8(&buf[..1 + len]).unwrap_or("0")
    } else {
        format_u64_into(val as u64, buf)
    }
}

fn contains_ignore_case(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    let hb = haystack.as_bytes();
    let nb = needle.as_bytes();
    if nb.len() > hb.len() {
        return false;
    }
    for start in 0..=(hb.len() - nb.len()) {
        let mut matched = true;
        for j in 0..nb.len() {
            if hb[start + j].to_ascii_lowercase() != nb[j].to_ascii_lowercase() {
                matched = false;
                break;
            }
        }
        if matched {
            return true;
        }
    }
    false
}

struct LauncherEntry {
    name: &'static str,
    icon: AppIconKind,
    app: ActiveApp,
    cat: LauncherCategory,
    game: Option<GameType>,
}

const LAUNCHER_ENTRIES: &[LauncherEntry] = &[
    LauncherEntry { name: "Files",      icon: AppIconKind::Files,          app: ActiveApp::FileManager,    cat: LauncherCategory::Productivity, game: None },
    LauncherEntry { name: "Terminal",   icon: AppIconKind::Terminal,       app: ActiveApp::Terminal,       cat: LauncherCategory::System,       game: None },
    LauncherEntry { name: "Browser",    icon: AppIconKind::Browser,        app: ActiveApp::Browser,        cat: LauncherCategory::Productivity, game: None },
    LauncherEntry { name: "Editor",     icon: AppIconKind::TextEditor,     app: ActiveApp::TextEditor,     cat: LauncherCategory::Productivity, game: None },
    LauncherEntry { name: "Calculator", icon: AppIconKind::Calculator,     app: ActiveApp::Calculator,     cat: LauncherCategory::Productivity, game: None },
    LauncherEntry { name: "Gallery",    icon: AppIconKind::ImageViewer,    app: ActiveApp::ImageViewer,    cat: LauncherCategory::Productivity, game: None },
    LauncherEntry { name: "Settings",   icon: AppIconKind::Settings,       app: ActiveApp::Settings,       cat: LauncherCategory::System,       game: None },
    LauncherEntry { name: "Monitor",    icon: AppIconKind::SystemMonitor,  app: ActiveApp::SystemMonitor,  cat: LauncherCategory::System,       game: None },
    LauncherEntry { name: "Network",    icon: AppIconKind::NetworkManager, app: ActiveApp::NetworkManager, cat: LauncherCategory::System,       game: None },
    LauncherEntry { name: "S-Pkg Hub",  icon: AppIconKind::SoftwareStore,  app: ActiveApp::SoftwareStore,  cat: LauncherCategory::System,       game: None },
    LauncherEntry { name: "About OS",   icon: AppIconKind::About,          app: ActiveApp::About,          cat: LauncherCategory::System,       game: None },
    LauncherEntry { name: "Chess",      icon: AppIconKind::Games,          app: ActiveApp::Games,          cat: LauncherCategory::Games,        game: Some(GameType::Chess) },
    LauncherEntry { name: "Snake",      icon: AppIconKind::Games,          app: ActiveApp::Games,          cat: LauncherCategory::Games,        game: Some(GameType::Snake) },
    LauncherEntry { name: "Sudoku",     icon: AppIconKind::Games,          app: ActiveApp::Games,          cat: LauncherCategory::Games,        game: Some(GameType::Sudoku) },
    LauncherEntry { name: "2048",       icon: AppIconKind::Games,          app: ActiveApp::Games,          cat: LauncherCategory::Games,        game: Some(GameType::Game2048) },
    LauncherEntry { name: "Tetris",     icon: AppIconKind::Games,          app: ActiveApp::Games,          cat: LauncherCategory::Games,        game: Some(GameType::Tetris) },
];

const DESKTOP_ICONS: &[(&str, AppIconKind, ActiveApp)] = &[
    ("Files",    AppIconKind::Files,         ActiveApp::FileManager),
    ("Terminal", AppIconKind::Terminal,      ActiveApp::Terminal),
    ("Browser",  AppIconKind::Browser,       ActiveApp::Browser),
    ("Settings", AppIconKind::Settings,      ActiveApp::Settings),
    ("Monitor",  AppIconKind::SystemMonitor, ActiveApp::SystemMonitor),
    ("Games",    AppIconKind::Games,         ActiveApp::Games),
];

const TASKBAR_PINNED: &[(AppIconKind, ActiveApp)] = &[
    (AppIconKind::Files,          ActiveApp::FileManager),
    (AppIconKind::Terminal,       ActiveApp::Terminal),
    (AppIconKind::Browser,        ActiveApp::Browser),
    (AppIconKind::TextEditor,     ActiveApp::TextEditor),
    (AppIconKind::Calculator,     ActiveApp::Calculator),
    (AppIconKind::NetworkManager, ActiveApp::NetworkManager),
    (AppIconKind::Games,          ActiveApp::Games),
    (AppIconKind::SystemMonitor,  ActiveApp::SystemMonitor),
    (AppIconKind::Settings,       ActiveApp::Settings),
    (AppIconKind::SoftwareStore,  ActiveApp::SoftwareStore),
    (AppIconKind::ImageViewer,    ActiveApp::ImageViewer),
];

const CTX_ITEMS: &[&str] = &[
    "Open App Launcher",
    "Open File Manager",
    "Open Terminal",
    "Network Settings",
    "System Settings",
    "Refresh Desktop",
];

fn app_title(app: ActiveApp) -> &'static str {
    match app {
        ActiveApp::Browser => "Web Browser - S-Web MicroText",
        ActiveApp::Terminal => "Terminal - srikesh@srikesh-os:~",
        ActiveApp::FileManager => "File Manager - Persistent VFS",
        ActiveApp::TextEditor => "Text Editor - Royal S-Edit",
        ActiveApp::Calculator => "Calculator - Royal Math Engine",
        ActiveApp::ImageViewer => "Image Viewer - Royal Gallery",
        ActiveApp::NetworkManager => "Network Manager - Hardware & Link",
        ActiveApp::Games => "Games Suite - 9 Offline Games",
        ActiveApp::SystemMonitor => "System Monitor - Scheduler & Telemetry",
        ActiveApp::Settings => "System Settings - Hardware & Power",
        ActiveApp::SoftwareStore => "Software Store - S-Pkg Native Hub",
        ActiveApp::About => "About SRIKESH OS",
        _ => "SRIKESH OS",
    }
}

fn app_icon_kind(app: ActiveApp) -> AppIconKind {
    match app {
        ActiveApp::Browser => AppIconKind::Browser,
        ActiveApp::Terminal => AppIconKind::Terminal,
        ActiveApp::FileManager => AppIconKind::Files,
        ActiveApp::TextEditor => AppIconKind::TextEditor,
        ActiveApp::Calculator => AppIconKind::Calculator,
        ActiveApp::ImageViewer => AppIconKind::ImageViewer,
        ActiveApp::NetworkManager => AppIconKind::NetworkManager,
        ActiveApp::Games => AppIconKind::Games,
        ActiveApp::SystemMonitor => AppIconKind::SystemMonitor,
        ActiveApp::Settings => AppIconKind::Settings,
        ActiveApp::SoftwareStore => AppIconKind::SoftwareStore,
        ActiveApp::About => AppIconKind::About,
        _ => AppIconKind::StartLauncher,
    }
}

/// Renders the complete SRIKESH OS Egyptian Royal Windows-Inspired Desktop Frame
pub fn render_desktop_frame() {
    let mut gfx = GRAPHICS.lock();
    if gfx.width == 0 || gfx.height == 0 {
        return;
    }

    let screen_w = gfx.width;
    let screen_h = gfx.height;
    let mouse_now = get_mouse_state();
    let eth_up = e1000::is_link_up();

    // Snapshot desktop state quickly to avoid holding DESKTOP lock during heavy rendering
    let (
        windows_snap,
        focused_slot,
        launcher_open,
        launcher_cat,
        search_buf,
        search_len,
        selected_desktop_icon,
        ctx_open,
        ctx_x,
        ctx_y,
        volume_muted,
        selected_file_idx,
        viewer_image_idx,
        net_ping_ok,
        calc_current,
        calc_op,
    ) = {
        let d = DESKTOP.lock();
        (
            d.windows,
            d.focused_window_slot(),
            d.launcher_open,
            d.launcher_category,
            d.search_buf,
            d.search_len,
            d.selected_desktop_icon,
            d.ctx_menu_open,
            d.ctx_menu_x,
            d.ctx_menu_y,
            d.volume_muted,
            d.selected_file_idx,
            d.viewer_image_idx,
            d.net_ping_ok,
            d.calc_current,
            d.calc_op,
        )
    };

    // 1. Supplied Reference Image 1: Anubis / Jackal on Golden Throne Wallpaper
    gfx.draw_image_rgb(
        0,
        0,
        screen_w,
        screen_h,
        DESKTOP_WALLPAPER_RAW,
        DESKTOP_WALLPAPER_WIDTH,
        DESKTOP_WALLPAPER_HEIGHT,
    );
    gfx.fill_rect(0, 0, screen_w, screen_h, Color::with_alpha(10, 8, 6, 36));

    // 2. Top Royal Status Bar
    gfx.fill_rect(0, 0, screen_w, TOP_BAR_H, Color::with_alpha(12, 10, 8, 235));
    gfx.fill_rect(0, TOP_BAR_H - 1, screen_w, 1, Color::ROYAL_GOLD);

    gfx.draw_text(12, 10, "< S > SRIKESH OS", Color::BRIGHT_GOLD, 1);
    if screen_w >= 900 {
        gfx.draw_text(156, 10, "|  Awaken the Power of Technology.", Color::ROYAL_SAND, 1);
    }

    let boot_cnt = CONFIG.lock().boot_count;
    let right_x = screen_w.saturating_sub(420);
    if eth_up {
        gfx.draw_text(right_x, 10, "[ETH: UP 192.168.1.120]", Color::SUCCESS_GREEN, 1);
    } else {
        gfx.draw_text(right_x, 10, "[NET: OFFLINE]", Color::ROYAL_AMBER, 1);
    }
    gfx.draw_text(right_x + 196, 10, "RAM: 128 MB", Color::WHITE, 1);
    let mut boot_str_buf = [0u8; 24];
    let b_num = format_u64_into(boot_cnt.max(1) as u64, &mut boot_str_buf);
    gfx.draw_text(right_x + 296, 10, "BOOT #", Color::ROYAL_GOLD, 1);
    gfx.draw_text(right_x + 344, 10, b_num, Color::BRIGHT_GOLD, 1);

    // Top-right clickable Power badge
    let pwr_x = screen_w.saturating_sub(46);
    gfx.fill_rounded_rect(pwr_x, 4, 38, 20, 4, Color::new(56, 20, 18));
    gfx.draw_rect_outline(pwr_x, 4, 38, 20, Color::DANGER_RED, 1);
    gfx.draw_text(pwr_x + 7, 10, "PWR", Color::BRIGHT_GOLD, 1);

    // 3. Desktop Graphical Icons (Left vertical column — Windows-style)
    let tile_w: usize = 78;
    let tile_h: usize = 68;
    let start_x: usize = 16;
    let start_y: usize = TOP_BAR_H + 14;
    let gap_y: usize = 10;

    for (idx, &(label, icon_kind, _target_app)) in DESKTOP_ICONS.iter().enumerate() {
        let col = idx / 6;
        let row = idx % 6;
        let ix = start_x + col * (tile_w + 12);
        let iy = start_y + row * (tile_h + gap_y);
        if iy + tile_h > screen_h.saturating_sub(TASKBAR_H + 8) {
            continue;
        }

        let is_selected = selected_desktop_icon == Some(idx);
        let is_hovered = mouse_now.x >= ix && mouse_now.x <= ix + tile_w
            && mouse_now.y >= iy && mouse_now.y <= iy + tile_h;

        if is_selected {
            gfx.fill_rounded_rect(ix, iy, tile_w, tile_h, 8, Color::with_alpha(90, 72, 24, 185));
            gfx.draw_rect_outline(ix, iy, tile_w, tile_h, Color::BRIGHT_GOLD, 1);
        } else if is_hovered {
            gfx.fill_rounded_rect(ix, iy, tile_w, tile_h, 8, Color::with_alpha(48, 38, 20, 160));
            gfx.draw_rect_outline(ix, iy, tile_w, tile_h, Color::ROYAL_GOLD, 1);
        } else {
            gfx.fill_rounded_rect(ix, iy, tile_w, tile_h, 8, Color::with_alpha(14, 11, 8, 120));
        }

        // Draw crisp 36x36 graphical icon
        icons::draw_app_icon(&mut gfx, icon_kind, ix + (tile_w - 36) / 2, iy + 6, 36);

        // Centered label below icon
        let lw = label.len() * 8;
        let lx = ix + tile_w.saturating_sub(lw) / 2;
        gfx.draw_text(lx, iy + 48, label, Color::WHITE, 1);
    }

    // 4. Render Open Windows sorted by Z-order (bottom to top)
    let mut order: [usize; MAX_WINDOWS] = [0, 1, 2, 3, 4, 5];
    for i in 0..MAX_WINDOWS {
        for j in (i + 1)..MAX_WINDOWS {
            if windows_snap[order[j]].z < windows_snap[order[i]].z {
                order.swap(i, j);
            }
        }
    }

    for &slot_idx in &order {
        let win = windows_snap[slot_idx];
        if !win.open || win.minimized {
            continue;
        }
        let is_focused = focused_slot == Some(slot_idx);
        render_window(
            &mut gfx,
            &win,
            is_focused,
            mouse_now.x,
            mouse_now.y,
            selected_file_idx,
            viewer_image_idx,
            net_ping_ok,
            calc_current,
            calc_op,
            eth_up,
        );
    }

    // 5. Windows-11 / Slax-Inspired Application Launcher Overlay
    if launcher_open {
        let search_str = core::str::from_utf8(&search_buf[..search_len]).unwrap_or("");
        render_app_launcher(&mut gfx, screen_w, screen_h, mouse_now.x, mouse_now.y, launcher_cat, search_str);
    }

    // 6. Right-Click Desktop Context Menu
    if ctx_open {
        let menu_w: usize = 186;
        let item_h: usize = 24;
        let menu_h: usize = CTX_ITEMS.len() * item_h + 10;
        let mx = ctx_x.min(screen_w.saturating_sub(menu_w + 4));
        let my = ctx_y.min(screen_h.saturating_sub(TASKBAR_H + menu_h + 4));

        gfx.fill_rounded_rect(mx + 3, my + 3, menu_w, menu_h, 6, Color::with_alpha(0, 0, 0, 180));
        gfx.fill_rounded_rect(mx, my, menu_w, menu_h, 6, Color::new(20, 16, 12));
        gfx.draw_rect_outline(mx, my, menu_w, menu_h, Color::ROYAL_GOLD, 1);

        for (i, &label) in CTX_ITEMS.iter().enumerate() {
            let iy = my + 5 + i * item_h;
            let hovered = mouse_now.x >= mx + 4 && mouse_now.x <= mx + menu_w - 4
                && mouse_now.y >= iy && mouse_now.y < iy + item_h;
            if hovered {
                gfx.fill_rounded_rect(mx + 4, iy, menu_w - 8, item_h - 2, 4, Color::new(68, 54, 22));
            }
            gfx.draw_text(mx + 12, iy + 7, label, if hovered { Color::BRIGHT_GOLD } else { Color::WHITE }, 1);
        }
    }

    // 7. Windows-11-Inspired Bottom Taskbar
    let tb_y = screen_h.saturating_sub(TASKBAR_H);
    gfx.fill_rect(0, tb_y, screen_w, TASKBAR_H, Color::with_alpha(14, 11, 8, 242));
    gfx.fill_rect(0, tb_y, screen_w, 1, Color::ROYAL_GOLD);

    // Compute centered taskbar icon strip (Start + 11 pinned icons)
    let icon_slot_w: usize = 42;
    let total_icons = 1 + TASKBAR_PINNED.len();
    let strip_w = total_icons * icon_slot_w;
    let tray_reserve: usize = 200;
    let strip_x = if screen_w > strip_w + tray_reserve + 24 {
        (screen_w.saturating_sub(tray_reserve) - strip_w) / 2
    } else {
        8
    };

    // Start / App Launcher button
    let start_hovered = mouse_now.x >= strip_x && mouse_now.x < strip_x + icon_slot_w
        && mouse_now.y >= tb_y && mouse_now.y < screen_h;
    if launcher_open || start_hovered {
        gfx.fill_rounded_rect(strip_x + 2, tb_y + 5, icon_slot_w - 4, TASKBAR_H - 10, 6,
            if launcher_open { Color::new(82, 64, 20) } else { Color::new(42, 34, 18) });
        gfx.draw_rect_outline(strip_x + 2, tb_y + 5, icon_slot_w - 4, TASKBAR_H - 10, Color::ROYAL_GOLD, 1);
    }
    icons::draw_app_icon(&mut gfx, AppIconKind::StartLauncher, strip_x + 7, tb_y + 10, 28);

    // Pinned graphical app icons with running indicators
    for (i, &(icon_kind, target_app)) in TASKBAR_PINNED.iter().enumerate() {
        let bx = strip_x + (i + 1) * icon_slot_w;
        let hovered = mouse_now.x >= bx && mouse_now.x < bx + icon_slot_w
            && mouse_now.y >= tb_y && mouse_now.y < screen_h;

        let mut is_open = false;
        let mut is_focused_app = false;
        for s in 0..MAX_WINDOWS {
            if windows_snap[s].open && windows_snap[s].app == target_app {
                is_open = true;
                if focused_slot == Some(s) {
                    is_focused_app = true;
                }
            }
        }

        if is_focused_app || hovered {
            let bg = if is_focused_app { Color::new(66, 52, 20) } else { Color::new(38, 30, 18) };
            gfx.fill_rounded_rect(bx + 2, tb_y + 5, icon_slot_w - 4, TASKBAR_H - 10, 6, bg);
            if is_focused_app {
                gfx.draw_rect_outline(bx + 2, tb_y + 5, icon_slot_w - 4, TASKBAR_H - 10, Color::ROYAL_GOLD, 1);
            }
        }

        icons::draw_app_icon(&mut gfx, icon_kind, bx + 7, tb_y + 9, 28);

        // Windows-11-style running indicator pill at bottom
        if is_focused_app {
            gfx.fill_rect(bx + 11, tb_y + TASKBAR_H - 5, 20, 3, Color::BRIGHT_GOLD);
        } else if is_open {
            gfx.fill_rect(bx + 17, tb_y + TASKBAR_H - 5, 8, 3, Color::ROYAL_SAND);
        }
    }

    // System Tray (Right side of Taskbar — Windows-11 inspired)
    let tray_x = screen_w.saturating_sub(196);
    gfx.fill_rect(tray_x - 8, tb_y + 10, 1, TASKBAR_H - 20, Color::DARK_GOLD);

    // Network icon (clickable -> opens Network Manager)
    icons::draw_tray_network(&mut gfx, tray_x, tb_y + 16, eth_up);
    // Volume icon (clickable -> toggles mute)
    icons::draw_tray_volume(&mut gfx, tray_x + 26, tb_y + 16, volume_muted);
    // Battery icon
    icons::draw_tray_battery(&mut gfx, tray_x + 52, tb_y + 17, 99);

    // Stacked Live RTC Time & Date
    let (rtc_h, rtc_m, _rtc_s, rtc_d, rtc_mo, rtc_y) = read_rtc_time();
    let mut time_buf = [0u8; 5];
    time_buf[0] = b'0' + (rtc_h / 10).min(9);
    time_buf[1] = b'0' + (rtc_h % 10).min(9);
    time_buf[2] = b':';
    time_buf[3] = b'0' + (rtc_m / 10).min(9);
    time_buf[4] = b'0' + (rtc_m % 10).min(9);

    let mut date_buf = [0u8; 10];
    date_buf[0] = b'0' + (rtc_d / 10).min(9);
    date_buf[1] = b'0' + (rtc_d % 10).min(9);
    date_buf[2] = b'-';
    date_buf[3] = b'0' + (rtc_mo / 10).min(9);
    date_buf[4] = b'0' + (rtc_mo % 10).min(9);
    date_buf[5] = b'-';
    date_buf[6] = b'2';
    date_buf[7] = b'0';
    date_buf[8] = b'0' + (rtc_y / 10).min(9);
    date_buf[9] = b'0' + (rtc_y % 10).min(9);

    if let Ok(t_str) = core::str::from_utf8(&time_buf) {
        gfx.draw_text(tray_x + 114, tb_y + 10, t_str, Color::BRIGHT_GOLD, 1);
    }
    if let Ok(d_str) = core::str::from_utf8(&date_buf) {
        gfx.draw_text(tray_x + 96, tb_y + 26, d_str, Color::ROYAL_SAND, 1);
    }

    // 8. Hardware Mouse Cursor (always topmost)
    gfx.draw_cursor(mouse_now.x, mouse_now.y);
    gfx.swap_buffers();
}

#[allow(clippy::too_many_arguments)]
fn render_window(
    gfx: &mut GraphicsEngine,
    win: &WindowSlot,
    is_focused: bool,
    mouse_x: usize,
    mouse_y: usize,
    selected_file_idx: usize,
    viewer_image_idx: usize,
    net_ping_ok: Option<bool>,
    calc_current: i64,
    calc_op: u8,
    eth_up: bool,
) {
    let win_x = win.x;
    let win_y = win.y;
    let win_w = win.w;
    let win_h = win.h;
    let title_h: usize = 32;

    // Drop shadow & frame
    gfx.fill_rounded_rect(win_x + 4, win_y + 4, win_w, win_h, 8, Color::with_alpha(0, 0, 0, 140));
    gfx.fill_rounded_rect(win_x, win_y, win_w, win_h, 8, Color::new(20, 16, 12));
    let border_col = if is_focused { Color::BRIGHT_GOLD } else { Color::DARK_GOLD };
    gfx.draw_rect_outline(win_x, win_y, win_w, win_h, border_col, 1);

    // Title Bar
    let tb_col = if is_focused { Color::new(42, 33, 20) } else { Color::new(28, 23, 17) };
    gfx.fill_rounded_rect(win_x + 1, win_y + 1, win_w - 2, title_h, 6, tb_col);
    gfx.fill_rect(win_x + 1, win_y + title_h, win_w - 2, 1, border_col);

    // Mini graphical icon + title on left
    icons::draw_app_icon(gfx, app_icon_kind(win.app), win_x + 8, win_y + 6, 20);
    gfx.draw_text(win_x + 34, win_y + 12, app_title(win.app), if is_focused { Color::BRIGHT_GOLD } else { Color::ROYAL_SAND }, 1);

    // Windows-style Minimize [-], Maximize/Restore [O], Close [X] buttons on top-right
    let btn_w: usize = 28;
    let btn_h: usize = 22;
    let btn_y = win_y + 5;
    let close_x = win_x + win_w - btn_w - 6;
    let max_x = close_x - btn_w - 4;
    let min_x = max_x - btn_w - 4;

    let min_hov = mouse_x >= min_x && mouse_x < min_x + btn_w && mouse_y >= btn_y && mouse_y < btn_y + btn_h;
    let max_hov = mouse_x >= max_x && mouse_x < max_x + btn_w && mouse_y >= btn_y && mouse_y < btn_y + btn_h;
    let cls_hov = mouse_x >= close_x && mouse_x < close_x + btn_w && mouse_y >= btn_y && mouse_y < btn_y + btn_h;

    gfx.fill_rounded_rect(min_x, btn_y, btn_w, btn_h, 4, if min_hov { Color::new(72, 58, 24) } else { Color::new(32, 26, 18) });
    gfx.draw_text(min_x + 10, btn_y + 7, "-", Color::ROYAL_GOLD, 1);

    gfx.fill_rounded_rect(max_x, btn_y, btn_w, btn_h, 4, if max_hov { Color::new(40, 90, 56) } else { Color::new(32, 26, 18) });
    gfx.draw_rect_outline(max_x + 9, btn_y + 6, 10, 10, Color::SUCCESS_GREEN, 1);

    gfx.fill_rounded_rect(close_x, btn_y, btn_w, btn_h, 4, if cls_hov { Color::DANGER_RED } else { Color::new(96, 26, 22) });
    gfx.draw_text(close_x + 10, btn_y + 7, "X", Color::WHITE, 1);

    // Window Body Content
    let cx = win_x + 16;
    let mut cy = win_y + title_h + 12;

    match win.app {
        ActiveApp::Browser => {
            BROWSER.lock().render(gfx, win_x, win_y, win_w, win_h);
        }
        ActiveApp::Terminal => {
            let shell = SHELL.lock();
            let count = shell.history_count;
            let max_lines = ((win_h.saturating_sub(title_h + 48)) / 16).clamp(6, 20);
            let start = count.saturating_sub(max_lines);
            for i in start..count {
                let len = shell.history_lens[i];
                if let Ok(line) = core::str::from_utf8(&shell.history_lines[i][..len]) {
                    gfx.draw_text(cx, cy, line, Color::SUCCESS_GREEN, 1);
                }
                cy += 16;
            }
            gfx.draw_text(cx, cy, "srikesh@srikesh-os:~$ ", Color::BRIGHT_GOLD, 1);
            let prompt_w = 22 * 8;
            if let Ok(input_str) = core::str::from_utf8(&shell.input_buf[..shell.input_len]) {
                gfx.draw_text(cx + prompt_w, cy, input_str, Color::WHITE, 1);
                gfx.fill_rect(cx + prompt_w + shell.input_len * 8, cy, 8, 10, Color::BRIGHT_GOLD);
            }
        }
        ActiveApp::FileManager => {
            gfx.draw_text(cx, cy, "Path: /home/srikesh/  (Click a file to preview contents)", Color::ROYAL_SAND, 1);
            cy += 22;
            let list_w = (win_w - 40) / 2;
            let vfs = VFS.lock();
            if let Ok((items, num_items)) = vfs.list_dir("/home/srikesh") {
                for i in 0..num_items.min(8) {
                    let (name, is_dir, _sz) = items[i];
                    let ry = cy + i * 28;
                    let sel = i == selected_file_idx;
                    gfx.fill_rounded_rect(cx, ry, list_w - 8, 24, 4,
                        if sel { Color::new(72, 56, 22) } else { Color::new(28, 22, 16) });
                    gfx.draw_rect_outline(cx, ry, list_w - 8, 24,
                        if sel { Color::BRIGHT_GOLD } else { Color::DARK_GOLD }, 1);
                    gfx.draw_text(cx + 8, ry + 8, if is_dir { "[DIR]" } else { "[TXT]" }, Color::ROYAL_GOLD, 1);
                    gfx.draw_text(cx + 56, ry + 8, name, Color::WHITE, 1);
                }

                // Right preview pane
                let px = cx + list_w + 4;
                let pw = win_w.saturating_sub(list_w + 36);
                let ph = win_h.saturating_sub(title_h + 52);
                gfx.fill_rounded_rect(px, cy, pw, ph, 6, Color::new(14, 11, 8));
                gfx.draw_rect_outline(px, cy, pw, ph, Color::DARK_GOLD, 1);
                gfx.draw_text(px + 10, cy + 8, "File Preview:", Color::BRIGHT_GOLD, 1);
                if selected_file_idx < num_items {
                    let (fname, _, _) = items[selected_file_idx];
                    let mut path_buf = [0u8; 64];
                    let prefix = b"/home/srikesh/";
                    let mut plen = 0;
                    for &b in prefix {
                        path_buf[plen] = b;
                        plen += 1;
                    }
                    for &b in fname.as_bytes() {
                        if plen < 63 {
                            path_buf[plen] = b;
                            plen += 1;
                        }
                    }
                    if let Ok(full_path) = core::str::from_utf8(&path_buf[..plen]) {
                        if let Ok(data) = vfs.read_file(full_path) {
                            if let Ok(txt) = core::str::from_utf8(data) {
                                let mut py = cy + 28;
                                for line in txt.split('\n').take(12) {
                                    let max_c = (pw.saturating_sub(20)) / 8;
                                    let clipped = if line.len() > max_c { &line[..max_c] } else { line };
                                    gfx.draw_text(px + 10, py, clipped, Color::ROYAL_SAND, 1);
                                    py += 16;
                                }
                            }
                        }
                    }
                }
            }
        }
        ActiveApp::TextEditor => {
            gfx.fill_rect(cx, cy, win_w - 32, 22, Color::new(28, 22, 16));
            gfx.draw_text(cx + 8, cy + 7, "File: /home/srikesh/notes.txt  |  UTF-8  |  Royal S-Edit", Color::ROYAL_SAND, 1);
            cy += 30;
            let lines = [
                ("1 |", "// SRIKESH OS - Kernel Architecture Notes"),
                ("2 |", "// Custom x86-64 Rust no_std Micro-OS"),
                ("3 |", "// Target: Dell Vostro 3520 (Intel/AMD x86-64)"),
                ("4 |", ""),
                ("5 |", "fn kernel_entry() {"),
                ("6 |", "    init_preemptive_scheduler(); // 50ms Round-Robin"),
                ("7 |", "    lock_host_internal_storage(); // Zero-Write Guard"),
                ("8 |", "    init_multi_window_compositor();"),
                ("9 |", "    awaken_power_of_technology();"),
                ("10|", "}"),
            ];
            for (n, code) in lines {
                gfx.draw_text(cx + 4, cy, n, Color::TEXT_MUTED, 1);
                gfx.draw_text(cx + 38, cy, code, Color::BRIGHT_GOLD, 1);
                cy += 18;
            }
        }
        ActiveApp::Calculator => {
            let calc_w: usize = 296;
            let calc_x = win_x + win_w.saturating_sub(calc_w) / 2;
            let mut calc_y = cy + 4;
            gfx.fill_rounded_rect(calc_x, calc_y, calc_w, 46, 6, Color::new(12, 22, 18));
            gfx.draw_rect_outline(calc_x, calc_y, calc_w, 46, Color::BRIGHT_GOLD, 2);

            let mut num_buf = [0u8; 24];
            let num_str = format_i64_into(calc_current, &mut num_buf);
            gfx.draw_text(calc_x + 14, calc_y + 14, num_str, Color::BRIGHT_GOLD, 2);
            if calc_op != 0 {
                let op_arr = [calc_op];
                if let Ok(op_s) = core::str::from_utf8(&op_arr) {
                    gfx.draw_text(calc_x + calc_w - 28, calc_y + 16, op_s, Color::SUCCESS_GREEN, 2);
                }
            }
            calc_y += 56;

            let buttons = [
                ["7", "8", "9", "/"],
                ["4", "5", "6", "*"],
                ["1", "2", "3", "-"],
                ["C", "0", "=", "+"],
            ];
            for row in buttons {
                let mut bx = calc_x;
                for label in row {
                    let is_op = matches!(label, "/" | "*" | "-" | "+" | "=" | "C");
                    let hov = mouse_x >= bx && mouse_x < bx + 68 && mouse_y >= calc_y && mouse_y < calc_y + 38;
                    let bg = if hov {
                        Color::new(84, 66, 24)
                    } else if is_op {
                        Color::new(46, 36, 18)
                    } else {
                        Color::new(30, 24, 18)
                    };
                    gfx.fill_rounded_rect(bx, calc_y, 68, 38, 4, bg);
                    gfx.draw_rect_outline(bx, calc_y, 68, 38, Color::ROYAL_GOLD, 1);
                    gfx.draw_text(bx + 28, calc_y + 14, label, Color::WHITE, 1);
                    bx += 76;
                }
                calc_y += 44;
            }
        }
        ActiveApp::ImageViewer => {
            gfx.draw_text(cx, cy, "Royal Gallery — Official Supplied Artworks (Click to switch):", Color::ROYAL_SAND, 1);
            cy += 22;
            // Toggle buttons
            let b1_bg = if viewer_image_idx == 0 { Color::new(82, 64, 22) } else { Color::new(28, 22, 16) };
            let b2_bg = if viewer_image_idx == 1 { Color::new(82, 64, 22) } else { Color::new(28, 22, 16) };
            gfx.fill_rounded_rect(cx, cy, 160, 24, 4, b1_bg);
            gfx.draw_rect_outline(cx, cy, 160, 24, Color::ROYAL_GOLD, 1);
            gfx.draw_text(cx + 12, cy + 8, "1. Anubis Desktop", Color::BRIGHT_GOLD, 1);

            gfx.fill_rounded_rect(cx + 172, cy, 160, 24, 4, b2_bg);
            gfx.draw_rect_outline(cx + 172, cy, 160, 24, Color::ROYAL_GOLD, 1);
            gfx.draw_text(cx + 184, cy + 8, "2. Pharaoh Welcome", Color::BRIGHT_GOLD, 1);
            cy += 32;

            let img_w = (win_w - 40).min(480);
            let img_h = (win_h.saturating_sub(title_h + 90)).min(270);
            let img_x = win_x + (win_w - img_w) / 2;
            gfx.draw_rect_outline(img_x - 2, cy - 2, img_w + 4, img_h + 4, Color::BRIGHT_GOLD, 2);
            if viewer_image_idx == 0 {
                gfx.draw_image_rgb(img_x, cy, img_w, img_h, DESKTOP_WALLPAPER_RAW, DESKTOP_WALLPAPER_WIDTH, DESKTOP_WALLPAPER_HEIGHT);
            } else {
                gfx.draw_image_rgb(img_x, cy, img_w, img_h, STARTUP_EGYPTIAN_RAW, STARTUP_EGYPTIAN_WIDTH, STARTUP_EGYPTIAN_HEIGHT);
            }
        }
        ActiveApp::NetworkManager => {
            let pci = PCI.lock();
            let eth_chip = pci.ethernet_chipset;
            let wifi_det = pci.wifi_detected;
            let wifi_chip = pci.wifi_chipset;
            drop(pci);

            gfx.fill_rect(cx, cy, win_w - 32, 20, Color::new(28, 22, 14));
            gfx.draw_text(cx + 8, cy + 6, "[ ETHERNET -- Hardware Link Status ]", Color::BRIGHT_GOLD, 1);
            cy += 24;

            gfx.draw_text(cx + 4, cy, "Adapter       :", Color::ROYAL_GOLD, 1);
            gfx.draw_text(cx + 140, cy, eth_chip, Color::WHITE, 1);
            cy += 18;

            gfx.draw_text(cx + 4, cy, "Link State    :", Color::ROYAL_GOLD, 1);
            if eth_up {
                gfx.draw_text(cx + 140, cy, "CONNECTED (Intel e1000 Physical Link UP)", Color::SUCCESS_GREEN, 1);
            } else {
                gfx.draw_text(cx + 140, cy, "DISCONNECTED / NO ACTIVE ETHERNET LINK", Color::ROYAL_AMBER, 1);
            }
            cy += 18;

            gfx.draw_text(cx + 4, cy, "IPv4 / Stack  :", Color::ROYAL_GOLD, 1);
            if eth_up {
                gfx.draw_text(cx + 140, cy, "192.168.1.120 / 24  (ARP + IPv4 + ICMP)", Color::WHITE, 1);
            } else {
                gfx.draw_text(cx + 140, cy, "Unassigned (Offline)", Color::TEXT_MUTED, 1);
            }
            cy += 26;

            // Clickable Ping Diagnostic Button
            gfx.fill_rounded_rect(cx + 4, cy, 180, 24, 4, Color::new(42, 34, 18));
            gfx.draw_rect_outline(cx + 4, cy, 180, 24, Color::BRIGHT_GOLD, 1);
            gfx.draw_text(cx + 16, cy + 8, "[ Test ICMP Ping ]", Color::BRIGHT_GOLD, 1);
            if let Some(ok) = net_ping_ok {
                gfx.draw_text(
                    cx + 196,
                    cy + 8,
                    if ok { "ICMP Echo Sent OK" } else { "Link Offline - Not Sent" },
                    if ok { Color::SUCCESS_GREEN } else { Color::ROYAL_AMBER },
                    1,
                );
            }
            cy += 34;

            gfx.fill_rect(cx, cy, win_w - 32, 20, Color::new(28, 22, 14));
            gfx.draw_text(cx + 8, cy + 6, "[ WI-FI -- PCI Hardware Discovery ]", Color::BRIGHT_GOLD, 1);
            cy += 24;

            gfx.draw_text(cx + 4, cy, "PCI Discovery :", Color::ROYAL_GOLD, 1);
            gfx.draw_text(
                cx + 140,
                cy,
                if wifi_det { wifi_chip } else { "No PCI Wireless Adapter on Bus" },
                if wifi_det { Color::SUCCESS_GREEN } else { Color::ROYAL_SAND },
                1,
            );
            cy += 18;
            gfx.draw_text(cx + 4, cy, "Association   :", Color::ROYAL_GOLD, 1);
            gfx.draw_text(cx + 140, cy, "Not Connected (WPA2/802.11 firmware not loaded)", Color::TEXT_MUTED, 1);
        }
        ActiveApp::Games => {
            GAMES.lock().render(gfx, win_x, win_y, win_w, win_h);
        }
        ActiveApp::SystemMonitor => {
            let ticks = crate::interrupts::TIMER_TICKS.load(Ordering::Relaxed);
            let switches = crate::CONTEXT_SWITCH_COUNT.load(Ordering::Relaxed);
            let t1 = crate::TASK1_DISPATCH_COUNT.load(Ordering::Relaxed);
            let t2 = crate::TASK2_DISPATCH_COUNT.load(Ordering::Relaxed);

            let mut b1 = [0u8; 24];
            let mut b2 = [0u8; 24];
            let mut b3 = [0u8; 24];
            let mut b4 = [0u8; 24];

            let rows: &[(&str, &str)] = &[
                ("CPU Mode          :", "x86-64 Long Mode (Custom Rust no_std Kernel)"),
                ("PIT Timer Ticks   :", format_u64_into(ticks, &mut b1)),
                ("Context Switches  :", format_u64_into(switches, &mut b2)),
                ("Task 1 Dispatches :", format_u64_into(t1, &mut b3)),
                ("Task 2 Dispatches :", format_u64_into(t2, &mut b4)),
                ("Scheduler Quantum :", "5 PIT Ticks (50 ms Round-Robin Preemption)"),
                ("Memory Allocator  :", "64 KB Kernel Heap Bump Allocator + Dynamic Mapping"),
                ("Host Disk Guard   :", "ACTIVE (Internal NVMe/SATA Strictly Read-Only)"),
            ];
            for &(k, v) in rows {
                gfx.draw_text(cx, cy, k, Color::ROYAL_GOLD, 1);
                gfx.draw_text(cx + 170, cy, v, Color::WHITE, 1);
                cy += 22;
            }
        }
        ActiveApp::Settings => {
            let pci = PCI.lock();
            let gpu = pci.gpu_name;
            let usb = pci.usb_controller;
            let nvme = pci.nvme_detected;
            let sata = pci.sata_detected;
            let audio = pci.audio_detected;
            let dev_cnt = pci.count;
            drop(pci);

            let mut dev_buf = [0u8; 24];
            let rows: &[(&str, &str)] = &[
                ("Display / GPU     :", gpu),
                ("USB Controller    :", usb),
                ("NVMe Storage      :", if nvme { "Detected (Host Guard: Protected)" } else { "Not Detected" }),
                ("SATA AHCI         :", if sata { "Detected" } else { "Not Detected" }),
                ("Audio Controller  :", if audio { "PCI Audio Detected" } else { "Not Detected" }),
                ("PCI Devices Found :", format_u64_into(dev_cnt as u64, &mut dev_buf)),
                ("Visual Theme      :", "Egyptian Royal Black & Gold (Anubis / Pharaoh)"),
            ];
            for &(k, v) in rows {
                gfx.draw_text(cx, cy, k, Color::ROYAL_GOLD, 1);
                gfx.draw_text(cx + 170, cy, v, Color::WHITE, 1);
                cy += 20;
            }

            cy += 16;
            gfx.draw_text(cx, cy, "ACPI System Power Controls:", Color::BRIGHT_GOLD, 1);
            cy += 22;

            // Clickable Reboot & Shutdown buttons
            gfx.fill_rounded_rect(cx, cy, 140, 28, 5, Color::new(56, 44, 18));
            gfx.draw_rect_outline(cx, cy, 140, 28, Color::ROYAL_GOLD, 1);
            gfx.draw_text(cx + 22, cy + 10, "Reboot System", Color::BRIGHT_GOLD, 1);

            gfx.fill_rounded_rect(cx + 156, cy, 150, 28, 5, Color::new(82, 22, 20));
            gfx.draw_rect_outline(cx + 156, cy, 150, 28, Color::DANGER_RED, 1);
            gfx.draw_text(cx + 174, cy + 10, "Shutdown System", Color::WHITE, 1);
        }
        ActiveApp::SoftwareStore => {
            gfx.draw_text(cx, cy, "S-PKG NATIVE SOFTWARE HUB (Lightweight Micro-OS Packages):", Color::ROYAL_SAND, 1);
            cy += 24;
            let pkgs = [
                ("s-shell-extras v1.4", "Extended CLI utilities & diagnostics", true),
                ("royal-games-pack v2.0", "9 Lightweight Offline Games Suite", true),
                ("net-tools v1.2", "ICMP Ping & S-Web HTTP Client", true),
                ("s-edit-pro v1.0", "Enhanced text viewer & notes manager", false),
            ];
            for (name, desc, installed) in pkgs {
                let rw = win_w - 32;
                gfx.fill_rounded_rect(cx, cy, rw, 26, 4, Color::new(28, 22, 16));
                gfx.draw_rect_outline(cx, cy, rw, 26, Color::DARK_GOLD, 1);
                gfx.draw_text(cx + 10, cy + 9, name, Color::BRIGHT_GOLD, 1);
                gfx.draw_text(cx + 190, cy + 9, desc, Color::WHITE, 1);
                let (btxt, bcol) = if installed { ("[INSTALLED]", Color::SUCCESS_GREEN) } else { ("[AVAILABLE]", Color::ROYAL_SAND) };
                gfx.draw_text(cx + rw - 96, cy + 9, btxt, bcol, 1);
                cy += 32;
            }
            cy += 8;
            gfx.draw_text(cx, cy, "Note: Native S-Pkg runs without Linux glibc/APT/Snap dependencies.", Color::TEXT_MUTED, 1);
        }
        ActiveApp::About => {
            gfx.draw_text(cx, cy, "< S >  S R I K E S H   O S", Color::BRIGHT_GOLD, 2);
            cy += 28;
            gfx.draw_text(cx, cy, "Awaken the Power of Technology.", Color::ROYAL_SAND, 1);
            cy += 24;
            let about = [
                ("Developer      :", "Srikesh S (CSE AI/ML, USN: 25BTRCL258)"),
                ("Architecture   :", "Independent x86-64 Custom Rust no_std Micro-OS"),
                ("Image Footprint:", "Under 6 MB (Strict <= 500 MB Budget Compliant)"),
                ("Target Laptop  :", "Dell Vostro 3520 & x86-64 Intel/AMD Systems"),
                ("Boot Support   :", "Hybrid UEFI GOP & Legacy BIOS Bootable USB"),
                ("Verification   :", "Oracle VirtualBox (Zero QEMU Policy)"),
            ];
            for (k, v) in about {
                gfx.draw_text(cx, cy, k, Color::ROYAL_GOLD, 1);
                gfx.draw_text(cx + 140, cy, v, Color::WHITE, 1);
                cy += 20;
            }
        }
        _ => {}
    }
}

fn launcher_rect(screen_w: usize, screen_h: usize) -> (usize, usize, usize, usize) {
    let lw = 640usize.min(screen_w.saturating_sub(24));
    let lh = 410usize.min(screen_h.saturating_sub(TOP_BAR_H + TASKBAR_H + 20));
    let lx = (screen_w - lw) / 2;
    let ly = screen_h.saturating_sub(TASKBAR_H + lh + 10);
    (lx, ly, lw, lh)
}

fn render_app_launcher(
    gfx: &mut GraphicsEngine,
    screen_w: usize,
    screen_h: usize,
    mouse_x: usize,
    mouse_y: usize,
    active_cat: LauncherCategory,
    search_str: &str,
) {
    let (lx, ly, lw, lh) = launcher_rect(screen_w, screen_h);

    // Floating glass panel
    gfx.fill_rounded_rect(lx + 4, ly + 4, lw, lh, 10, Color::with_alpha(0, 0, 0, 170));
    gfx.fill_rounded_rect(lx, ly, lw, lh, 10, Color::with_alpha(18, 15, 11, 245));
    gfx.draw_rect_outline(lx, ly, lw, lh, Color::BRIGHT_GOLD, 1);

    // Top Search Bar
    let sb_x = lx + 16;
    let sb_y = ly + 14;
    let sb_w = lw - 32;
    gfx.fill_rounded_rect(sb_x, sb_y, sb_w, 28, 6, Color::new(10, 9, 7));
    gfx.draw_rect_outline(sb_x, sb_y, sb_w, 28, Color::ROYAL_GOLD, 1);
    if search_str.is_empty() {
        gfx.draw_text(sb_x + 12, sb_y + 10, "Type to search apps & games...", Color::TEXT_MUTED, 1);
    } else {
        gfx.draw_text(sb_x + 12, sb_y + 10, "Search: ", Color::ROYAL_GOLD, 1);
        gfx.draw_text(sb_x + 76, sb_y + 10, search_str, Color::WHITE, 1);
    }

    // Left Category Sidebar
    let side_x = lx + 14;
    let side_y = ly + 54;
    let side_w: usize = 124;
    let cats: &[(&str, LauncherCategory)] = &[
        ("All Apps",     LauncherCategory::All),
        ("Productivity", LauncherCategory::Productivity),
        ("System",       LauncherCategory::System),
        ("Games",        LauncherCategory::Games),
    ];
    for (i, &(label, cat)) in cats.iter().enumerate() {
        let cy = side_y + i * 32;
        let sel = cat == active_cat;
        let hov = mouse_x >= side_x && mouse_x < side_x + side_w && mouse_y >= cy && mouse_y < cy + 26;
        if sel || hov {
            gfx.fill_rounded_rect(side_x, cy, side_w, 26, 5, if sel { Color::new(76, 60, 20) } else { Color::new(38, 30, 16) });
            if sel {
                gfx.draw_rect_outline(side_x, cy, side_w, 26, Color::BRIGHT_GOLD, 1);
            }
        }
        gfx.draw_text(side_x + 10, cy + 9, label, if sel { Color::BRIGHT_GOLD } else { Color::WHITE }, 1);
    }

    // Sidebar bottom power controls
    let pwr_y = ly + lh - 68;
    gfx.fill_rounded_rect(side_x, pwr_y, side_w, 24, 4, Color::new(48, 38, 16));
    gfx.draw_rect_outline(side_x, pwr_y, side_w, 24, Color::ROYAL_GOLD, 1);
    gfx.draw_text(side_x + 16, pwr_y + 8, "Reboot", Color::BRIGHT_GOLD, 1);

    gfx.fill_rounded_rect(side_x, pwr_y + 30, side_w, 24, 4, Color::new(74, 20, 18));
    gfx.draw_rect_outline(side_x, pwr_y + 30, side_w, 24, Color::DANGER_RED, 1);
    gfx.draw_text(side_x + 16, pwr_y + 38, "Shutdown", Color::WHITE, 1);

    // Divider line
    gfx.fill_rect(lx + side_w + 22, side_y, 1, lh - 66, Color::DARK_GOLD);

    // Main Graphical App Grid (4 columns)
    let grid_x = lx + side_w + 34;
    let grid_y = side_y;
    let cell_w: usize = 108;
    let cell_h: usize = 74;
    let cols: usize = 4;

    let mut shown = 0usize;
    for entry in LAUNCHER_ENTRIES {
        if active_cat != LauncherCategory::All && entry.cat != active_cat {
            continue;
        }
        if !contains_ignore_case(entry.name, search_str) {
            continue;
        }
        if shown >= 16 {
            break;
        }
        let col = shown % cols;
        let row = shown / cols;
        let cx = grid_x + col * (cell_w + 8);
        let cy = grid_y + row * (cell_h + 8);
        if cy + cell_h > ly + lh - 8 {
            break;
        }

        let hov = mouse_x >= cx && mouse_x < cx + cell_w && mouse_y >= cy && mouse_y < cy + cell_h;
        gfx.fill_rounded_rect(cx, cy, cell_w, cell_h, 6, if hov { Color::new(66, 52, 22) } else { Color::new(26, 20, 14) });
        gfx.draw_rect_outline(cx, cy, cell_w, cell_h, if hov { Color::BRIGHT_GOLD } else { Color::DARK_GOLD }, 1);

        icons::draw_app_icon(gfx, entry.icon, cx + (cell_w - 36) / 2, cy + 8, 36);
        let nw = entry.name.len() * 8;
        gfx.draw_text(cx + cell_w.saturating_sub(nw) / 2, cy + 52, entry.name, Color::WHITE, 1);

        shown += 1;
    }
}

/// Handle mouse drag / move updates (for smooth window dragging)
pub fn handle_mouse_move(mouse_x: usize, mouse_y: usize, left_down: bool) -> bool {
    let (screen_w, screen_h) = {
        let g = GRAPHICS.lock();
        (g.width, g.height)
    };
    if screen_w == 0 || screen_h == 0 {
        return false;
    }
    let mut d = DESKTOP.lock();
    if let Some(slot) = d.dragging_slot {
        if !left_down {
            d.dragging_slot = None;
            return false;
        }
        let max_x = screen_w.saturating_sub(d.windows[slot].w + 2);
        let max_y = screen_h.saturating_sub(TASKBAR_H + d.windows[slot].h + 2);
        let new_x = mouse_x.saturating_sub(d.drag_offset_x).min(max_x);
        let new_y = mouse_y.saturating_sub(d.drag_offset_y).clamp(TOP_BAR_H + 1, max_y.max(TOP_BAR_H + 1));
        d.windows[slot].x = new_x;
        d.windows[slot].y = new_y;
        return true;
    }
    false
}

/// Handle right mouse click on desktop -> open context menu
pub fn handle_right_click(mouse_x: usize, mouse_y: usize) {
    let (screen_w, screen_h) = {
        let g = GRAPHICS.lock();
        (g.width, g.height)
    };
    if screen_w == 0 || screen_h == 0 {
        return;
    }
    if mouse_y <= TOP_BAR_H || mouse_y >= screen_h.saturating_sub(TASKBAR_H) {
        return;
    }
    let mut d = DESKTOP.lock();
    d.launcher_open = false;
    d.ctx_menu_open = true;
    d.ctx_menu_x = mouse_x;
    d.ctx_menu_y = mouse_y;
    d.sync_active_app();
}

/// Handle left mouse click across Taskbar, System Tray, Context Menu, Launcher, Windows, and Desktop Icons
pub fn handle_mouse_click(mouse_x: usize, mouse_y: usize) {
    let (screen_w, screen_h) = {
        let g = GRAPHICS.lock();
        (g.width, g.height)
    };
    if screen_w == 0 || screen_h == 0 {
        return;
    }

    // 1. Top Status Bar clicks (Power button & Network badge)
    if mouse_y < TOP_BAR_H {
        let pwr_x = screen_w.saturating_sub(46);
        let right_x = screen_w.saturating_sub(420);
        if mouse_x >= pwr_x {
            DESKTOP.lock().open_or_focus_app(ActiveApp::Settings, screen_w, screen_h);
            return;
        }
        if mouse_x >= right_x && mouse_x < right_x + 190 {
            DESKTOP.lock().open_or_focus_app(ActiveApp::NetworkManager, screen_w, screen_h);
            return;
        }
        return;
    }

    // 2. Bottom Taskbar & System Tray clicks
    let tb_y = screen_h.saturating_sub(TASKBAR_H);
    if mouse_y >= tb_y {
        let tray_x = screen_w.saturating_sub(196);
        // System Tray: Network icon
        if mouse_x >= tray_x && mouse_x < tray_x + 24 {
            DESKTOP.lock().open_or_focus_app(ActiveApp::NetworkManager, screen_w, screen_h);
            return;
        }
        // System Tray: Volume icon (mute toggle)
        if mouse_x >= tray_x + 24 && mouse_x < tray_x + 48 {
            let mut d = DESKTOP.lock();
            d.volume_muted = !d.volume_muted;
            return;
        }

        // Start & Pinned Icons strip
        let icon_slot_w: usize = 42;
        let total_icons = 1 + TASKBAR_PINNED.len();
        let strip_w = total_icons * icon_slot_w;
        let tray_reserve: usize = 200;
        let strip_x = if screen_w > strip_w + tray_reserve + 24 {
            (screen_w.saturating_sub(tray_reserve) - strip_w) / 2
        } else {
            8
        };

        if mouse_x >= strip_x && mouse_x < strip_x + icon_slot_w {
            DESKTOP.lock().open_or_focus_app(ActiveApp::AppLauncher, screen_w, screen_h);
            return;
        }
        for (i, &(_icon, target_app)) in TASKBAR_PINNED.iter().enumerate() {
            let bx = strip_x + (i + 1) * icon_slot_w;
            if mouse_x >= bx && mouse_x < bx + icon_slot_w {
                let mut d = DESKTOP.lock();
                // Toggle minimize if already focused; else open/restore/focus
                if let Some(foc) = d.focused_window_slot() {
                    if d.windows[foc].app == target_app && !d.launcher_open {
                        d.minimize_slot(foc);
                        return;
                    }
                }
                d.open_or_focus_app(target_app, screen_w, screen_h);
                return;
            }
        }
        return;
    }

    // 3. Right-Click Context Menu clicks
    {
        let mut d = DESKTOP.lock();
        if d.ctx_menu_open {
            let menu_w: usize = 186;
            let item_h: usize = 24;
            let menu_h: usize = CTX_ITEMS.len() * item_h + 10;
            let mx = d.ctx_menu_x.min(screen_w.saturating_sub(menu_w + 4));
            let my = d.ctx_menu_y.min(screen_h.saturating_sub(TASKBAR_H + menu_h + 4));
            d.ctx_menu_open = false;

            if mouse_x >= mx && mouse_x <= mx + menu_w && mouse_y >= my + 5 && mouse_y < my + 5 + CTX_ITEMS.len() * item_h {
                let idx = (mouse_y - (my + 5)) / item_h;
                match idx {
                    0 => d.open_or_focus_app(ActiveApp::AppLauncher, screen_w, screen_h),
                    1 => d.open_or_focus_app(ActiveApp::FileManager, screen_w, screen_h),
                    2 => d.open_or_focus_app(ActiveApp::Terminal, screen_w, screen_h),
                    3 => d.open_or_focus_app(ActiveApp::NetworkManager, screen_w, screen_h),
                    4 => d.open_or_focus_app(ActiveApp::Settings, screen_w, screen_h),
                    _ => {} // Refresh desktop
                }
                return;
            }
        }
    }

    // 4. App Launcher Overlay clicks
    {
        let mut d = DESKTOP.lock();
        if d.launcher_open {
            let (lx, ly, lw, lh) = launcher_rect(screen_w, screen_h);
            if mouse_x < lx || mouse_x > lx + lw || mouse_y < ly || mouse_y > ly + lh {
                d.launcher_open = false;
                d.sync_active_app();
                return;
            }

            // Category sidebar
            let side_x = lx + 14;
            let side_y = ly + 54;
            let side_w: usize = 124;
            let cats = [
                LauncherCategory::All,
                LauncherCategory::Productivity,
                LauncherCategory::System,
                LauncherCategory::Games,
            ];
            for (i, &cat) in cats.iter().enumerate() {
                let cy = side_y + i * 32;
                if mouse_x >= side_x && mouse_x < side_x + side_w && mouse_y >= cy && mouse_y < cy + 26 {
                    d.launcher_category = cat;
                    return;
                }
            }

            // Power buttons in sidebar
            let pwr_y = ly + lh - 68;
            if mouse_x >= side_x && mouse_x < side_x + side_w && mouse_y >= pwr_y && mouse_y < pwr_y + 24 {
                drop(d);
                acpi::reboot();
            }
            if mouse_x >= side_x && mouse_x < side_x + side_w && mouse_y >= pwr_y + 30 && mouse_y < pwr_y + 54 {
                drop(d);
                acpi::shutdown();
            }

            // App grid clicks
            let grid_x = lx + side_w + 34;
            let grid_y = side_y;
            let cell_w: usize = 108;
            let cell_h: usize = 74;
            let cols: usize = 4;
            let search_buf = d.search_buf;
            let search_len = d.search_len;
            let active_cat = d.launcher_category;
            let search_str = core::str::from_utf8(&search_buf[..search_len]).unwrap_or("");

            let mut shown = 0usize;
            for entry in LAUNCHER_ENTRIES {
                if active_cat != LauncherCategory::All && entry.cat != active_cat {
                    continue;
                }
                if !contains_ignore_case(entry.name, search_str) {
                    continue;
                }
                if shown >= 16 {
                    break;
                }
                let col = shown % cols;
                let row = shown / cols;
                let cx = grid_x + col * (cell_w + 8);
                let cy = grid_y + row * (cell_h + 8);
                if mouse_x >= cx && mouse_x < cx + cell_w && mouse_y >= cy && mouse_y < cy + cell_h {
                    if let Some(gt) = entry.game {
                        GAMES.lock().current_game = gt;
                    }
                    d.search_len = 0;
                    d.open_or_focus_app(entry.app, screen_w, screen_h);
                    return;
                }
                shown += 1;
            }
            return;
        }
    }

    // 5. Open Windows hit-testing (topmost z-order first!)
    let hit_slot: Option<usize> = {
        let d = DESKTOP.lock();
        let mut best: Option<(usize, u32)> = None;
        for i in 0..MAX_WINDOWS {
            let w = d.windows[i];
            if w.open && !w.minimized
                && mouse_x >= w.x && mouse_x < w.x + w.w
                && mouse_y >= w.y && mouse_y < w.y + w.h
            {
                if best.map_or(true, |(_, bz)| w.z > bz) {
                    best = Some((i, w.z));
                }
            }
        }
        best.map(|(i, _)| i)
    };

    if let Some(slot) = hit_slot {
        let mut d = DESKTOP.lock();
        // Focus clicked window
        d.next_z += 1;
        d.windows[slot].z = d.next_z;
        d.sync_active_app();

        let win = d.windows[slot];
        let title_h: usize = 32;

        // Title bar controls (Minimize, Maximize/Restore, Close, or Drag)
        if mouse_y < win.y + title_h {
            let btn_w: usize = 28;
            let btn_h: usize = 22;
            let btn_y = win.y + 5;
            let close_x = win.x + win.w - btn_w - 6;
            let max_x = close_x - btn_w - 4;
            let min_x = max_x - btn_w - 4;

            if mouse_y >= btn_y && mouse_y < btn_y + btn_h {
                if mouse_x >= close_x && mouse_x < close_x + btn_w {
                    d.close_slot(slot);
                    return;
                }
                if mouse_x >= max_x && mouse_x < max_x + btn_w {
                    d.toggle_maximize_slot(slot, screen_w, screen_h);
                    return;
                }
                if mouse_x >= min_x && mouse_x < min_x + btn_w {
                    d.minimize_slot(slot);
                    return;
                }
            }
            // Start dragging window if not maximized
            if !win.maximized {
                d.dragging_slot = Some(slot);
                d.drag_offset_x = mouse_x.saturating_sub(win.x);
                d.drag_offset_y = mouse_y.saturating_sub(win.y);
            }
            return;
        }

        // In-app interactive click handling
        let cx = win.x + 16;
        let cy = win.y + title_h + 12;

        match win.app {
            ActiveApp::Calculator => {
                let calc_w: usize = 296;
                let calc_x = win.x + win.w.saturating_sub(calc_w) / 2;
                let grid_y = cy + 60;
                let buttons = [
                    ["7", "8", "9", "/"],
                    ["4", "5", "6", "*"],
                    ["1", "2", "3", "-"],
                    ["C", "0", "=", "+"],
                ];
                for (r, row) in buttons.iter().enumerate() {
                    let by = grid_y + r * 44;
                    for (c, &label) in row.iter().enumerate() {
                        let bx = calc_x + c * 76;
                        if mouse_x >= bx && mouse_x < bx + 68 && mouse_y >= by && mouse_y < by + 38 {
                            d.calc_press(label);
                            return;
                        }
                    }
                }
            }
            ActiveApp::FileManager => {
                let list_w = (win.w - 40) / 2;
                let list_y = cy + 22;
                if mouse_x >= cx && mouse_x < cx + list_w - 8 && mouse_y >= list_y {
                    let idx = (mouse_y - list_y) / 28;
                    if idx < 8 {
                        d.selected_file_idx = idx;
                    }
                }
            }
            ActiveApp::ImageViewer => {
                let by = cy + 22;
                if mouse_y >= by && mouse_y < by + 24 {
                    if mouse_x >= cx && mouse_x < cx + 160 {
                        d.viewer_image_idx = 0;
                    } else if mouse_x >= cx + 172 && mouse_x < cx + 332 {
                        d.viewer_image_idx = 1;
                    }
                }
            }
            ActiveApp::NetworkManager => {
                let btn_y = cy + 24 + 18 + 18 + 26;
                if mouse_x >= cx + 4 && mouse_x < cx + 184 && mouse_y >= btn_y && mouse_y < btn_y + 24 {
                    let ok = if e1000::is_link_up() {
                        net::send_icmp_ping([192, 168, 1, 1], 1)
                    } else {
                        false
                    };
                    d.net_ping_ok = Some(ok);
                }
            }
            ActiveApp::Settings => {
                let btn_y = cy + 7 * 20 + 16 + 22;
                if mouse_y >= btn_y && mouse_y < btn_y + 28 {
                    if mouse_x >= cx && mouse_x < cx + 140 {
                        drop(d);
                        acpi::reboot();
                    }
                    if mouse_x >= cx + 156 && mouse_x < cx + 306 {
                        drop(d);
                        acpi::shutdown();
                    }
                }
            }
            _ => {}
        }
        return;
    }

    // 6. Desktop Icons (Single-click select, Double-click open)
    let tile_w: usize = 78;
    let tile_h: usize = 68;
    let start_x: usize = 16;
    let start_y: usize = TOP_BAR_H + 14;
    let gap_y: usize = 10;
    let now_tick = crate::interrupts::TIMER_TICKS.load(Ordering::Relaxed);

    let mut d = DESKTOP.lock();
    for (idx, &(_label, _icon, target_app)) in DESKTOP_ICONS.iter().enumerate() {
        let col = idx / 6;
        let row = idx % 6;
        let ix = start_x + col * (tile_w + 12);
        let iy = start_y + row * (tile_h + gap_y);
        if mouse_x >= ix && mouse_x <= ix + tile_w && mouse_y >= iy && mouse_y <= iy + tile_h {
            let is_double = d.last_icon_click_idx == Some(idx)
                && now_tick.saturating_sub(d.last_icon_click_tick) <= 80;
            d.selected_desktop_icon = Some(idx);
            d.last_icon_click_idx = Some(idx);
            d.last_icon_click_tick = now_tick;
            if is_double {
                d.open_or_focus_app(target_app, screen_w, screen_h);
            }
            return;
        }
    }

    // Clicked empty desktop -> clear selection
    d.selected_desktop_icon = None;
}
