//! SRIKESH OS - PS/2 & Legacy-USB Mouse Driver (IRQ 12 + Polling Hybrid)
//!
//! Features:
//! - IRQ 12 (vector 44) handler via Slave PIC + 8042 polling fallback.
//! - Lock-safe design: all non-ISR accesses run inside `without_interrupts`.
//! - Full 8042 controller initialization enabling both Port 1 (KB) & Port 2 (Mouse).
//! - Standard 3-byte AND IntelliMouse 4-byte (scroll wheel) packet decoder.
//! - Smooth cursor position tracking with screen bounds clamping.

use x86_64::instructions::port::Port;
use x86_64::instructions::interrupts::without_interrupts;
use spin::Mutex;

const DATA_PORT: u16 = 0x60;
const STATUS_PORT: u16 = 0x64;

#[derive(Debug, Clone, Copy)]
pub struct MouseState {
    pub x: usize,
    pub y: usize,
    pub left_button: bool,
    pub right_button: bool,
    pub middle_button: bool,
    pub scroll_delta: i32,
}

pub struct MouseDriver {
    x: usize,
    y: usize,
    max_x: usize,
    max_y: usize,
    packet: [u8; 4],
    cycle: u8,
    has_scroll_wheel: bool,
    left: bool,
    right: bool,
    middle: bool,
    scroll_accum: i32,
    pub initialized: bool,
}

impl MouseDriver {
    pub const fn new() -> Self {
        Self {
            x: 512,
            y: 384,
            max_x: 1024,
            max_y: 768,
            packet: [0; 4],
            cycle: 0,
            has_scroll_wheel: false,
            left: false,
            right: false,
            middle: false,
            scroll_accum: 0,
            initialized: false,
        }
    }

    pub fn set_bounds(&mut self, max_x: usize, max_y: usize) {
        self.max_x = max_x.max(1);
        self.max_y = max_y.max(1);
        self.x = max_x / 2;
        self.y = max_y / 2;
    }

    unsafe fn wait_write(&self) {
        let mut status = Port::<u8>::new(STATUS_PORT);
        for _ in 0..10_000 {
            if (status.read() & 0x02) == 0 {
                return;
            }
        }
    }

    unsafe fn write_cmd(&self, cmd: u8) {
        self.wait_write();
        let mut port = Port::<u8>::new(STATUS_PORT);
        port.write(cmd);
    }

    unsafe fn write_data(&self, data: u8) {
        self.wait_write();
        let mut port = Port::<u8>::new(DATA_PORT);
        port.write(data);
    }

    unsafe fn read_data_timeout(&self, timeout: u32) -> Option<u8> {
        let mut status = Port::<u8>::new(STATUS_PORT);
        let mut data = Port::<u8>::new(DATA_PORT);
        for _ in 0..timeout {
            if (status.read() & 0x01) != 0 {
                return Some(data.read());
            }
        }
        None
    }

    unsafe fn drain_output_buffer(&self) {
        let mut status = Port::<u8>::new(STATUS_PORT);
        let mut data = Port::<u8>::new(DATA_PORT);
        for _ in 0..64 {
            if (status.read() & 0x01) != 0 {
                let _ = data.read();
            } else {
                break;
            }
        }
    }

    unsafe fn write_mouse(&self, data: u8) {
        self.write_cmd(0xD4);
        self.write_data(data);
    }

    /// Initialize 8042 PS/2 keyboard + mouse ports safely with timeouts.
    pub fn init(&mut self) {
        unsafe {
            self.drain_output_buffer();

            // Enable both Keyboard (0xAE) and Auxiliary Mouse (0xA8) ports
            self.write_cmd(0xAE);
            self.write_cmd(0xA8);

            // Read 8042 Controller Command Byte
            self.write_cmd(0x20);
            let mut ccb = self.read_data_timeout(3000).unwrap_or(0x47);
            // Enable IRQ 1 (bit 0) and IRQ 12 (bit 1)
            ccb |= 0x03;
            // Clear Keyboard Clock Disable (bit 4) and Mouse Clock Disable (bit 5)
            ccb &= !0x30;
            self.write_cmd(0x60);
            self.write_data(ccb);

            // Reset mouse
            self.write_mouse(0xFF);
            let ack1 = self.read_data_timeout(8000);
            let _sts = self.read_data_timeout(8000);
            let _id = self.read_data_timeout(3000);

            // Set defaults
            self.write_mouse(0xF6);
            let _ = self.read_data_timeout(3000);

            // Probe IntelliMouse scroll wheel magic sequence: rate 200, 100, 80
            for &rate in &[200u8, 100, 80] {
                self.write_mouse(0xF3);
                let _ = self.read_data_timeout(2000);
                self.write_mouse(rate);
                let _ = self.read_data_timeout(2000);
            }
            // Ask for device ID (0xF2)
            self.write_mouse(0xF2);
            let _ = self.read_data_timeout(2000); // ACK
            if let Some(dev_id) = self.read_data_timeout(3000) {
                if dev_id == 3 || dev_id == 4 {
                    self.has_scroll_wheel = true;
                }
            }

            // Enable data reporting (streaming)
            self.write_mouse(0xF4);
            let ack_enable = self.read_data_timeout(3000);

            self.initialized = ack1 == Some(0xFA) || ack_enable == Some(0xFA);
            self.cycle = 0;

            // Drain any remaining ACK bytes so IRQs fire cleanly
            self.drain_output_buffer();
        }
    }

    pub fn process_byte(&mut self, byte: u8) {
        let packet_len: u8 = if self.has_scroll_wheel { 4 } else { 3 };

        match self.cycle {
            0 => {
                // Standard PS/2 byte 0 always has bit 3 == 1
                if (byte & 0x08) != 0 {
                    self.packet[0] = byte;
                    self.cycle = 1;
                }
            }
            1 => {
                self.packet[1] = byte;
                self.cycle = 2;
            }
            2 => {
                self.packet[2] = byte;
                if packet_len == 3 {
                    self.cycle = 0;
                    self.decode_packet(false);
                } else {
                    self.cycle = 3;
                }
            }
            3 => {
                self.packet[3] = byte;
                self.cycle = 0;
                self.decode_packet(true);
            }
            _ => {
                self.cycle = 0;
            }
        }
    }

    fn decode_packet(&mut self, has_z: bool) {
        let flags = self.packet[0];
        // Ignore X/Y overflow packets
        if (flags & 0xC0) != 0 {
            return;
        }

        let mut dx = self.packet[1] as i32;
        let mut dy = self.packet[2] as i32;

        if (flags & 0x10) != 0 {
            dx |= !0xFF;
        }
        if (flags & 0x20) != 0 {
            dy |= !0xFF;
        }

        self.left = (flags & 0x01) != 0;
        self.right = (flags & 0x02) != 0;
        self.middle = (flags & 0x04) != 0;

        if has_z {
            let z = self.packet[3] as i8;
            self.scroll_accum = self.scroll_accum.saturating_add(z as i32);
        }

        // PS/2 positive Y is upward
        let new_x = (self.x as i32 + dx).clamp(0, self.max_x.saturating_sub(1) as i32);
        let new_y = (self.y as i32 - dy).clamp(0, self.max_y.saturating_sub(1) as i32);

        self.x = new_x as usize;
        self.y = new_y as usize;
    }

    pub fn handle_interrupt(&mut self) {
        let mut status_port = Port::<u8>::new(STATUS_PORT);
        let mut data_port = Port::<u8>::new(DATA_PORT);

        let status = unsafe { status_port.read() };
        if (status & 0x01) == 0 {
            return;
        }
        let byte = unsafe { data_port.read() };
        // If bit 5 is NOT set, this byte is actually from the keyboard port
        if (status & 0x20) == 0 {
            crate::keyboard::process_scancode(byte);
            return;
        }
        self.process_byte(byte);
    }

    pub fn state(&self) -> MouseState {
        MouseState {
            x: self.x,
            y: self.y,
            left_button: self.left,
            right_button: self.right,
            middle_button: self.middle,
            scroll_delta: self.scroll_accum,
        }
    }
}

pub static MOUSE: Mutex<MouseDriver> = Mutex::new(MouseDriver::new());

/// Called inside IRQ 12 handler (interrupts already disabled)
pub fn handle_mouse_interrupt() {
    MOUSE.lock().handle_interrupt();
}

/// Called from keyboard ISR if an aux byte arrived on IRQ 1
pub fn process_mouse_byte(byte: u8) {
    MOUSE.lock().process_byte(byte);
}

/// Safe non-ISR mouse state read (disables interrupts to prevent deadlock)
pub fn get_mouse_state() -> MouseState {
    without_interrupts(|| MOUSE.lock().state())
}

/// Consume accumulated scroll wheel delta
pub fn take_scroll_delta() -> i32 {
    without_interrupts(|| {
        let mut m = MOUSE.lock();
        let d = m.scroll_accum;
        m.scroll_accum = 0;
        d
    })
}

/// Safe non-ISR bounds setup
pub fn set_mouse_bounds(max_x: usize, max_y: usize) {
    without_interrupts(|| MOUSE.lock().set_bounds(max_x, max_y));
}

/// Safe non-ISR init
pub fn init_mouse() {
    without_interrupts(|| MOUSE.lock().init());
}

/// Hybrid polling fallback for 8042 PS/2 & Legacy-USB HID controller.
/// Guarantees keyboard & mouse input even if UEFI/APIC suppresses legacy IRQ 1/12.
pub fn poll_8042() {
    without_interrupts(|| {
        let mut status_port = Port::<u8>::new(STATUS_PORT);
        let mut data_port = Port::<u8>::new(DATA_PORT);

        for _ in 0..16 {
            let status = unsafe { status_port.read() };
            if (status & 0x01) == 0 {
                break;
            }
            let byte = unsafe { data_port.read() };
            if (status & 0x20) != 0 {
                MOUSE.lock().process_byte(byte);
            } else {
                crate::keyboard::process_scancode(byte);
            }
        }
    });
}
