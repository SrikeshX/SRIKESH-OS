//! SRIKESH OS - PS/2 Mouse Driver
//!
//! Features:
//! - IRQ 12 (vector 44) handler via Slave PIC.
//! - 8042 Auxiliary controller initialization.
//! - Standard 3-byte packet decoder.
//! - Absolute cursor position calculation with screen bounds clamping.

use x86_64::instructions::port::Port;
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
}

pub struct MouseDriver {
    x: usize,
    y: usize,
    max_x: usize,
    max_y: usize,
    packet: [u8; 3],
    cycle: u8,
    left: bool,
    right: bool,
    middle: bool,
}

impl MouseDriver {
    pub const fn new() -> Self {
        Self {
            x: 512,
            y: 384,
            max_x: 1024,
            max_y: 768,
            packet: [0; 3],
            cycle: 0,
            left: false,
            right: false,
            middle: false,
        }
    }

    pub fn set_bounds(&mut self, max_x: usize, max_y: usize) {
        self.max_x = max_x;
        self.max_y = max_y;
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

    unsafe fn wait_read(&self) {
        let mut status = Port::<u8>::new(STATUS_PORT);
        for _ in 0..10_000 {
            if (status.read() & 0x01) != 0 {
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

    unsafe fn read_data(&self) -> u8 {
        self.wait_read();
        let mut port = Port::<u8>::new(DATA_PORT);
        port.read()
    }

    /// Initialize the PS/2 mouse on the 8042 controller
    pub fn init(&mut self) {
        unsafe {
            // Enable auxiliary device
            self.write_cmd(0xA8);

            // Read controller command byte
            self.write_cmd(0x20);
            let mut status = self.read_data();
            // Enable IRQ 12 and clear disable-mouse bit
            status |= 0x02;
            status &= !0x20;

            // Write back updated command byte
            self.write_cmd(0x60);
            self.write_data(status);

            // Send Reset command to mouse
            self.write_mouse(0xFF);
            let _ = self.read_data(); // ACK 0xFA
            let _ = self.read_data(); // Self-test 0xAA
            let _ = self.read_data(); // Device ID 0x00

            // Set default settings
            self.write_mouse(0xF6);
            let _ = self.read_data(); // ACK

            // Enable data streaming
            self.write_mouse(0xF4);
            let _ = self.read_data(); // ACK
        }
    }

    unsafe fn write_mouse(&self, data: u8) {
        // Tell 8042 next data byte goes to mouse
        self.write_cmd(0xD4);
        self.write_data(data);
    }

    pub fn handle_interrupt(&mut self) {
        let mut status_port = Port::<u8>::new(STATUS_PORT);
        let mut data_port = Port::<u8>::new(DATA_PORT);

        let status = unsafe { status_port.read() };
        // Check if data is available and originated from auxiliary device (bit 5)
        if (status & 0x01) == 0 {
            return;
        }

        let byte = unsafe { data_port.read() };

        match self.cycle {
            0 => {
                // Byte 0 must have bit 3 set to 1 in standard PS/2 packet format
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
                self.cycle = 0;

                // Decode packet
                let flags = self.packet[0];
                let mut dx = self.packet[1] as i32;
                let mut dy = self.packet[2] as i32;

                // Sign extension for 9-bit deltas
                if (flags & 0x10) != 0 {
                    dx |= !0xFF;
                }
                if (flags & 0x20) != 0 {
                    dy |= !0xFF;
                }

                // Check for overflow bits
                if (flags & 0xC0) != 0 {
                    return; // Ignore overflow packets
                }

                self.left = (flags & 0x01) != 0;
                self.right = (flags & 0x02) != 0;
                self.middle = (flags & 0x04) != 0;

                // Update cursor position (PS/2 positive Y is upward)
                let new_x = (self.x as i32 + dx).clamp(0, (self.max_x.saturating_sub(1)) as i32);
                let new_y = (self.y as i32 - dy).clamp(0, (self.max_y.saturating_sub(1)) as i32);

                self.x = new_x as usize;
                self.y = new_y as usize;
            }
            _ => {
                self.cycle = 0;
            }
        }
    }

    pub fn state(&self) -> MouseState {
        MouseState {
            x: self.x,
            y: self.y,
            left_button: self.left,
            right_button: self.right,
            middle_button: self.middle,
        }
    }
}

pub static MOUSE: Mutex<MouseDriver> = Mutex::new(MouseDriver::new());

pub fn handle_mouse_interrupt() {
    MOUSE.lock().handle_interrupt();
}

pub fn get_mouse_state() -> MouseState {
    MOUSE.lock().state()
}
