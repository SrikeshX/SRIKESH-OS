//! SRIKESH OS - Reusable Text Input Control & System Clipboard Foundation
//!
//! Provides genuine text editing across Terminal, Browser Address Bar, App Launcher Search,
//! Wi-Fi Password Field, and Text Editor:
//! - Cursor positioning, mid-line insertion, Backspace, Delete, Home, End, Left/Right arrows
//! - Shift+Arrow text selection & Ctrl+A Select All
//! - Real System Clipboard (Ctrl+C Copy, Ctrl+X Cut, Ctrl+V Paste)
//! - Password masking (`*`) and focus highlight rendering

use crate::graphics::{Color, GraphicsEngine};
use crate::keyboard::{Key, KeyEvent};
use spin::Mutex;

pub struct Clipboard {
    pub data: [u8; 256],
    pub len: usize,
}

impl Clipboard {
    pub const fn new() -> Self {
        Self { data: [0; 256], len: 0 }
    }

    pub fn set(&mut self, bytes: &[u8]) {
        let n = bytes.len().min(self.data.len());
        for i in 0..n {
            self.data[i] = bytes[i];
        }
        self.len = n;
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data[..self.len]
    }
}

pub static CLIPBOARD: Mutex<Clipboard> = Mutex::new(Clipboard::new());

#[derive(Clone, Copy)]
pub struct TextField {
    pub buf: [u8; 128],
    pub len: usize,
    pub cursor: usize,
    pub sel_anchor: Option<usize>,
    pub masked: bool,
}

impl TextField {
    pub const fn new(masked: bool) -> Self {
        Self {
            buf: [0; 128],
            len: 0,
            cursor: 0,
            sel_anchor: None,
            masked,
        }
    }

    pub fn set_str(&mut self, s: &str) {
        let bytes = s.as_bytes();
        let n = bytes.len().min(126);
        for i in 0..n {
            self.buf[i] = bytes[i];
        }
        self.len = n;
        self.cursor = n;
        self.sel_anchor = None;
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }

    pub fn clear(&mut self) {
        self.len = 0;
        self.cursor = 0;
        self.sel_anchor = None;
    }

    fn selection_range(&self) -> Option<(usize, usize)> {
        self.sel_anchor.and_then(|a| {
            if a == self.cursor {
                None
            } else if a < self.cursor {
                Some((a, self.cursor))
            } else {
                Some((self.cursor, a))
            }
        })
    }

    fn delete_selection(&mut self) -> bool {
        if let Some((start, end)) = self.selection_range() {
            let count = end - start;
            for i in start..(self.len - count) {
                self.buf[i] = self.buf[i + count];
            }
            self.len -= count;
            self.cursor = start;
            self.sel_anchor = None;
            true
        } else {
            false
        }
    }

    fn insert_byte(&mut self, b: u8) {
        self.delete_selection();
        if self.len >= 126 {
            return;
        }
        for i in (self.cursor..self.len).rev() {
            self.buf[i + 1] = self.buf[i];
        }
        self.buf[self.cursor] = b;
        self.len += 1;
        self.cursor += 1;
    }

    /// Processes a KeyEvent. Returns `true` if Enter was pressed (submit).
    pub fn handle_key(&mut self, ev: KeyEvent) -> bool {
        // Ctrl shortcuts
        if ev.ctrl {
            if let Key::Char(c) = ev.key {
                match c.to_ascii_lowercase() {
                    'a' => {
                        self.sel_anchor = Some(0);
                        self.cursor = self.len;
                        return false;
                    }
                    'c' => {
                        if let Some((s, e)) = self.selection_range() {
                            CLIPBOARD.lock().set(&self.buf[s..e]);
                        } else if self.len > 0 {
                            CLIPBOARD.lock().set(&self.buf[..self.len]);
                        }
                        return false;
                    }
                    'x' => {
                        if let Some((s, e)) = self.selection_range() {
                            CLIPBOARD.lock().set(&self.buf[s..e]);
                            self.delete_selection();
                        }
                        return false;
                    }
                    'v' => {
                        let mut tmp = [0u8; 128];
                        let n = {
                            let cb = CLIPBOARD.lock();
                            let bytes = cb.as_bytes();
                            let k = bytes.len().min(126);
                            for i in 0..k {
                                tmp[i] = bytes[i];
                            }
                            k
                        };
                        for i in 0..n {
                            let b = tmp[i];
                            if b >= 0x20 && b <= 0x7E {
                                self.insert_byte(b);
                            }
                        }
                        return false;
                    }
                    _ => return false,
                }
            }
        }

        match ev.key {
            Key::Enter => return true,
            Key::Left => {
                if ev.shift {
                    if self.sel_anchor.is_none() {
                        self.sel_anchor = Some(self.cursor);
                    }
                } else {
                    self.sel_anchor = None;
                }
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
            }
            Key::Right => {
                if ev.shift {
                    if self.sel_anchor.is_none() {
                        self.sel_anchor = Some(self.cursor);
                    }
                } else {
                    self.sel_anchor = None;
                }
                if self.cursor < self.len {
                    self.cursor += 1;
                }
            }
            Key::Home => {
                if ev.shift {
                    if self.sel_anchor.is_none() {
                        self.sel_anchor = Some(self.cursor);
                    }
                } else {
                    self.sel_anchor = None;
                }
                self.cursor = 0;
            }
            Key::End => {
                if ev.shift {
                    if self.sel_anchor.is_none() {
                        self.sel_anchor = Some(self.cursor);
                    }
                } else {
                    self.sel_anchor = None;
                }
                self.cursor = self.len;
            }
            Key::Backspace => {
                if !self.delete_selection() && self.cursor > 0 {
                    for i in (self.cursor - 1)..(self.len - 1) {
                        self.buf[i] = self.buf[i + 1];
                    }
                    self.len -= 1;
                    self.cursor -= 1;
                }
            }
            Key::Delete => {
                if !self.delete_selection() && self.cursor < self.len {
                    for i in self.cursor..(self.len - 1) {
                        self.buf[i] = self.buf[i + 1];
                    }
                    self.len -= 1;
                }
            }
            Key::Char(ch) => {
                let b = ch as u8;
                if b >= 0x20 && b <= 0x7E {
                    self.insert_byte(b);
                }
            }
            _ => {}
        }
        false
    }

    pub fn render(&self, gfx: &mut GraphicsEngine, x: usize, y: usize, w: usize, h: usize, placeholder: &str, focused: bool) {
        gfx.fill_rounded_rect(x, y, w, h, 5, Color::new(12, 10, 8));
        gfx.draw_rect_outline(x, y, w, h, if focused { Color::BRIGHT_GOLD } else { Color::DARK_GOLD }, 1);

        let text_x = x + 8;
        let text_y = y + (h.saturating_sub(8)) / 2;
        let max_chars = w.saturating_sub(18) / 8;

        if self.len == 0 && !placeholder.is_empty() {
            let clipped = if placeholder.len() > max_chars { &placeholder[..max_chars] } else { placeholder };
            gfx.draw_text(text_x, text_y, clipped, Color::TEXT_MUTED, 1);
        } else {
            let scroll = if self.cursor > max_chars { self.cursor - max_chars } else { 0 };
            let visible_end = (scroll + max_chars).min(self.len);

            // Selection highlight
            if let Some((s, e)) = self.selection_range() {
                let vs = s.max(scroll);
                let ve = e.min(visible_end);
                if ve > vs {
                    gfx.fill_rect(text_x + (vs - scroll) * 8, text_y.saturating_sub(2), (ve - vs) * 8, 12, Color::LAPIS_LIGHT);
                }
            }

            // Draw characters (or '*' when masked)
            for idx in scroll..visible_end {
                let ch = if self.masked { '*' } else { self.buf[idx] as char };
                gfx.draw_char(text_x + (idx - scroll) * 8, text_y, ch, Color::WHITE, 1);
            }

            // Caret cursor
            if focused {
                let caret_x = text_x + (self.cursor.saturating_sub(scroll)) * 8;
                if caret_x < x + w - 4 {
                    gfx.fill_rect(caret_x, text_y.saturating_sub(2), 2, 12, Color::BRIGHT_GOLD);
                }
            }
        }
    }
}
