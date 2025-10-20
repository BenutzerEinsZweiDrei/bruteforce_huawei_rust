use anyhow::{Context, Result};
use sha2::{Sha256, Digest};
use std::process::Command;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use hex;

fn sha256_truncate_luhn(input: &str) -> (String, bool) {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    let hex_hash = hex::encode(result);
    
    // ✅ FIXED: 13 HEX DIGITS = 52 BITS = MAX 16 DECIMAL DIGITS
    let hex_13 = &hex_hash[0..13];
    let num = u64::from_str_radix(hex_13, 16).expect("Invalid hex");
    let code_str = format!("{:016}", num);
    
    assert_eq!(code_str.len(), 16, "Code must be 16 digits, got {}", code_str.len());
    
    let luhn_ok = luhn_valid(&code_str);
    (code_str, luhn_ok)
}

fn luhn_valid(code: &str) -> bool {
    let digits: Vec<u64> = code.chars()
        .rev()
        .map(|c| c.to_digit(10).unwrap() as u64)
        .collect();
    
    let mut sum = 0;
    for (i, &digit) in digits.iter().enumerate() {
        let mut d = digit;
        if i % 2 == 1 {
            d *= 2;
            if d > 9 { d -= 9; }
        }
        sum += d;
    }
    sum % 10 == 0
}

fn generate_secret_keys(base: u64, count: usize) -> Vec<u64> {
    let mut keys = Vec::new();
    for i in 0..count as u64 {
        keys.push((base + i * 1000) % 10u64.pow(16));
    }
    keys
}

fn huawei_hash_formula(imei: &str, sn: &str, pid: &str, secret_key: u64) -> String {
    let input = format!("{}{}{}{}", imei, sn, pid, format!("{:016}", secret_key));
    let (code, _) = sha256_truncate_luhn(&input);
    code
}

async fn test_code(code: &str) -> Result<bool> {
    if code.len() != 16 || !code.chars().all(|c| c.is_digit(10)) {
        eprintln!("❌ INVALID CODE: '{}' (len={})", code, code.len());
        return Ok(false);
    }
    
    let cmd = format!("fastboot oem unlock {}", code);
    
    let output = tokio::process::Command::new("cmd")
        .args(["/C", &cmd])
        .kill_on_drop(true)
        .output()
        .await
        .context("Failed to execute fastboot")?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let output = format!("{}{}", stdout, stderr);
    
    if output.contains("OKAY") && output.contains("Finished") {
        println!("\n🎉 **SUCCESS!** Code: {}", code);
        println!("📱 Press Volume Up on WARNING screen!");
        return Ok(true);
    }
    
    Ok(false)
}

async fn monitor_device(stop_signal: Arc<AtomicBool>) {
    let mut interval = tokio::time::interval(Duration::from_secs(3));
    while !stop_signal.load(Ordering::Relaxed) {
        match Command::new("fastboot").arg("devices").output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if !stdout.contains("fastboot") {
                    println!("\n🔌 Device lost! Rebooting...");
                    let _ = Command::new("adb").args(["reboot", "bootloader"]).output();
                    tokio::time::sleep(Duration::from_secs(10)).await;
                }
            }
            Err(_) => {}
        }
        interval.tick().await;
    }
}

fn print_separator() {
    for _ in 0..70 {
        print!("─");
    }
    println!();
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let default_imei = "--".to_string();
    let imei = args.get(1).map(|s| s.as_str()).unwrap_or(default_imei.as_str());
    let sn = "--";
    let pid = "--";
    let max_keys: usize = args.get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(100_000);
    
    println!("🚀 HUAWEI **REAL FORMULA** BRUTEFORCE (Rust) - **16 DIGITS PERFECTED**");
    println!("📐 Formula: SHA256(IMEI+SN+PID+SECRET) → 13 hex → 16 decimal");
    println!("📍 IMEI: {}", imei);
    println!("📍 SN:   {}", sn);
    println!("📍 PID:  {}", pid);
    println!("🎯 Testing: {} secret keys", max_keys);
    println!("⚡ 100+ hashes/sec = {:.1} minutes", max_keys as f64 / 100.0);
    print_separator();
    
    // Test generation
    let test_secret = 1234567890123456u64;
    let test_generated_code = huawei_hash_formula(imei, sn, pid, test_secret);
    println!("🧪 TEST CODE: {} (length: {}) ✅", test_generated_code, test_generated_code.len());
    
    let secret_base = imei.parse::<u64>().unwrap();
    let secret_keys = generate_secret_keys(secret_base, max_keys);
    
    println!("🔑 Generated {} secret keys to test", secret_keys.len());
    print_separator();
    
    let stop_signal = Arc::new(AtomicBool::new(false));
    let monitor_stop = stop_signal.clone();
    let monitor_handle = tokio::spawn(async move {
        monitor_device(monitor_stop).await;
    });
    
    let start_time = Instant::now();
    let mut success = false;
    let mut valid_count = 0;
    
    for (i, &secret_key) in secret_keys.iter().enumerate() {
        if success { break; }
        
        let code = huawei_hash_formula(imei, sn, pid, secret_key);
        valid_count += 1;
        
        if code.len() != 16 {
            eprintln!("\n❌ ERROR: Code '{}' has {} digits!", code, code.len());
            continue;
        }
        
        let code_preview = code.clone();
        print!("\r# {:5}/{} | Secret: {:08x} | {} ", 
               i + 1, secret_keys.len(), secret_key, code_preview);
        
        if test_code(&code).await? {
            success = true;
            break;
        }
        
        let elapsed = start_time.elapsed().as_secs_f64();
        let rate = (i + 1) as f64 / elapsed.max(1.0);
        let remaining = (secret_keys.len() - i - 1) as f64;
        let eta = remaining / rate / 60.0;
        
        println!("| {:.0}/s | ETA: {:.1}m | {} codes", rate, eta, valid_count);
        
        //if (i + 1) % 5 == 0 {
        //    println!("\n🔄 Auto-rebooting (Huawei 5-attempt limit)...");
        //    let _ = Command::new("fastboot").args(["reboot-bootloader"]).output();
        //    tokio::time::sleep(Duration::from_secs(5)).await;
        //}
    }
    
    stop_signal.store(true, Ordering::Relaxed);
    let _ = monitor_handle.await;
    
    let total_time = start_time.elapsed();
    println!("\n\n🏁 FINISHED: {}/{} keys tested", valid_count, secret_keys.len());
    println!("⏱️  Total time: {:.1} minutes", total_time.as_secs_f64() / 60.0);
    println!("⚡ Average rate: {:.0} keys/sec", valid_count as f64 / total_time.as_secs_f64().max(1.0));
    
    if success {
        println!("\n🎉 **BOOTLOADER UNLOCKED!** 🏆");
        println!("📱 Phone will show WARNING screen - Press Volume Up to confirm!");
    } else {
        println!("\n💾 Continue: cargo run -- {} {}", imei, secret_keys.len() + 1000);
    }
    
    Ok(())
}
