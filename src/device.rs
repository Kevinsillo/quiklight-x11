use anyhow::{bail, Result};
use std::fs;
use std::path::PathBuf;

const VENDOR_ID: u32 = 0x1A86;
const PRODUCT_ID: u32 = 0xFE07;

fn find_hidraw() -> Result<PathBuf> {
    let target = format!("0003:{:08X}:{:08X}", VENDOR_ID, PRODUCT_ID);
    let mut candidates: Vec<PathBuf> = Vec::new();

    for entry in fs::read_dir("/sys/bus/hid/devices")? {
        let dev = entry?.path();
        let Ok(content) = fs::read_to_string(dev.join("uevent")) else {
            continue;
        };
        if !content.to_uppercase().contains(&target) {
            continue;
        }

        if let Ok(iter) = fs::read_dir(dev.join("hidraw")) {
            for he in iter.flatten() {
                let name = he.file_name().to_string_lossy().to_string();
                if name.starts_with("hidraw") {
                    candidates.push(PathBuf::from(format!("/dev/{}", name)));
                }
            }
        }
    }

    if candidates.is_empty() {
        bail!(
            "Quiklight device not found (VID=0x{:04X}, PID=0x{:04X}).\n\
            Check that the USB cable is plugged in and the udev rule is installed.",
            VENDOR_ID,
            PRODUCT_ID
        );
    }

    candidates.sort();
    Ok(candidates.remove(0))
}

pub fn open_device() -> Result<fs::File> {
    let path = find_hidraw()?;
    eprintln!("Device: {}", path.display());
    Ok(fs::OpenOptions::new().read(true).write(true).open(&path)?)
}
