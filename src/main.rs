use rusb::{DeviceHandle, GlobalContext, UsbContext};
use std::{thread, time::Duration};

const GOOGLE_VENDOR_ID: u16 = 0x18d1;
const FASTBOOT_PRODUCT_IDS: [u16; 4] = [0x4ee0, 0xd00d, 0x0d02, 0x4ee1]; // je nach Gerät unterschiedlich

const TIMEOUT: Duration = Duration::from_millis(15);

fn find_fastboot_device_by_serial(serial: &str) -> Option<DeviceHandle<GlobalContext>> {
    let devices = rusb::devices().ok()?;

    for device in devices.iter() {
        let desc = device.device_descriptor().ok()?;

        if desc.vendor_id() == GOOGLE_VENDOR_ID && FASTBOOT_PRODUCT_IDS.contains(&desc.product_id()) {
            let handle = device.open().ok()?;

            if let Ok(serial_number) = handle.read_serial_number_string_ascii(&desc) {
                if serial_number == serial {
                    println!("✅ Found device with serial: {}", serial);
                    if handle.claim_interface(0).is_ok() {
                        return Some(handle);
                    } else {
                        eprintln!("⚠️  Could not claim interface 0");
                        return None;
                    }
                } else {
                    println!("⚠️  Skipping device with serial: {}", serial_number);
                }
            }
        }
    }

    None
}



fn send_command<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
    cmd: &str,
) -> rusb::Result<String> {
    let out_endpoint = 0x01; // USB OUT endpoint
    let in_endpoint = 0x81;  // USB IN endpoint

    println!("\n=> Sending command: {}", cmd);
    handle.write_bulk(out_endpoint, cmd.as_bytes(), TIMEOUT)?;

    let mut buf = [0u8; 512];
    let len = handle.read_bulk(in_endpoint, &mut buf, TIMEOUT)?;
    let response = String::from_utf8_lossy(&buf[..len]).to_string();
    println!("<= Received response: {}", response);

    Ok(response)
}

fn main() {
    // Deine Ziel-Seriennummer – aus fastboot devices bekannt
    let target_serial = "FUH7N16B16009379";

    match find_fastboot_device_by_serial(target_serial) {
        Some(mut device) => {
            println!("Connected to Fastboot device with serial {}.", target_serial);

			

            for i in 1..=9000000000000000u64 {
				let cmd = format!("getvar {:0>16}", i);
                match send_command(&mut device, &cmd) {
                    Ok(response) => {
                        if response.starts_with("FAIL") {
                            eprintln!("Command failed: {}", response);
                        } else if response.starts_with("OKAY") {
                            println!("Command succeeded.");
                        } else {
                            println!("Unknown response: {}", response);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error sending command '{}': {}", cmd, e);
                        break;
                    }
                }

            }
        }
        None => {
            eprintln!("❌ No Fastboot device with serial {} found.", target_serial);
        }
    }
}

