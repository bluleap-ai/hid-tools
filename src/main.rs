use clap::Parser;
use hidapi::HidApi;
use std::time::Duration;
use std::thread;

const HID_REPORT_SIZE: usize = 64;
const VENDOR_PAGE: u16 = 0xFF42;
const MAX_RETRIES: u32 = 3;
const RETRY_DELAY_MS: u64 = 100;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Vendor ID of the HID device (hexadecimal)
    #[arg(short, long, value_parser = parse_hex)]
    vid: u16,

    /// Product ID of the HID device (hexadecimal)
    #[arg(short, long, value_parser = parse_hex)]
    pid: u16,

    /// Data to send (hex string, will be padded to 64 bytes)
    #[arg(short = 's', long)]
    data: Option<String>,

    /// Number of times to retry if device is busy
    #[arg(short = 'r', long, default_value = "3")]
    retries: u32,

    /// Delay between retries in milliseconds
    #[arg(short = 'd', long, default_value = "100")]
    retry_delay: u64,

    /// Keep reading input reports after sending data
    #[arg(short = 'c', long = "continuous", default_value = "false")]
    continuous: bool,
}

fn parse_hex(s: &str) -> Result<u16, String> {
    let s = s.trim_start_matches("0x");
    u16::from_str_radix(s, 16).map_err(|e| e.to_string())
}

fn open_device_with_retry(api: &HidApi, vid: u16, pid: u16, vendor_page: u16, max_retries: u32, retry_delay_ms: u64) -> anyhow::Result<hidapi::HidDevice> {
    let mut last_error = None;
    
    for attempt in 0..=max_retries {
        // Find the vendor-specific interface
        let device_info = api.device_list()
            .find(|d| d.vendor_id() == vid && 
                      d.product_id() == pid && 
                      d.usage_page() == vendor_page)
            .or_else(|| {
                // Fallback to any interface if vendor page not found
                api.device_list()
                    .find(|d| d.vendor_id() == vid && 
                              d.product_id() == pid)
            });

        if let Some(device_info) = device_info {
            match api.open_path(device_info.path()) {
                Ok(device) => return Ok(device),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_retries {
                        println!("Attempt {} failed: {}. Retrying in {}ms...", 
                            attempt + 1, last_error.as_ref().unwrap(), retry_delay_ms);
                        thread::sleep(Duration::from_millis(retry_delay_ms));
                    }
                }
            }
        } else {
            return Err(anyhow::anyhow!("Device not found"));
        }
    }
    
    Err(anyhow::anyhow!("Failed to open device after {} retries: {}", 
        max_retries, last_error.unwrap()))
}

fn read_input_reports(device: &hidapi::HidDevice) {
    let mut input_report = [0u8; HID_REPORT_SIZE];
    loop {
        match device.read(&mut input_report) {
            Ok(len) => {
                println!("\nReceived Input Report ({} bytes):", len);
                println!("Hex: {}", hex::encode(&input_report[..len]));
            }
            Err(e) => {
                eprintln!("Error reading input report: {}", e);
                break;
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    
    // Initialize the HID API
    let api = HidApi::new()?;
    
    println!("Searching for devices with VID:PID = {:04x}:{:04x}\n", args.vid, args.pid);
    
    // Open device with retries
    let device = open_device_with_retry(&api, args.vid, args.pid, VENDOR_PAGE, args.retries, args.retry_delay)?;
    
    println!("Successfully opened device");
    
    if let Some(data) = args.data {
        // Prepare output report (64 bytes)
        let mut output_report = [0u8; HID_REPORT_SIZE];
        
        // Convert hex string to bytes
        let bytes = hex::decode(data)?;
        let len = bytes.len().min(HID_REPORT_SIZE);
        output_report[..len].copy_from_slice(&bytes[..len]);
        
        println!("\nSending Output Report ({} bytes):", HID_REPORT_SIZE);
        println!("Hex: {}", hex::encode(&output_report));
        
        // Send output report
        match device.write(&output_report) {
            Ok(_) => println!("Successfully sent data"),
            Err(e) => eprintln!("Error sending data: {}", e),
        }
        
        if args.continuous {
            // Keep reading input reports in a loop
            read_input_reports(&device);
        } else {
            // Read a single input report
            let mut input_report = [0u8; HID_REPORT_SIZE];
            match device.read(&mut input_report) {
                Ok(len) => {
                    println!("\nReceived Input Report ({} bytes):", len);
                    println!("Hex: {}", hex::encode(&input_report[..len]));
                }
                Err(e) => eprintln!("Error reading response: {}", e),
            }
        }
    } else if args.continuous {
        // Just read input reports continuously
        read_input_reports(&device);
    } else {
        // Read a single input report
        let mut input_report = [0u8; HID_REPORT_SIZE];
        match device.read(&mut input_report) {
            Ok(len) => {
                println!("\nCurrent Input Report ({} bytes):", len);
                println!("Hex: {}", hex::encode(&input_report[..len]));
            }
            Err(e) => println!("Could not read input report: {}", e),
        }
    }
    
    Ok(())
}
