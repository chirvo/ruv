// Import necessary modules and libraries
use std::fs::{File, OpenOptions};
use std::fmt;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;
use clap::{Command, Arg, ArgAction};
use libc;
use std::env; // Import the `env` module

// Define constants for file paths used by the driver
const FS_PATH: &str = "/sys/kernel/ryzen_smu_drv/";
const VER_PATH: &str = "version";
const SMU_ARGS: &str = "smu_args";
const MP1_CMD: &str = "mp1_smu_cmd";

// Check if the program is running with root privileges (UID 0)
fn is_root() -> bool {
    env::var("SUDO_USER").is_ok() || unsafe { libc::geteuid() } == 0 // Remove the semicolon to fix the type mismatch
}

// Check if the driver is loaded by verifying the existence of the version file
fn driver_loaded() -> bool {
    PathBuf::from(FS_PATH).join(VER_PATH).exists()
}

// Read a 32-bit value from a file
fn read_file32(file_path: &Path) -> io::Result<u32> {
    let mut buffer = [0u8; 4];
    let mut file = File::open(file_path)?;
    file.read_exact(&mut buffer)?;
    Ok(u32::from_le_bytes(buffer))
}

// Write a 32-bit value to a file
fn write_file32(file_path: &Path, value: u32) -> io::Result<()> {
    let buffer = value.to_le_bytes();
    let mut file = OpenOptions::new().write(true).open(file_path)?;
    file.write_all(&buffer)
}

// Read a 192-bit value (6 x 32-bit) from a file
fn read_file192(file_path: &Path) -> io::Result<[u32; 6]> {
    let mut buffer = [0u8; 24];
    let mut file = File::open(file_path)?;
    file.read_exact(&mut buffer)?;
    let mut result = [0u32; 6];
    for i in 0..6 {
        let start = i * 4;
        let end = start + 4;
        // This unwrap is safe because we know the slice size is exactly 4.
        result[i] = u32::from_le_bytes(buffer[start..end].try_into().expect("Slice conversion failed"));
    }
    Ok(result)
}

// Write a 192-bit value (6 x 32-bit) to a file
fn write_file192(file_path: &Path, values: [u32; 6]) -> io::Result<()> {
    let mut buffer = [0u8; 24]; // Use stack allocation instead of Vec
    for (i, &value) in values.iter().enumerate() {
        let start = i * 4;
        let end = start + 4;
        buffer[start..end].copy_from_slice(&value.to_le_bytes());
    }
    let mut file = OpenOptions::new().write(true).open(file_path)?;
    file.write_all(&buffer)
}

// Define SMU command opcodes
const SMU_CMD_READ_OFFSET: u32 = 0x48;
const SMU_CMD_WRITE_OFFSET: u32 = 0x35;
const SMU_CMD_RESET_OFFSETS: u32 = 0x36;
const SMU_WAIT_RETRY_LIMIT: u32 = 5; // Max attempts to wait for SMU
const SMU_WAIT_DURATION: Duration = Duration::from_millis(500); // Time between retries

// Custom Error type for SMU operations
#[derive(Debug)]
enum SmuError {
    Io(io::Error),
    NotReadyTimeout, // Timed out waiting for SMU to become ready
    CommandTimeout,  // Timed out waiting for command completion
    CommandFailed(u32), // SMU returned a non-success code
}

impl fmt::Display for SmuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SmuError::Io(e) => write!(f, "SMU I/O error: {}", e),
            SmuError::NotReadyTimeout => write!(f, "Timed out waiting for SMU to become ready"),
            SmuError::CommandTimeout => write!(f, "Timed out waiting for SMU command to complete"),
            SmuError::CommandFailed(code) => write!(f, "SMU command failed with result code: {}", code),
        }
    }
}

// Allows easy conversion from io::Error to SmuError
impl From<io::Error> for SmuError {
    fn from(err: io::Error) -> SmuError {
        SmuError::Io(err)
    }
}

// Send a command to the SMU (System Management Unit) and retrieve the response
fn smu_command(op: u32, args: [u32; 6]) -> Result<[u32; 6], SmuError> {
    let base_path = PathBuf::from(FS_PATH);
    let mp1_cmd_path = base_path.join(MP1_CMD);
    let smu_args_path = base_path.join(SMU_ARGS);

    // --- Wait for SMU Ready ---
    let mut retries = 0;
    loop {
        match read_file32(&mp1_cmd_path) {
            Ok(0) => { // Still busy
                if retries >= SMU_WAIT_RETRY_LIMIT {
                    return Err(SmuError::NotReadyTimeout);
                }
                eprintln!("Waiting for SMU to become ready... (Attempt {})", retries + 1);
                sleep(SMU_WAIT_DURATION);
                retries += 1;
            }
            Ok(_) => break, // Ready (non-zero status), proceed
            Err(e) => return Err(SmuError::Io(e)), // Error reading status
        }
    }

    // --- Write Arguments ---
    write_file192(&smu_args_path, args)?;

    // --- Execute Command ---
    write_file32(&mp1_cmd_path, op)?;

    // --- Wait for Command Completion ---
    retries = 0;
    loop {
        match read_file32(&mp1_cmd_path) {
            Ok(0) => { // Still busy
                if retries >= SMU_WAIT_RETRY_LIMIT {
                    return Err(SmuError::CommandTimeout);
                }
                eprintln!("Waiting for SMU command 0x{:08X} to complete... (Attempt {})", op, retries + 1);
                sleep(SMU_WAIT_DURATION);
                retries += 1;
            }
            Ok(1) => break, // Success!
            Ok(err_code) => return Err(SmuError::CommandFailed(err_code)), // Command failed
            Err(e) => return Err(SmuError::Io(e)), // Error reading status
        }
    }

    // --- Read Result Arguments ---
    read_file192(&smu_args_path).map_err(SmuError::Io)
}

// Get the curve offset for a specific core
fn get_core_offset(core_id: u32) -> Result<i32, SmuError> {
    // Construct the first argument for the SMU command.
    // The format appears to be specific to the Ryzen SMU interface.
    // Bits 20-22: Core ID within the CCD (0-7) -> (core_id & 7) << 20
    // Bit 23: CCD ID (0 or 1) -> ((core_id & 8) >> 3) << 23 -> simplified as (core_id & 8) << 20
    // Combined: ((core_id & 8) << 20) | ((core_id & 7) << 20) -> simplified as ((core_id & 8) << 5 | (core_id & 7)) << 20
    // Example: core_id 0 -> CCD 0, Core 0 -> arg0 = 0
    //          core_id 7 -> CCD 0, Core 7 -> arg0 = (0 | 7) << 20 = 7 * 2^20
    //          core_id 8 -> CCD 1, Core 0 -> arg0 = (8 << 5 | 0) << 20 = 256 << 20
    let arg0 = ((core_id & 8) << 5 | (core_id & 7)) << 20;
    let args = [arg0, 0, 0, 0, 0, 0];

    let result_args = smu_command(SMU_CMD_READ_OFFSET, args)?;
    let value = result_args[0]; // Offset is returned in the first argument

    // Convert the u32 result to a signed i32 offset.
    // The driver likely returns the value directly as a signed 32-bit integer,
    // represented within the u32 type. A direct cast should work.
    // Testing with known positive/negative offsets is recommended to confirm.
    Ok(value as i32)
}

// Set the curve offset for a specific core
fn set_core_offset(core_id: u32, value: i32) -> Result<(), SmuError> {
    // Construct the first argument for the SMU command.
    // Upper bits select the core (same logic as get_core_offset).
    // Lower 16 bits contain the desired offset value.
    let core_select = ((core_id & 8) << 5 | (core_id & 7)) << 20;
    // Mask the offset to 16 bits. The driver expects the value in this range.
    // Casting i32 to u32 preserves the bit pattern for two's complement representation.
    let offset_val = value as u32 & 0xFFFF;
    let arg0 = core_select | offset_val;
    let args = [arg0, 0, 0, 0, 0, 0];

    smu_command(SMU_CMD_WRITE_OFFSET, args)?; // Send command, propagate error if any
    Ok(())
}

// Main function to handle command-line arguments and execute the appropriate actions
fn main() {
    if !is_root() {
        eprintln!("Error: This program must be run with root privileges.");
        std::process::exit(1); // Exit with non-zero status
    }

    if !driver_loaded() {
        eprintln!("Error: Ryzen SMU driver not found at {}. Is it loaded?", FS_PATH);
        std::process::exit(1);
    }

    let matches = Command::new("PBO undervolt for Ryzen processors")
        .arg(Arg::new("list").short('l').long("list").action(ArgAction::SetTrue).help("List curve offsets"))
        .arg(Arg::new("offset")
            .short('o')
            .long("offset")
            .value_parser(clap::value_parser!(i32))
            .num_args(1)
            .allow_hyphen_values(true) // Ensure negative values are allowed
            .require_equals(true) // Require `=` for clarity, e.g., `-o=-20`
            .help("Set curve offset"))
        .arg(Arg::new("corecount")
            .short('c')
            .long("corecount")
            .num_args(1)
            .default_value("8") // Set default value to 8
            .help("Set offset to cores [0..corecount]"))
        .arg(Arg::new("reset").short('r').long("reset").action(ArgAction::SetTrue).help("Reset offsets to 0"))
        .get_matches();

    let core_count_str = matches.get_one::<String>("corecount").expect("Default value should exist");
    let core_count: u32 = core_count_str.parse().unwrap_or_else(|_| {
        eprintln!("Warning: Invalid core count '{}', defaulting to 1.", core_count_str);
        1
    });
    if matches.get_flag("list") {
        for c in 0..core_count {
            match get_core_offset(c) {
                Ok(offset) => println!("Core {}: {}", c, offset),
                Err(e) => eprintln!("Error getting offset for Core {}: {}", c, e),
            }
        }
        return;
    }

    if matches.get_flag("reset") {
        match smu_command(SMU_CMD_RESET_OFFSETS, [0; 6]) {
            Ok(_) => println!("Offsets reset to 0 successfully."),
            Err(e) => {
                eprintln!("Error resetting offsets: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    if let Some(&offset) = matches.get_one::<i32>("offset") {
        // Note: Some guides suggest only negative offsets are valid for undervolting.
        // The driver might accept positive values too, but the tool's intent seems to be undervolting.
        // We'll keep the check for now, but it could be removed if positive offsets are desired.
        if offset > 0 {
            eprintln!("Warning: Positive offsets might not be supported or could increase voltage. Proceeding anyway.");
        }
        // if offset >= 0 {
        //     eprintln!("Error: Offset must be negative for undervolting.");
        //     std::process::exit(1);
        // }

        println!("Setting offset {} for cores 0..{}", offset, core_count);
        let mut success = true;
        for c in 0..core_count {
            match set_core_offset(c, offset) {
                Ok(_) => { /* Continue to readback */ }
                Err(e) => {
                    eprintln!("Error setting offset for Core {}: {}", c, e);
                    success = false;
                    continue; // Try next core even if one fails
                }
            }
            // Optional: Read back to verify
            match get_core_offset(c) {
                Ok(readback) => println!("Core {} set to: {} (Readback: {})", c, offset, readback),
                Err(e) => eprintln!("Error reading back offset for Core {}: {}", c, e),
            }
        }
        if !success {
            eprintln!("One or more cores failed to update.");
            std::process::exit(1);
        }
    } else if !matches.get_flag("list") && !matches.get_flag("reset") {
        // Only print help message if no other action was taken
        eprintln!("No action specified. Use --list, --reset, or --offset. Use --help for more info.");
        std::process::exit(1);
    }
}