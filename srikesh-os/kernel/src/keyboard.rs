//! SRIKESH OS - PS/2 Keyboard Driver
//!
//! Features:
//! - IRQ 1 (vector 33) handler.
//! - I/O port 0x60 data reader.
//! - Scancode Set 1 decoder with Shift, CapsLock, Ctrl modifier tracking.
//! - Lock-free atomic / spinlock FIFO character ring buffer.

use x86_64::instructions::port::Port;
use spin::Mutex;

const DATA_PORT: u16 = 0x60;
const STATUS_PORT: u16 = 0x64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    Char(char),
    Enter,
    Backspace,
    Tab,
    Escape,
    Up,
    Down,
    Left,
    Right,
    None,
}

struct KeyboardState {
    shift_pressed: bool,
    caps_lock: bool,
    ctrl_pressed: bool,
    extended_scancode: bool,
    buffer: [char; 128],
    head: usize,
    tail: usize,
}

impl KeyboardState {
    const fn new() -> Self {
        Self {
            shift_pressed: false,
            caps_lock: false,
            ctrl_pressed: false,
            extended_scancode: false,
            buffer: ['\0'; 128],
            head: 0,
            tail: 0,
        }
    }

    fn push_char(&mut self, ch: char) {
        let next_head = (self.head + 1) % self.buffer.len();
        if next_head != self.tail {
            self.buffer[self.head] = ch;
            self.head = next_head;
        }
    }

    fn pop_char(&mut self) -> Option<char> {
        if self.head == self.tail {
            None
        } else {
            let ch = self.buffer[self.tail];
            self.tail = (self.tail + 1) % self.buffer.len();
            Some(ch)
        }
    }
}

static KEYBOARD: Mutex<KeyboardState> = Mutex::new(KeyboardState::new());

/// Scan Code Set 1 lookup table (unshifted)
const SCANCODE_LOWER: [char; 58] = [
    '\0', '\x1B', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', '\x08',
    '\t', 'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']', '\n',
    '\0', 'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';', '\'', '`',
    '\0', '\\', 'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/', '\0',
    '*', '\0', ' ',
];

/// Scan Code Set 1 lookup table (shifted)
const SCANCODE_UPPER: [char; 58] = [
    '\0', '\x1B', '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '_', '+', '\x08',
    '\t', 'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', '{', '}', '\n',
    '\0', 'A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', ':', '"', '~',
    '\0', '|', 'Z', 'X', 'C', 'V', 'B', 'N', 'M', '<', '>', '?', '\0',
    '*', '\0', ' ',
];

/// Interrupt handler for IRQ 1 (Keyboard)
pub fn handle_keyboard_interrupt() {
    let mut status_port = Port::<u8>::new(STATUS_PORT);
    let mut data_port = Port::<u8>::new(DATA_PORT);

    let status = unsafe { status_port.read() };
    if (status & 0x01) == 0 {
        return; // No data in output buffer
    }

    let scancode = unsafe { data_port.read() };
    let mut kb = KEYBOARD.lock();

    if scancode == 0xE0 {
        kb.extended_scancode = true;
        return;
    }

    if kb.extended_scancode {
        kb.extended_scancode = false;
        // Ignore extended release codes
        if (scancode & 0x80) != 0 {
            return;
        }
        match scancode {
            0x48 => kb.push_char('\x11'), // Up
            0x50 => kb.push_char('\x12'), // Down
            0x4B => kb.push_char('\x13'), // Left
            0x4D => kb.push_char('\x14'), // Right
            _ => {}
        }
        return;
    }

    // Key release (break code)
    if (scancode & 0x80) != 0 {
        let release = scancode & 0x7F;
        match release {
            0x2A | 0x36 => kb.shift_pressed = false,
            0x1D => kb.ctrl_pressed = false,
            _ => {}
        }
        return;
    }

    // Key press (make code)
    match scancode {
        0x2A | 0x36 => {
            kb.shift_pressed = true;
        }
        0x1D => {
            kb.ctrl_pressed = true;
        }
        0x3A => {
            kb.caps_lock = !kb.caps_lock;
        }
        0x3B => { // F1 - Terminal
            crate::desktop::DESKTOP.lock().active_app = crate::desktop::ActiveApp::Terminal;
        }
        0x3C => { // F2 - File Manager
            crate::desktop::DESKTOP.lock().active_app = crate::desktop::ActiveApp::FileManager;
        }
        0x3D => { // F3 - Software Store
            crate::desktop::DESKTOP.lock().active_app = crate::desktop::ActiveApp::SoftwareStore;
        }
        0x3E => { // F4 - System Monitor
            crate::desktop::DESKTOP.lock().active_app = crate::desktop::ActiveApp::SystemMonitor;
        }
        0x3F => { // F5 - Browser
            crate::desktop::DESKTOP.lock().active_app = crate::desktop::ActiveApp::Browser;
        }
        0x40 => { // F6 - Games
            crate::desktop::DESKTOP.lock().active_app = crate::desktop::ActiveApp::Games;
        }
        0x41 => { // F7 - Calculator
            crate::desktop::DESKTOP.lock().active_app = crate::desktop::ActiveApp::Calculator;
        }
        0x42 => { // F8 - Network Manager
            crate::desktop::DESKTOP.lock().active_app = crate::desktop::ActiveApp::NetworkManager;
        }
        0x43 => { // F9 - Settings
            crate::desktop::DESKTOP.lock().active_app = crate::desktop::ActiveApp::Settings;
        }
        0x44 => { // F10 - About/Info
            crate::desktop::DESKTOP.lock().active_app = crate::desktop::ActiveApp::About;
        }
        0x57 => { // F11 - Desktop (minimize all / show desktop)
            crate::desktop::DESKTOP.lock().active_app = crate::desktop::ActiveApp::DesktopOnly;
        }
        code if (code as usize) < SCANCODE_LOWER.len() => {
            let use_upper = kb.shift_pressed ^ kb.caps_lock;
            let ch = if use_upper {
                SCANCODE_UPPER[code as usize]
            } else {
                SCANCODE_LOWER[code as usize]
            };
            if ch != '\0' {
                kb.push_char(ch);
            }
        }
        _ => {}
    }
}

pub fn pop_key() -> Option<char> {
    KEYBOARD.lock().pop_char()
}
