//! SRIKESH OS - Chrome-Inspired Web Browser & Playable Offline Dinosaur Runner
//!
//! Features:
//! - Chrome-inspired Multi-Tab bar (up to 3 tabs + [+] New Tab button)
//! - Rounded Omnibox address/search bar + Back / Forward / Refresh / Home toolbar
//! - New Tab Page with clickable Google, YouTube, Example.com, NeverSSL, CERN & Dino shortcuts
//! - Real UDP DNS + TCP 3-way handshake + Live HTTP/1.0 HTML parser & renderer
//! - Polished Chrome-style Offline Page ("No internet") with a REAL playable T-Rex Dinosaur Game

use crate::graphics::{Color, GraphicsEngine};
use crate::keyboard::{Key, KeyEvent};
use crate::text_input::TextField;
use crate::e1000::{self, E1000};
use crate::net;
use crate::serial;

const MAX_HTTP_BYTES: usize = 8192;
const MAX_RENDER_TEXT: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    NewTab,
    LiveWeb,
    OfflineDino,
}

pub struct WebBrowser {
    pub url_field: TextField,
    pub mode: ViewMode,
    pub tab_count: usize,
    pub active_tab: usize,
    pub tab_titles: [[u8; 20]; 3],
    pub tab_title_lens: [usize; 3],
    pub page_title: [u8; 96],
    pub page_title_len: usize,
    pub page_text: [u8; MAX_RENDER_TEXT],
    pub page_text_len: usize,
    pub http_status_line: [u8; 64],
    pub http_status_len: usize,
    pub resolved_ip: [u8; 4],
    pub bytes_received: usize,
    pub scroll_line: usize,
    pub last_error: Option<&'static str>,
    pub history: [[u8; 96]; 8],
    pub history_lens: [usize; 8],
    pub history_count: usize,
    pub history_idx: usize,
    // Playable Chrome-style Offline Dinosaur Runner state
    pub dino_running: bool,
    pub dino_game_over: bool,
    pub dino_y: i32,          // height above ground (0 = on ground)
    pub dino_vy: i32,         // vertical velocity
    pub cactus_x: i32,        // obstacle x in game coordinates (0..420)
    pub cactus2_x: i32,       // second obstacle x
    pub dino_score: u32,
    pub dino_high_score: u32,
    pub dino_leg_frame: u8,
}

impl WebBrowser {
    pub const fn new() -> Self {
        let mut field = TextField::new(false);
        let init = b"http://example.com";
        let mut i = 0;
        while i < init.len() {
            field.buf[i] = init[i];
            i += 1;
        }
        field.len = init.len();
        field.cursor = init.len();

        let mut tt = [[0u8; 20]; 3];
        let t0 = b"New Tab";
        let mut k = 0;
        while k < t0.len() {
            tt[0][k] = t0[k];
            k += 1;
        }

        Self {
            url_field: field,
            mode: ViewMode::NewTab,
            tab_count: 1,
            active_tab: 0,
            tab_titles: tt,
            tab_title_lens: [t0.len(), 0, 0],
            page_title: [0; 96],
            page_title_len: 0,
            page_text: [0; MAX_RENDER_TEXT],
            page_text_len: 0,
            http_status_line: [0; 64],
            http_status_len: 0,
            resolved_ip: [0, 0, 0, 0],
            bytes_received: 0,
            scroll_line: 0,
            last_error: None,
            history: [[0; 96]; 8],
            history_lens: [0; 8],
            history_count: 0,
            history_idx: 0,
            dino_running: false,
            dino_game_over: false,
            dino_y: 0,
            dino_vy: 0,
            cactus_x: 360,
            cactus2_x: 560,
            dino_score: 0,
            dino_high_score: 0,
            dino_leg_frame: 0,
        }
    }

    fn set_active_tab_title(&mut self, s: &str) {
        let b = s.as_bytes();
        let n = b.len().min(18);
        self.tab_titles[self.active_tab][..n].copy_from_slice(&b[..n]);
        self.tab_title_lens[self.active_tab] = n;
    }

    pub fn start_or_jump_dino(&mut self) {
        if self.dino_game_over || !self.dino_running {
            self.dino_running = true;
            self.dino_game_over = false;
            self.dino_y = 0;
            self.dino_vy = 16;
            self.cactus_x = 380;
            self.cactus2_x = 580;
            self.dino_score = 0;
        } else if self.dino_y == 0 {
            self.dino_vy = 16;
        }
    }

    /// Advances the offline Dinosaur Runner game by one physics tick.
    /// Returns true if the screen needs a re-render.
    pub fn tick_dino(&mut self) -> bool {
        if self.mode != ViewMode::OfflineDino || !self.dino_running || self.dino_game_over {
            return false;
        }

        // Jump gravity physics
        if self.dino_y > 0 || self.dino_vy > 0 {
            self.dino_y += self.dino_vy;
            self.dino_vy -= 2;
            if self.dino_y <= 0 {
                self.dino_y = 0;
                self.dino_vy = 0;
            }
        }

        // Scroll cacti faster as score increases
        let speed = 7 + (self.dino_score / 150).min(8) as i32;
        self.cactus_x -= speed;
        self.cactus2_x -= speed;
        if self.cactus_x < -20 {
            self.cactus_x = 420 + ((self.dino_score * 37) % 120) as i32;
        }
        if self.cactus2_x < -20 {
            self.cactus2_x = self.cactus_x + 190 + ((self.dino_score * 53) % 100) as i32;
        }

        self.dino_score = self.dino_score.saturating_add(1);
        if self.dino_score > self.dino_high_score {
            self.dino_high_score = self.dino_score;
        }
        self.dino_leg_frame = (self.dino_leg_frame + 1) % 4;

        // Collision check (Dino is at x=46..68, y=dino_y..dino_y+26; cactus height=24)
        let hit1 = self.cactus_x >= 38 && self.cactus_x <= 66 && self.dino_y < 22;
        let hit2 = self.cactus2_x >= 38 && self.cactus2_x <= 66 && self.dino_y < 22;
        if hit1 || hit2 {
            self.dino_game_over = true;
            self.dino_running = false;
        }
        true
    }

    pub fn set_url(&mut self, url: &str) {
        self.url_field.set_str(url);
        self.navigate(true);
    }

    pub fn handle_key(&mut self, ev: KeyEvent) {
        // Space or Up arrow jumps in Offline Dinosaur Runner!
        if self.mode == ViewMode::OfflineDino && (ev.key == Key::Up || ev.key == Key::Char(' ')) {
            self.start_or_jump_dino();
            return;
        }
        if self.mode == ViewMode::LiveWeb {
            match ev.key {
                Key::Up => {
                    if self.scroll_line > 0 { self.scroll_line -= 1; }
                    return;
                }
                Key::Down => {
                    self.scroll_line = self.scroll_line.saturating_add(1);
                    return;
                }
                Key::PageUp => {
                    self.scroll_line = self.scroll_line.saturating_sub(8);
                    return;
                }
                Key::PageDown => {
                    self.scroll_line = self.scroll_line.saturating_add(8);
                    return;
                }
                _ => {}
            }
        }
        if self.url_field.handle_key(ev) {
            self.navigate(true);
        }
    }

    fn parse_host_path<'a>(raw: &'a str) -> (&'a str, &'a str) {
        let trimmed = raw.trim();
        let no_scheme = if let Some(rest) = trimmed.strip_prefix("http://") {
            rest
        } else if let Some(rest) = trimmed.strip_prefix("https://") {
            rest
        } else {
            trimmed
        };
        if let Some(slash) = no_scheme.find('/') {
            (&no_scheme[..slash], &no_scheme[slash..])
        } else {
            (no_scheme, "/")
        }
    }

    pub fn navigate(&mut self, record_history: bool) {
        let mut url_copy = [0u8; 128];
        let ulen = self.url_field.len.min(127);
        url_copy[..ulen].copy_from_slice(&self.url_field.buf[..ulen]);
        let url_str = core::str::from_utf8(&url_copy[..ulen]).unwrap_or("").trim();

        if url_str.is_empty() || url_str == "srikesh://newtab" {
            self.mode = ViewMode::NewTab;
            self.set_active_tab_title("New Tab");
            return;
        }
        if url_str == "chrome://dino" || url_str == "srikesh://dino" {
            self.mode = ViewMode::OfflineDino;
            self.last_error = Some("ERR_INTERNET_DISCONNECTED");
            self.set_active_tab_title("Offline Dino");
            return;
        }

        if record_history {
            let slot = self.history_count.min(7);
            let copy_n = ulen.min(96);
            self.history[slot][..copy_n].copy_from_slice(&url_copy[..copy_n]);
            self.history_lens[slot] = copy_n;
            self.history_idx = slot;
            if self.history_count < 8 {
                self.history_count += 1;
            }
        }

        let (host, path) = Self::parse_host_path(url_str);
        if host.is_empty() {
            return;
        }

        serial::write_str("[BROWSER] Live HTTP GET: ");
        serial::write_str(host);
        serial::write_str(path);
        serial::write_str("\n");

        self.scroll_line = 0;
        self.last_error = None;
        self.bytes_received = 0;
        self.http_status_len = 0;

        let mut raw_buf = [0u8; MAX_HTTP_BYTES];
        match net::http_get(host, path, &mut raw_buf) {
            Ok((total, ip)) => {
                self.mode = ViewMode::LiveWeb;
                self.resolved_ip = ip;
                self.bytes_received = total;
                self.parse_http_response(&raw_buf[..total], host);
                self.set_active_tab_title(host);
                serial::write_str("[BROWSER] Received ");
                serial::write_u64(total as u64);
                serial::write_str(" live bytes from remote server!\n");
            }
            Err(err_msg) => {
                // Offline -> show polished Chrome-style offline page with playable Dinosaur Runner!
                self.mode = ViewMode::OfflineDino;
                self.last_error = Some(err_msg);
                self.set_active_tab_title("No internet");
                self.dino_running = false;
                self.dino_game_over = false;
            }
        }
    }

    fn set_title(&mut self, s: &str) {
        let b = s.as_bytes();
        let n = b.len().min(95);
        self.page_title[..n].copy_from_slice(&b[..n]);
        self.page_title_len = n;
    }

    fn append_text(&mut self, s: &str) {
        for &b in s.as_bytes() {
            if self.page_text_len < MAX_RENDER_TEXT {
                self.page_text[self.page_text_len] = b;
                self.page_text_len += 1;
            }
        }
    }

    fn parse_http_response(&mut self, raw: &[u8], fallback_host: &str) {
        let mut body_start = 0usize;
        for i in 0..raw.len().saturating_sub(3) {
            if &raw[i..i + 4] == b"\r\n\r\n" {
                body_start = i + 4;
                break;
            }
        }

        let mut line_end = raw.len().min(63);
        for i in 0..line_end {
            if raw[i] == b'\r' || raw[i] == b'\n' {
                line_end = i;
                break;
            }
        }
        self.http_status_line[..line_end].copy_from_slice(&raw[..line_end]);
        self.http_status_len = line_end;

        let body = if body_start > 0 && body_start < raw.len() {
            &raw[body_start..]
        } else {
            raw
        };

        self.set_title(fallback_host);
        if let Ok(body_str) = core::str::from_utf8(body) {
            if let Some(ts) = body_str.find("<title>").or_else(|| body_str.find("<TITLE>")) {
                let after = &body_str[ts + 7..];
                if let Some(te) = after.find("</title>").or_else(|| after.find("</TITLE>")) {
                    self.set_title(after[..te].trim());
                }
            }
        }

        self.page_text_len = 0;
        let mut in_tag = false;
        let mut in_skip_block = false;
        let mut last_was_space = true;
        let mut i = 0;
        while i < body.len() && self.page_text_len < MAX_RENDER_TEXT - 2 {
            if !in_tag && body[i] == b'<' {
                in_tag = true;
                let rem = &body[i..];
                if rem.len() >= 7 && (rem[..7].eq_ignore_ascii_case(b"<script") || rem[..6].eq_ignore_ascii_case(b"<style")) {
                    in_skip_block = true;
                } else if rem.len() >= 8 && (rem[..8].eq_ignore_ascii_case(b"</script") || rem[..7].eq_ignore_ascii_case(b"</style")) {
                    in_skip_block = false;
                } else if rem.len() >= 3 && (rem[..3].eq_ignore_ascii_case(b"<br") || rem[..2].eq_ignore_ascii_case(b"<p") || rem[..3].eq_ignore_ascii_case(b"</p") || rem[..3].eq_ignore_ascii_case(b"</h") || rem[..4].eq_ignore_ascii_case(b"</di") || rem[..3].eq_ignore_ascii_case(b"<li")) {
                    if self.page_text_len > 0 && self.page_text[self.page_text_len - 1] != b'\n' {
                        self.page_text[self.page_text_len] = b'\n';
                        self.page_text_len += 1;
                        last_was_space = true;
                    }
                }
                i += 1;
                continue;
            }
            if in_tag {
                if body[i] == b'>' { in_tag = false; }
                i += 1;
                continue;
            }
            if !in_skip_block {
                let b = body[i];
                if b == b'\r' || b == b'\n' || b == b'\t' || b == b' ' {
                    if !last_was_space {
                        self.page_text[self.page_text_len] = b' ';
                        self.page_text_len += 1;
                        last_was_space = true;
                    }
                } else if b >= 0x20 && b <= 0x7E {
                    self.page_text[self.page_text_len] = b;
                    self.page_text_len += 1;
                    last_was_space = false;
                }
            }
            i += 1;
        }

        if self.page_text_len < 40 && body_start > 0 {
            self.append_text("\n--- Live HTTP Server Response Headers ---\n");
            let hdr = &raw[..body_start.min(500)];
            for &b in hdr {
                if (b >= 0x20 && b <= 0x7E) || b == b'\n' {
                    if self.page_text_len < MAX_RENDER_TEXT {
                        self.page_text[self.page_text_len] = b;
                        self.page_text_len += 1;
                    }
                }
            }
        }
    }

    pub fn handle_click(&mut self, mouse_x: usize, mouse_y: usize, win_x: usize, win_y: usize, win_w: usize) {
        // 1. Tab Strip clicks (win_y + 34 .. win_y + 56)
        let tab_y = win_y + 34;
        if mouse_y >= tab_y && mouse_y < tab_y + 22 {
            for t in 0..self.tab_count {
                let tx = win_x + 10 + t * 136;
                if mouse_x >= tx && mouse_x < tx + 130 {
                    self.active_tab = t;
                    return;
                }
            }
            let plus_x = win_x + 10 + self.tab_count * 136;
            if mouse_x >= plus_x && mouse_x < plus_x + 26 && self.tab_count < 3 {
                self.active_tab = self.tab_count;
                self.tab_count += 1;
                self.set_active_tab_title("New Tab");
                self.mode = ViewMode::NewTab;
                return;
            }
        }

        // 2. Chrome Toolbar clicks (win_y + 58 .. win_y + 88)
        let nav_y = win_y + 58;
        if mouse_y >= nav_y + 3 && mouse_y < nav_y + 27 {
            if mouse_x >= win_x + 12 && mouse_x < win_x + 36 {
                if self.history_idx > 0 {
                    self.history_idx -= 1;
                    let len = self.history_lens[self.history_idx];
                    let mut tmp = [0u8; 96];
                    tmp[..len].copy_from_slice(&self.history[self.history_idx][..len]);
                    if let Ok(s) = core::str::from_utf8(&tmp[..len]) {
                        self.url_field.set_str(s);
                        self.navigate(false);
                    }
                }
                return;
            }
            if mouse_x >= win_x + 40 && mouse_x < win_x + 64 {
                if self.history_idx + 1 < self.history_count {
                    self.history_idx += 1;
                    let len = self.history_lens[self.history_idx];
                    let mut tmp = [0u8; 96];
                    tmp[..len].copy_from_slice(&self.history[self.history_idx][..len]);
                    if let Ok(s) = core::str::from_utf8(&tmp[..len]) {
                        self.url_field.set_str(s);
                        self.navigate(false);
                    }
                }
                return;
            }
            if mouse_x >= win_x + 68 && mouse_x < win_x + 92 {
                self.navigate(false);
                return;
            }
            if mouse_x >= win_x + 96 && mouse_x < win_x + 120 {
                self.mode = ViewMode::NewTab;
                self.set_active_tab_title("New Tab");
                return;
            }
            let go_x = win_x + win_w - 58;
            if mouse_x >= go_x && mouse_x < go_x + 46 {
                self.navigate(true);
                return;
            }
        }

        // 3. New Tab Shortcut Cards clicks
        if self.mode == ViewMode::NewTab {
            let shortcuts = [
                ("Google",      "http://google.com"),
                ("YouTube",     "http://youtube.com"),
                ("Example.com", "http://example.com"),
                ("NeverSSL",    "http://neverssl.com"),
                ("CERN Web",    "http://info.cern.ch"),
                ("Dino Game",   "chrome://dino"),
            ];
            let card_w = 94usize;
            let total_w = shortcuts.len() * (card_w + 8) - 8;
            let sx = win_x + win_w.saturating_sub(total_w) / 2;
            let sy = win_y + 190;
            if mouse_y >= sy && mouse_y < sy + 64 {
                for (i, &(_lbl, url)) in shortcuts.iter().enumerate() {
                    let cx = sx + i * (card_w + 8);
                    if mouse_x >= cx && mouse_x < cx + card_w {
                        self.set_url(url);
                        return;
                    }
                }
            }
        } else if self.mode == ViewMode::OfflineDino {
            // Clicking anywhere inside Dino viewport jumps / starts game!
            if mouse_y > nav_y + 32 {
                self.start_or_jump_dino();
            }
        }
    }

    pub fn render(&self, gfx: &mut GraphicsEngine, win_x: usize, win_y: usize, win_w: usize, win_h: usize) {
        // 1. Chrome-style Tab Strip
        let tab_y = win_y + 34;
        gfx.fill_rect(win_x + 8, tab_y, win_w - 16, 22, Color::new(15, 23, 42));
        for t in 0..self.tab_count {
            let tx = win_x + 10 + t * 136;
            let active = t == self.active_tab;
            gfx.fill_rounded_rect(tx, tab_y, 130, 22, 5, if active { Color::new(30, 41, 59) } else { Color::new(20, 29, 46) });
            if active {
                gfx.draw_rect_outline(tx, tab_y, 130, 22, Color::ROYAL_GOLD, 1);
            }
            let tname = core::str::from_utf8(&self.tab_titles[t][..self.tab_title_lens[t]]).unwrap_or("Tab");
            gfx.draw_text(tx + 8, tab_y + 7, tname, if active { Color::WHITE } else { Color::TEXT_MUTED }, 1);
        }
        if self.tab_count < 3 {
            let px = win_x + 10 + self.tab_count * 136;
            gfx.fill_rounded_rect(px, tab_y + 2, 24, 18, 4, Color::new(30, 41, 59));
            gfx.draw_text(px + 8, tab_y + 7, "+", Color::BRIGHT_GOLD, 1);
        }

        // 2. Chrome-style Navigation & Rounded Omnibox Toolbar
        let nav_y = win_y + 58;
        gfx.fill_rect(win_x + 8, nav_y, win_w - 16, 30, Color::new(24, 34, 52));
        gfx.draw_rect_outline(win_x + 8, nav_y, win_w - 16, 30, Color::DARK_GOLD, 1);

        for (i, lbl) in ["<", ">", "R", "H"].iter().enumerate() {
            let bx = win_x + 12 + i * 28;
            gfx.fill_rounded_rect(bx, nav_y + 3, 24, 24, 4, Color::new(30, 41, 59));
            gfx.draw_text(bx + 8, nav_y + 10, lbl, Color::BRIGHT_GOLD, 1);
        }

        let url_x = win_x + 126;
        let url_w = win_w.saturating_sub(192);
        self.url_field.render(gfx, url_x, nav_y + 3, url_w, 24, "Search or enter URL (example.com, google.com, chrome://dino)", true);

        let go_x = win_x + win_w - 58;
        gfx.fill_rounded_rect(go_x, nav_y + 3, 46, 24, 4, Color::ROYAL_GOLD);
        gfx.draw_text(go_x + 14, nav_y + 10, "Go", Color::OBSIDIAN, 1);

        // 3. Viewport
        let vx = win_x + 10;
        let vy = nav_y + 34;
        let vw = win_w - 20;
        let vh = win_h.saturating_sub((vy - win_y) + 24);
        gfx.fill_rounded_rect(vx, vy, vw, vh, 6, Color::new(15, 23, 42));
        gfx.draw_rect_outline(vx, vy, vw, vh, Color::DARK_GOLD, 1);

        match self.mode {
            ViewMode::NewTab => {
                let logo = "S R I K E S H";
                let lw = logo.len() * 16;
                gfx.draw_text(vx + vw.saturating_sub(lw) / 2, vy + 36, logo, Color::BRIGHT_GOLD, 2);
                let sub = "Chrome-Inspired Browser  |  Live DNS + TCP + HTTP";
                let sw = sub.len() * 8;
                gfx.draw_text(vx + vw.saturating_sub(sw) / 2, vy + 64, sub, Color::ROYAL_SAND, 1);

                // Clickable Quick Shortcut Cards
                let shortcuts = [
                    ("G",  "Google",      Color::new(59, 130, 246)),
                    ("YT", "YouTube",     Color::new(239, 68, 68)),
                    ("EX", "Example.com", Color::new(14, 165, 233)),
                    ("NS", "NeverSSL",    Color::new(34, 197, 94)),
                    ("W3", "CERN Web",    Color::new(168, 85, 247)),
                    ("DN", "Dino Game",   Color::new(251, 191, 36)),
                ];
                let card_w = 94usize;
                let total_w = shortcuts.len() * (card_w + 8) - 8;
                let sx = win_x + win_w.saturating_sub(total_w) / 2;
                let sy = win_y + 190;
                for (i, &(badge, name, col)) in shortcuts.iter().enumerate() {
                    let cx = sx + i * (card_w + 8);
                    gfx.fill_rounded_rect(cx, sy, card_w, 64, 8, Color::new(26, 36, 56));
                    gfx.draw_rect_outline(cx, sy, card_w, 64, Color::DARK_GOLD, 1);
                    gfx.fill_rounded_rect(cx + (card_w - 30) / 2, sy + 8, 30, 26, 13, col);
                    gfx.draw_text(cx + (card_w - badge.len() * 8) / 2, sy + 17, badge, Color::WHITE, 1);
                    gfx.draw_text(cx + card_w.saturating_sub(name.len() * 8) / 2, sy + 44, name, Color::WHITE, 1);
                }
            }
            ViewMode::LiveWeb => {
                let title_str = core::str::from_utf8(&self.page_title[..self.page_title_len]).unwrap_or("SRIKESH Browser");
                gfx.fill_rect(vx + 2, vy + 2, vw - 4, 24, Color::new(24, 34, 54));
                gfx.draw_text(vx + 10, vy + 10, title_str, Color::BRIGHT_GOLD, 1);

                let text_str = core::str::from_utf8(&self.page_text[..self.page_text_len]).unwrap_or("");
                let max_cols = (vw.saturating_sub(24) / 8).max(20);
                let max_rows = (vh.saturating_sub(36) / 15).max(4);

                let mut row_idx = 0usize;
                let mut drawn_rows = 0usize;
                let start_y = vy + 32;

                for raw_line in text_str.split('\n') {
                    let bytes = raw_line.as_bytes();
                    let mut off = 0usize;
                    if bytes.is_empty() {
                        if row_idx >= self.scroll_line && drawn_rows < max_rows { drawn_rows += 1; }
                        row_idx += 1;
                        continue;
                    }
                    while off < bytes.len() {
                        let end = (off + max_cols).min(bytes.len());
                        if row_idx >= self.scroll_line && drawn_rows < max_rows {
                            if let Ok(chunk) = core::str::from_utf8(&bytes[off..end]) {
                                gfx.draw_text(vx + 12, start_y + drawn_rows * 15, chunk, Color::WHITE, 1);
                            }
                            drawn_rows += 1;
                        }
                        row_idx += 1;
                        off = end;
                    }
                }
            }
            ViewMode::OfflineDino => {
                // Chrome-style Offline Page + Playable T-Rex Dinosaur Runner!
                let gx = vx + vw.saturating_sub(440) / 2;
                let gy = vy + 24;
                gfx.draw_text(gx, gy, "No internet", Color::WHITE, 2);
                gfx.draw_text(gx, gy + 26, "ERR_INTERNET_DISCONNECTED  |  Press SPACE or UP to play Dino!", Color::ROYAL_SAND, 1);

                // Game Canvas (440 x 140)
                let cy = gy + 48;
                gfx.fill_rounded_rect(gx, cy, 440, 140, 8, Color::new(22, 30, 46));
                gfx.draw_rect_outline(gx, cy, 440, 140, Color::ROYAL_GOLD, 1);

                // Score & High Score
                let mut sb = [0u8; 12];
                let score_s = fmt_u32(self.dino_score, &mut sb);
                let mut hb = [0u8; 12];
                let hi_s = fmt_u32(self.dino_high_score, &mut hb);
                gfx.draw_text(gx + 260, cy + 10, "HI ", Color::TEXT_MUTED, 1);
                gfx.draw_text(gx + 284, cy + 10, hi_s, Color::ROYAL_SAND, 1);
                gfx.draw_text(gx + 350, cy + 10, "SCORE ", Color::ROYAL_GOLD, 1);
                gfx.draw_text(gx + 398, cy + 10, score_s, Color::BRIGHT_GOLD, 1);

                // Ground line
                let ground_y = cy + 114;
                gfx.fill_rect(gx + 10, ground_y, 420, 2, Color::ROYAL_SAND);

                // Draw Pixel-Art T-Rex Dinosaur
                let dx = gx + 46;
                let dy = ground_y.saturating_sub(28 + self.dino_y.max(0) as usize);
                // Head & jaw
                gfx.fill_rect(dx + 10, dy, 14, 10, Color::BRIGHT_GOLD);
                gfx.fill_rect(dx + 14, dy + 2, 2, 2, Color::OBSIDIAN); // eye
                // Body & tail
                gfx.fill_rect(dx + 4, dy + 10, 14, 12, Color::ROYAL_GOLD);
                gfx.fill_rect(dx, dy + 12, 5, 6, Color::ROYAL_GOLD);
                // Animated legs
                if self.dino_leg_frame < 2 {
                    gfx.fill_rect(dx + 6, dy + 22, 4, 6, Color::BRIGHT_GOLD);
                    gfx.fill_rect(dx + 13, dy + 22, 4, 4, Color::BRIGHT_GOLD);
                } else {
                    gfx.fill_rect(dx + 6, dy + 22, 4, 4, Color::BRIGHT_GOLD);
                    gfx.fill_rect(dx + 13, dy + 22, 4, 6, Color::BRIGHT_GOLD);
                }

                // Draw Cacti Obstacles
                for &cx_rel in &[self.cactus_x, self.cactus2_x] {
                    if cx_rel >= 12 && cx_rel <= 415 {
                        let ox = gx + cx_rel as usize;
                        gfx.fill_rect(ox + 4, ground_y - 24, 6, 24, Color::SUCCESS_GREEN);
                        gfx.fill_rect(ox, ground_y - 16, 4, 8, Color::SUCCESS_GREEN);
                        gfx.fill_rect(ox + 10, ground_y - 18, 4, 8, Color::SUCCESS_GREEN);
                    }
                }

                if self.dino_game_over {
                    gfx.draw_text(gx + 130, cy + 54, "G A M E   O V E R", Color::DANGER_RED, 1);
                    gfx.draw_text(gx + 112, cy + 72, "Press SPACE or Click to Restart", Color::WHITE, 1);
                } else if !self.dino_running {
                    gfx.draw_text(gx + 108, cy + 58, "Press SPACE or Click to Start!", Color::BRIGHT_GOLD, 1);
                }
            }
        }

        // Bottom Status Strip
        let sy = win_y + win_h - 19;
        let eth_up = e1000::is_link_up();
        if self.mode == ViewMode::LiveWeb && self.http_status_len > 0 {
            let st = core::str::from_utf8(&self.http_status_line[..self.http_status_len]).unwrap_or("");
            gfx.draw_text(vx + 4, sy, st, Color::SUCCESS_GREEN, 1);
        } else if let Some(err) = self.last_error {
            gfx.draw_text(vx + 4, sy, err, Color::ROYAL_AMBER, 1);
        } else {
            let dhcp = E1000.lock().dhcp_ok;
            gfx.draw_text(
                vx + 4,
                sy,
                if eth_up && dhcp { "Online (DHCP Active) - Ready" } else if eth_up { "Ethernet Link UP - Auto-DHCP on navigate" } else { "Offline - Click Dino Game or check Network" },
                if eth_up { Color::ROYAL_GOLD } else { Color::TEXT_MUTED },
                1,
            );
        }
    }
}

fn fmt_u32(mut n: u32, buf: &mut [u8; 12]) -> &str {
    if n == 0 {
        buf[0] = b'0';
        return core::str::from_utf8(&buf[..1]).unwrap_or("0");
    }
    let mut tmp = [0u8; 12];
    let mut c = 0;
    while n > 0 && c < 12 {
        tmp[c] = b'0' + (n % 10) as u8;
        n /= 10;
        c += 1;
    }
    for i in 0..c {
        buf[i] = tmp[c - 1 - i];
    }
    core::str::from_utf8(&buf[..c]).unwrap_or("0")
}

pub static BROWSER: spin::Mutex<WebBrowser> = spin::Mutex::new(WebBrowser::new());
