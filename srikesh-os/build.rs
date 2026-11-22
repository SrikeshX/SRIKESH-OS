use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());

    let kernel = PathBuf::from(
        std::env::var_os("CARGO_BIN_FILE_KERNEL_kernel").unwrap()
    );

    let mut boot_cfg = bootloader::BootConfig::default();
    boot_cfg.frame_buffer_logging = false;
    boot_cfg.serial_logging = true;

    let uefi_path = out_dir.join("uefi.img");
    let mut uefi_boot = bootloader::UefiBoot::new(&kernel);
    uefi_boot.set_boot_config(&boot_cfg);
    uefi_boot.create_disk_image(&uefi_path).unwrap();

    let bios_path = out_dir.join("bios.img");
    let mut bios_boot = bootloader::BiosBoot::new(&kernel);
    bios_boot.set_boot_config(&boot_cfg);
    bios_boot.create_disk_image(&bios_path).unwrap();

    println!("cargo:rustc-env=UEFI_PATH={}", uefi_path.display());
    println!("cargo:rustc-env=BIOS_PATH={}", bios_path.display());
}
