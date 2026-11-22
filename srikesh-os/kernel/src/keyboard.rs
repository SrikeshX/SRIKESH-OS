//! SRIKESH OS - Complete PS/2 & Legacy-USB Keyboard Driver (IRQ 1 + Polling Hybrid)
//!
//! Features:
//! - Complete Scancode Set 1 + Extended (0xE0) decoding.
//! - All A-Z, 0-9, shifted symbols, punctuation, Enter, Escape, Backspace, Delete,
//!   Insert, Tab, Home, End, PageUp, PageDown, Arrow keys, F1-F12, and Windows/Super key.
//! - Full modifier state tracking: Shift (L/R), Ctrl (L/R), Alt (L/R), Super, Caps/Num/Scroll Lock.
//! - Lock-safe ring buffer (`without_interrupts`) — zero ISR spinlock deadlocks.

use x86_64::instructions::port::Port;
use x86_64::instructions::interrupts::without_interrupts;
use spin::Mutex;

const DATA_PORT: u16 = 0x60;
const STATUS_PORT: u16 = 0x64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Enter,
    Escape,
    Backspace,
    Tab,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    Up,
    Down,
    Left,
    Right,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Super,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    pub key: Key,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub super_key: bool,
}

struct KeyboardState {
    lshift: bool,
    rshift: bool,
    lctrl: bool,
    rctrl: bool,
    lalt: bool,
    ralt: bool,
    super_down: bool,
    caps_lock: bool,
    num_lock: bool,
    scroll_lock: bool,
    extended_scancode: bool,
    buffer: [Option<KeyEvent>; 128],
    head: usize,
    tail: usize,
}

impl KeyboardState {
    const fn new() -> Self {
        Self {
            lshift: false,
            rshift: false,
            lctrl: false,
            rctrl: false,
            lalt: false,
            ralt: false,
            super_down: false,
            caps_lock: false,
            num_lock: true,
            scroll_lock: false,
            extended_scancode: false,
            buffer: [None; 128],
            head: 0,
            tail: 0,
        }
    }

    fn shift(&self) -> bool { self.lshift || self.rshift }
    fn ctrl(&self) -> bool { self.lctrl || self.rctrl }
    fn alt(&self) -> bool { self.lalt || self.ralt }

    fn push_key(&mut self, key: Key) {
        let ev = KeyEvent {
            key,
            shift: self.shift(),
            ctrl: self.ctrl(),
            alt: self.alt(),
            super_key: self.super_down,
        };
        let next_head = (self.head + 1) % self.buffer.len();
        if next_head != self.tail {
            self.buffer[self.head] = Some(ev);
            self.head = next_head;
        }
    }

    fn pop_event(&mut self) -> Option<KeyEvent> {
        if self.head == self.tail {
            None
        } else {
            let ev = self.buffer[self.tail];
            self.tail = (self.tail + 1) % self.buffer.len();
            ev
        }
    }
}

static KEYBOARD: Mutex<KeyboardState> = Mutex::new(KeyboardState::new());

/// Scan Code Set 1 lookup table (unshifted)
const SCANCODE_LOWER: [char; 58] = [
    '\0', '\0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', '\0',
    '\0', 'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']', '\0',
    '\0', 'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';', '\'', '`',
    '\0', '\\', 'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/', '\0',
    '*', '\0', ' ',
];

/// Scan Code Set 1 lookup table (shifted)
const SCANCODE_UPPER: [char; 58] = [
    '\0', '\0', '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '_', '+', '\0',
    '\0', 'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', '{', '}', '\0',
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
    if scancode == 0xE1 {
        return; // Ignore Pause/Break prefix
    }

    if kb.extended_scancode {
        kb.extended_scancode = false;
        let released = (scancode & 0x80) != 0;
        let code = scancode & 0x7F;
        if released {
            match code {
                0x1D => kb.rctrl = false,
                0x38 => kb.ralt = false,
                0x5B | 0x5C => kb.super_down = false,
                _ => {}
            }
            return;
        }
        match code {
            0x1D => kb.rctrl = true,
            0x38 => kb.ralt = true,
            0x5B | 0x5C => {
                kb.super_down = true;
                kb.push_key(Key::Super);
            }
            0x1C => kb.push_key(Key::Enter),
            0x35 => kb.push_key(Key::Char('/')),
            0x47 => kb.push_key(Key::Home),
            0x48 => kb.push_key(Key::Up),
            0x49 => kb.push_key(Key::PageUp),
            0x4B => kb.push_key(Key::Left),
            0x4D => kb.push_key(Key::Right),
            0x4F => kb.push_key(Key::End),
            0x50 => kb.push_key(Key::Down),
            0x51 => kb.push_key(Key::PageDown),
            0x52 => kb.push_key(Key::Insert),
            0x53 => kb.push_key(Key::Delete),
            _ => {}
        }
        return;
    }

    // Key release (break code)
    if (scancode & 0x80) != 0 {
        let release = scancode & 0x7F;
        match release {
            0x2A => kb.lshift = false,
            0x36 => kb.rshift = false,
            0x1D => kb.lctrl = false,
            0x38 => kb.lalt = false,
            _ => {}
        }
        return;
    }

    // Key press (make code)
    let num_lock = kb.num_lock;
    match scancode {
        0x01 => kb.push_key(Key::Escape),
        0x0E => kb.push_key(Key::Backspace),
        0x0F => kb.push_key(Key::Tab),
        0x1C => kb.push_key(Key::Enter),
        0x1D => kb.lctrl = true,
        0x2A => kb.lshift = true,
        0x36 => kb.rshift = true,
        0x38 => kb.lalt = true,
        0x3A => kb.caps_lock = !kb.caps_lock,
        0x45 => kb.num_lock = !kb.num_lock,
        0x46 => kb.scroll_lock = !kb.scroll_lock,
        0x3B => kb.push_key(Key::F1),
        0x3C => kb.push_key(Key::F2),
        0x3D => kb.push_key(Key::F3),
        0x3E => kb.push_key(Key::F4),
        0x3F => kb.push_key(Key::F5),
        0x40 => kb.push_key(Key::F6),
        0x41 => kb.push_key(Key::F7),
        0x42 => kb.push_key(Key::F8),
        0x43 => kb.push_key(Key::F9),
        0x44 => kb.push_key(Key::F10),
        0x57 => kb.push_key(Key::F11),
        0x58 => kb.push_key(Key::F12),
        // Numpad keys (when NumLock is on -> digits; else navigation)
        0x47 => kb.push_key(if num_lock { Key::Char('7') } else { Key::Home }),
        0x48 => kb.push_key(if num_lock { Key::Char('8') } else { Key::Up }),
        0x49 => kb.push_key(if num_lock { Key::Char('9') } else { Key::PageUp }),
        0x4A => kb.push_key(Key::Char('-')),
        0x4B => kb.push_key(if num_lock { Key::Char('4') } else { Key::Left }),
        0x4C => { if num_lock { kb.push_key(Key::Char('5')); } }
        0x4D => kb.push_key(if num_lock { Key::Char('6') } else { Key::Right }),
        0x4E => kb.push_key(Key::Char('+')),
        0x4F => kb.push_key(if num_lock { Key::Char('1') } else { Key::End }),
        0x50 => kb.push_key(if num_lock { Key::Char('2') } else { Key::Down }),
        0x51 => kb.push_key(if num_lock { Key::Char('3') } else { Key::PageDown }),
        0x52 => kb.push_key(if num_lock { Key::Char('0') } else { Key::Insert }),
        0x53 => kb.push_key(if num_lock { Key::Char('.') } else { Key::Delete }),
        code if (code as usize) < SCANCODE_LOWER.len() => {
            let base = SCANCODE_LOWER[code as usize];
            if base != '\0' {
                let is_letter = base.is_ascii_alphabetic();
                let use_upper = if is_letter {
                    kb.shift() ^ kb.caps_lock
                } else {
                    kb.shift()
                };
                let ch = if use_upper {
                    SCANCODE_UPPER[code as usize]
                } else {
                    base
                };
                if ch != '\0' {
                    kb.push_key(Key::Char(ch));
                }
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
    if (status & 0x20) != 0 {
        let byte = unsafe { data_port.read() };
        crate::mouse::process_mouse_byte(byte);
        return;
    }

    let scancode = unsafe { data_port.read() };
    process_scancode(scancode);
}

/// Safe non-ISR rich key event pop
pub fn pop_key_event() -> Option<KeyEvent> {
    without_interrupts(|| KEYBOARD.lock().pop_event())
}

/// Flush all buffered key events (call after boot to discard spurious scancodes)
pub fn flush_key_buffer() {
    without_interrupts(|| {
        let mut kb = KEYBOARD.lock();
        kb.head = kb.tail;
    });
}
