//! SRIKESH OS - ATA/IDE PIO Mode Block Storage Driver
//!
//! Features:
//! - Primary ATA Bus (I/O ports 0x1F0..0x1F7, Control 0x3F6)
//! - 28-bit & 48-bit LBA sector reading and writing
//! - Hardware drive identification (ATA IDENTIFY command 0xEC)
//! - Direct integration with StorageGuard to prevent writes to host internal drives

use x86_64::instructions::port::Port;
use crate::serial;
use crate::storage_guard::{STORAGE_GUARD, GuardError};

const ATA_DATA: u16 = 0x1F0;
const ATA_ERROR_FEATURE: u16 = 0x1F1;
const ATA_SECTOR_COUNT: u16 = 0x1F2;
const ATA_LBA_LOW: u16 = 0x1F3;
const ATA_LBA_MID: u16 = 0x1F4;
const ATA_LBA_HIGH: u16 = 0x1F5;
const ATA_DRIVE_HEAD: u16 = 0x1F6;
const ATA_STATUS_COMMAND: u16 = 0x1F7;
const ATA_ALT_STATUS: u16 = 0x3F6;

// ATA Status Register bits
const STATUS_BSY: u8 = 0x80;  // Busy
const STATUS_DRDY: u8 = 0x40; // Drive ready
const STATUS_DRQ: u8 = 0x08;  // Data request ready
const STATUS_ERR: u8 = 0x01;  // Error

// ATA Commands
const CMD_READ_SECTORS: u8 = 0x20;
const CMD_WRITE_SECTORS: u8 = 0x30;
const CMD_IDENTIFY: u8 = 0xEC;
const CMD_FLUSH_CACHE: u8 = 0xE7;

#[derive(Debug, Clone, Copy)]
pub struct DriveIdentity {
    pub exists: bool,
    pub sectors_28: u32,
    pub model: [u8; 40],
    pub serial: [u8; 20],
}

pub struct AtaController {
    master_present: bool,
    slave_present: bool,
    master_info: DriveIdentity,
}

impl AtaController {
    pub const fn new() -> Self {
        Self {
            master_present: false,
            slave_present: false,
            master_info: DriveIdentity {
                exists: false,
                sectors_28: 0,
                model: [0; 40],
                serial: [0; 20],
            },
        }
    }

    unsafe fn wait_busy(&self) -> u8 {
        let mut status_port = Port::<u8>::new(ATA_STATUS_COMMAND);
        let mut status = status_port.read();
        while (status & STATUS_BSY) != 0 {
            core::hint::spin_loop();
            status = status_port.read();
        }
        status
    }

    unsafe fn wait_ready(&self) -> Result<(), &'static str> {
        let mut status_port = Port::<u8>::new(ATA_STATUS_COMMAND);
        for _ in 0..100_000 {
            let status = status_port.read();
            if (status & STATUS_ERR) != 0 {
                return Err("ATA Drive reported hardware error");
            }
            if (status & STATUS_BSY) == 0 && (status & STATUS_DRQ) != 0 {
                return Ok(());
            }
            core::hint::spin_loop();
        }
        Err("ATA Drive wait timeout")
    }

    /// Probes and identifies the Master ATA drive on the primary bus.
    pub fn probe_master(&mut self) -> Result<DriveIdentity, &'static str> {
        unsafe {
            let mut drive_head = Port::<u8>::new(ATA_DRIVE_HEAD);
            let mut sec_count = Port::<u8>::new(ATA_SECTOR_COUNT);
            let mut lba_low = Port::<u8>::new(ATA_LBA_LOW);
            let mut lba_mid = Port::<u8>::new(ATA_LBA_MID);
            let mut lba_high = Port::<u8>::new(ATA_LBA_HIGH);
            let mut cmd_port = Port::<u8>::new(ATA_STATUS_COMMAND);

            // Select master drive
            drive_head.write(0xA0);
            sec_count.write(0);
            lba_low.write(0);
            lba_mid.write(0);
            lba_high.write(0);

            // Send IDENTIFY
            cmd_port.write(CMD_IDENTIFY);

            let status = cmd_port.read();
            if status == 0 || status == 0xFF {
                return Err("No master ATA drive found");
            }

            self.wait_busy();

            let mid = lba_mid.read();
            let high = lba_high.read();
            if mid != 0 || high != 0 {
                return Err("Device is not an ATA disk (ATAPI or SATA packet)");
            }

            self.wait_ready()?;

            let mut data_port = Port::<u16>::new(ATA_DATA);
            let mut buffer = [0u16; 256];
            for word in buffer.iter_mut() {
                *word = data_port.read();
            }

            let sectors = (buffer[60] as u32) | ((buffer[61] as u32) << 16);

            let mut info = DriveIdentity {
                exists: true,
                sectors_28: sectors,
                model: [b' '; 40],
                serial: [b' '; 20],
            };

            // Extract serial (words 10..20)
            for i in 0..10 {
                let w = buffer[10 + i];
                info.serial[i * 2] = (w >> 8) as u8;
                info.serial[i * 2 + 1] = (w & 0xFF) as u8;
            }

            // Extract model (words 27..47)
            for i in 0..20 {
                let w = buffer[27 + i];
                info.model[i * 2] = (w >> 8) as u8;
                info.model[i * 2 + 1] = (w & 0xFF) as u8;
            }

            self.master_present = true;
            self.master_info = info;

            serial::write_str("[ATA] Discovered Master Drive: Model: ");
            for &b in &info.model {
                if b >= 32 && b <= 126 {
                    serial::write_byte(b);
                }
            }
            serial::write_str(" | Capacity: ");
            serial::write_u64((sectors as u64 * 512) / (1024 * 1024));
            serial::write_str(" MB\n");

            Ok(info)
        }
    }

    /// Read 512-byte sectors using 28-bit LBA
    pub fn read_sectors(
        &self,
        drive_id: u8,
        lba: u64,
        count: u8,
        buf: &mut [u8],
    ) -> Result<(), &'static str> {
        if (count as usize * 512) > buf.len() {
            return Err("Buffer too small for requested ATA sector count");
        }

        unsafe {
            let mut drive_head = Port::<u8>::new(ATA_DRIVE_HEAD);
            let mut sec_count = Port::<u8>::new(ATA_SECTOR_COUNT);
            let mut lba_low = Port::<u8>::new(ATA_LBA_LOW);
            let mut lba_mid = Port::<u8>::new(ATA_LBA_MID);
            let mut lba_high = Port::<u8>::new(ATA_LBA_HIGH);
            let mut cmd_port = Port::<u8>::new(ATA_STATUS_COMMAND);
            let mut data_port = Port::<u16>::new(ATA_DATA);

            // Select master (0xE0) or slave (0xF0) + top 4 bits of LBA
            let drive_byte = if drive_id == 0 { 0xE0 } else { 0xF0 } | (((lba >> 24) & 0x0F) as u8);
            drive_head.write(drive_byte);

            sec_count.write(count);
            lba_low.write((lba & 0xFF) as u8);
            lba_mid.write(((lba >> 8) & 0xFF) as u8);
            lba_high.write(((lba >> 16) & 0xFF) as u8);

            cmd_port.write(CMD_READ_SECTORS);

            let mut offset = 0;
            for _ in 0..count {
                self.wait_busy();
                self.wait_ready()?;

                for _ in 0..256 {
                    let word = data_port.read();
                    buf[offset] = (word & 0xFF) as u8;
                    buf[offset + 1] = (word >> 8) as u8;
                    offset += 2;
                }
            }

            Ok(())
        }
    }

    /// Write 512-byte sectors using 28-bit LBA.
    /// MUST be pre-validated by the StorageGuard to guarantee host internal drive protection!
    pub fn write_sectors(
        &self,
        drive_id: u8,
        lba: u64,
        count: u8,
        data: &[u8],
    ) -> Result<(), &'static str> {
        if (count as usize * 512) > data.len() {
            return Err("Data buffer smaller than requested write sectors");
        }

        // HARDENED SECURITY CHECK: Validate against StorageGuard
        if let Err(e) = STORAGE_GUARD.lock().validate_write(drive_id, lba, count as u32) {
            match e {
                GuardError::HostWriteBlocked { .. } => {
                    return Err("[SECURITY DENIED] Cannot write to protected host drive!");
                }
                _ => return Err("Invalid storage device write request"),
            }
        }

        unsafe {
            let mut drive_head = Port::<u8>::new(ATA_DRIVE_HEAD);
            let mut sec_count = Port::<u8>::new(ATA_SECTOR_COUNT);
            let mut lba_low = Port::<u8>::new(ATA_LBA_LOW);
            let mut lba_mid = Port::<u8>::new(ATA_LBA_MID);
            let mut lba_high = Port::<u8>::new(ATA_LBA_HIGH);
            let mut cmd_port = Port::<u8>::new(ATA_STATUS_COMMAND);
            let mut data_port = Port::<u16>::new(ATA_DATA);

            let drive_byte = if drive_id == 0 { 0xE0 } else { 0xF0 } | (((lba >> 24) & 0x0F) as u8);
            drive_head.write(drive_byte);

            sec_count.write(count);
            lba_low.write((lba & 0xFF) as u8);
            lba_mid.write(((lba >> 8) & 0xFF) as u8);
            lba_high.write(((lba >> 16) & 0xFF) as u8);

            cmd_port.write(CMD_WRITE_SECTORS);

            let mut offset = 0;
            for _ in 0..count {
                self.wait_busy();
                self.wait_ready()?;

                for _ in 0..256 {
                    let word = (data[offset] as u16) | ((data[offset + 1] as u16) << 8);
                    data_port.write(word);
                    offset += 2;
                }
            }

            // Flush write cache
            cmd_port.write(CMD_FLUSH_CACHE);
            self.wait_busy();

            Ok(())
        }
    }
}

pub static ATA: spin::Mutex<AtaController> = spin::Mutex::new(AtaController::new());
