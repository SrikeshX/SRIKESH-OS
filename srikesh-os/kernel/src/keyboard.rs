//! SRIKESH OS - PS/2 & Legacy-USB Keyboard Driver (IRQ 1 + Polling Hybrid)
//!
//! Features:
//! - IRQ 1 (vector 33) handler + polling fallback for UEFI/APIC compatibility.
//! - Lock-safe design: all non-ISR accesses run inside `without_interrupts`.
//! - Never locks DESKTOP inside an interrupt handler (eliminates spinlock deadlocks).
//! - Scancode Set 1 decoder with Shift, CapsLock, Ctrl, Arrow keys, and F1-F12.

use x86_64::instructions::port::Port;
use x86_64::instructions::interrupts::without_interrupts;
use spin::Mutex;

const DATA_PORT: u16 = 0x60;
const STATUS_PORT: u16 = 0x64;

// Special internal char codes for function keys & navigation
pub const KEY_UP: char = '\x11';
pub const KEY_DOWN: char = '\x12';
pub const KEY_LEFT: char = '\x13';
pub const KEY_RIGHT: char = '\x14';
pub const KEY_F1: char = '\u{0081}';
pub const KEY_F2: char = '\u{0082}';
pub const KEY_F3: char = '\u{0083}';
pub const KEY_F4: char = '\u{0084}';
pub const KEY_F5: char = '\u{0085}';
pub const KEY_F6: char = '\u{0086}';
pub const KEY_F7: char = '\u{0087}';
pub const KEY_F8: char = '\u{0088}';
pub const KEY_F9: char = '\u{0089}';
pub const KEY_F10: char = '\u{008A}';
pub const KEY_F11: char = '\u{008B}';
pub const KEY_F12: char = '\u{008C}';

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

/// Decode a single scancode byte (must be called with interrupts disabled)
pub fn process_scancode(scancode: u8) {
    let mut kb = KEYBOARD.lock();

    if scancode == 0xE0 {
        kb.extended_scancode = true;
        return;
    }

    if kb.extended_scancode {
        kb.extended_scancode = false;
        if (scancode & 0x80) != 0 {
            return;
        }
        match scancode {
            0x48 => kb.push_char(KEY_UP),
            0x50 => kb.push_char(KEY_DOWN),
            0x4B => kb.push_char(KEY_LEFT),
            0x4D => kb.push_char(KEY_RIGHT),
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
        0x2A | 0x36 => kb.shift_pressed = true,
        0x1D => kb.ctrl_pressed = true,
        0x3A => kb.caps_lock = !kb.caps_lock,
        0x3B => kb.push_char(KEY_F1),
        0x3C => kb.push_char(KEY_F2),
        0x3D => kb.push_char(KEY_F3),
        0x3E => kb.push_char(KEY_F4),
        0x3F => kb.push_char(KEY_F5),
        0x40 => kb.push_char(KEY_F6),
        0x41 => kb.push_char(KEY_F7),
        0x42 => kb.push_char(KEY_F8),
        0x43 => kb.push_char(KEY_F9),
        0x44 => kb.push_char(KEY_F10),
        0x57 => kb.push_char(KEY_F11),
        0x58 => kb.push_char(KEY_F12),
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

/// Interrupt handler for IRQ 1 (Keyboard) — runs with interrupts disabled
pub fn handle_keyboard_interrupt() {
    let mut status_port = Port::<u8>::new(STATUS_PORT);
    let mut data_port = Port::<u8>::new(DATA_PORT);

    let status = unsafe { status_port.read() };
    if (status & 0x01) == 0 {
        return;
    }
    // If bit 5 is set, this byte belongs to the auxiliary mouse port!
    if (status & 0x20) != 0 {
        let byte = unsafe { data_port.read() };
        crate::mouse::process_mouse_byte(byte);
        return;
    }

    let scancode = unsafe { data_port.read() };
    process_scancode(scancode);
}

/// Safe non-ISR pop (disables interrupts while holding KEYBOARD lock)
pub fn pop_key() -> Option<char> {
    without_interrupts(|| KEYBOARD.lock().pop_char())
}

/// Flush all buffered key events (call after boot to discard spurious scancodes)
pub fn flush_key_buffer() {
    without_interrupts(|| {
        let mut kb = KEYBOARD.lock();
        kb.head = kb.tail;
    });
}
