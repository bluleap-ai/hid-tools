# HID Command Tool

A Rust-based command-line tool for communicating with HID devices using Vendor ID (VID) and Product ID (PID).

## Requirements

- Rust and Cargo installed
- libusb development files (for Linux)
- Appropriate permissions to access USB devices

## Installation

```bash
cargo build --release
```

## Usage

```bash
# Basic usage to open a device
./target/release/hid-cmd --vid 0x1234 --pid 0x5678

# Send data to the device (hex string)
./target/release/hid-cmd --vid 0x1234 --pid 0x5678 --data "01020304"
```

### Arguments

- `--vid` or `-v`: Vendor ID of the HID device (required)
- `--pid` or `-p`: Product ID of the HID device (required)
- `--data` or `-d`: Data to send to the device (optional, hex string)

## Example

To send the bytes `[0x01, 0x02, 0x03, 0x04]` to a device with VID 0x1234 and PID 0x5678:

```bash
./target/release/hid-cmd --vid 0x1234 --pid 0x5678 --data "01020304"
```

## Notes

- The first byte of the data is treated as the report ID
- The program will attempt to read a response after sending data
- Make sure you have the necessary permissions to access USB devices 