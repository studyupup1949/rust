//! ESP32 Fingerprint Sensor Example
//!
//! This example demonstrates how to use the a608_embedded library with an ESP32
//! microcontroller using esp-hal. It shows fingerprint enrollment and matching.
//!
//! # Hardware Setup
//! - ESP32 DevKit or similar board
//! - R503/R307/AS608 fingerprint sensor
//!
//! # Wiring
//! - Sensor TX  -> ESP32 GPIO16 (UART2 RX)
//! - Sensor RX  -> ESP32 GPIO17 (UART2 TX)
//! - Sensor VCC -> 3.3V
//! - Sensor GND -> GND
//!
//! # Dependencies (Cargo.toml)
//! ```toml
//! [dependencies]
//! a608_embedded = "0.1.0"
//! esp-hal = { version = "0.22", features = ["esp32"] }
//! esp-backtrace = { version = "0.14", features = ["esp32", "panic-handler", "println"] }
//! esp-println = { version = "0.12", features = ["esp32", "log"] }
//! log = "0.4"
//! ```

#![no_std]
#![no_main]

use a608_embedded::{
    ENROLLMISMATCH, FingerprintSensor, IMAGEMESS, LedColor, LedMode, NOFINGER, NOMATCH, NOTFOUND,
    OK, PACKETRECIEVEERR, SensorBuffer, SystemParam,
};
use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::Io,
    prelude::*,
    uart::{self, Uart},
};
use log::{error, info, warn};

#[entry]
fn main() -> ! {
    // Initialize ESP32 peripherals
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Initialize logging
    esp_println::logger::init_logger_from_env();

    info!("ESP32 Fingerprint Sensor Example");

    // Setup GPIO
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    // Setup UART2 for fingerprint sensor (57600 baud, default for most sensors)
    let uart_config = uart::Config::default()
        .baudrate(57600)
        .data_bits(uart::DataBits::DataBits8)
        .parity(uart::Parity::ParityNone)
        .stop_bits(uart::StopBits::Stop1);

    let uart2 = Uart::new_with_config(
        peripherals.UART2,
        uart_config,
        io.pins.gpio16, // RX
        io.pins.gpio17, // TX
    )
    .expect("Failed to initialize UART");

    // Create delay provider
    let delay = Delay::new();

    // Create fingerprint sensor instance
    // Default password is [0, 0, 0, 0] for most sensors
    let mut sensor = FingerprintSensor::new(uart2, [0, 0, 0, 0]);

    // Initialize the sensor
    info!("Initializing fingerprint sensor...");
    match sensor.init() {
        Ok(()) => info!("Sensor initialized successfully!"),
        Err(e) => {
            error!("Failed to initialize sensor: {:?}", e);
            loop {
                delay.delay_millis(1000u32);
            }
        }
    }

    // Print system parameters
    if let Some(params) = sensor.system_parameters() {
        info!("=== System Parameters ===");
        info!("Library size: {}", params.library_size);
        info!("Security level: {}", params.security_level);
        info!("Baud rate: {} * 9600", params.baudrate);
        info!(
            "Data packet size: {} bytes",
            params.data_packet_size.byte_count()
        );
    }

    // Count existing templates
    match sensor.count_templates() {
        Ok(OK) => info!("Stored templates: {}", sensor.template_count),
        Ok(err) => warn!("Count templates error: 0x{:02X}", err),
        Err(e) => error!("Count templates failed: {:?}", e),
    }

    // Set LED to blue breathing to indicate ready (R503 only)
    let _ = sensor.set_led(LedColor::Blue, LedMode::Breathe, 100, 0);

    info!("=== Fingerprint Demo Started ===");
    info!("Place your finger on the sensor to identify...");
    info!("Or wait 10 seconds to enter enrollment mode...");

    let mut loop_count = 0u32;
    let mut enrollment_mode = false;
    let mut enrollment_slot: u16 = 0;

    loop {
        if !enrollment_mode {
            // Try to identify fingerprint
            match try_identify(&mut sensor) {
                Ok(Some((finger_id, confidence))) => {
                    info!("*** MATCH FOUND ***");
                    info!("Finger ID: {}, Confidence: {}", finger_id, confidence);

                    // Flash green LED for success
                    let _ = sensor.set_led(LedColor::Blue, LedMode::Flash, 50, 3);
                    delay.delay_millis(1500u32);
                    let _ = sensor.set_led(LedColor::Blue, LedMode::Breathe, 100, 0);
                }
                Ok(None) => {
                    // No finger or no match - this is normal
                }
                Err(e) => {
                    error!("Identification error: {:?}", e);
                }
            }

            loop_count += 1;

            // After ~10 seconds (100 * 100ms), switch to enrollment mode
            if loop_count > 100 {
                enrollment_mode = true;
                enrollment_slot = find_empty_slot(&mut sensor).unwrap_or(0);
                info!("=== ENROLLMENT MODE ===");
                info!("Will enroll to slot {}", enrollment_slot);
                info!("Place your finger on the sensor (1st capture)...");
                let _ = sensor.set_led(LedColor::Purple, LedMode::Breathe, 100, 0);
            }
        } else {
            // Enrollment mode
            match enroll_fingerprint(&mut sensor, enrollment_slot, &delay) {
                Ok(()) => {
                    info!("*** ENROLLMENT SUCCESSFUL ***");
                    info!("Fingerprint stored at slot {}", enrollment_slot);

                    // Flash purple LED for success
                    let _ = sensor.set_led(LedColor::Purple, LedMode::Flash, 50, 5);
                    delay.delay_millis(2000u32);

                    // Return to identification mode
                    enrollment_mode = false;
                    loop_count = 0;
                    info!("Returning to identification mode...");
                    let _ = sensor.set_led(LedColor::Blue, LedMode::Breathe, 100, 0);
                }
                Err(msg) => {
                    warn!("Enrollment failed: {}", msg);
                    // Flash red LED for failure
                    let _ = sensor.set_led(LedColor::Red, LedMode::Flash, 50, 3);
                    delay.delay_millis(1500u32);

                    // Retry enrollment
                    info!("Retrying enrollment...");
                    let _ = sensor.set_led(LedColor::Purple, LedMode::Breathe, 100, 0);
                }
            }
        }

        delay.delay_millis(100u32);
    }
}

/// Try to identify a fingerprint
fn try_identify<UART, E>(
    sensor: &mut FingerprintSensor<UART>,
) -> Result<Option<(u16, u16)>, a608_embedded::Error<E>>
where
    UART: embedded_io::Write<Error = E> + embedded_io::Read<Error = E>,
{
    // Try to capture image
    match sensor.get_image()? {
        OK => {}
        NOFINGER => return Ok(None),  // No finger present
        IMAGEMESS => return Ok(None), // Image too messy
        _ => return Ok(None),
    }

    // Convert image to template in slot 1
    match sensor.image_2_tz(1)? {
        OK => {}
        _ => return Ok(None),
    }

    // Search for matching fingerprint
    match sensor.finger_fast_search()? {
        OK => Ok(Some((sensor.finger_id, sensor.confidence))),
        NOTFOUND => {
            log::info!("Finger not found in database");
            Ok(None)
        }
        _ => Ok(None),
    }
}

/// Enroll a new fingerprint
fn enroll_fingerprint<UART, E>(
    sensor: &mut FingerprintSensor<UART>,
    slot: u16,
    delay: &Delay,
) -> Result<(), &'static str>
where
    UART: embedded_io::Write<Error = E> + embedded_io::Read<Error = E>,
{
    // === First capture ===
    log::info!("Waiting for finger (1st capture)...");

    // Wait for finger
    loop {
        match sensor.get_image() {
            Ok(OK) => break,
            Ok(NOFINGER) => {}
            Ok(_) => return Err("Failed to capture first image"),
            Err(_) => return Err("UART error during first capture"),
        }
        delay.delay_millis(100u32);
    }

    // Convert to template in slot 1
    match sensor.image_2_tz(1) {
        Ok(OK) => {}
        _ => return Err("Failed to convert first image"),
    }

    log::info!("First capture OK. Remove finger...");
    let _ = sensor.set_led(LedColor::Blue, LedMode::On, 0, 0);

    // Wait for finger removal
    loop {
        match sensor.get_image() {
            Ok(NOFINGER) => break,
            _ => {}
        }
        delay.delay_millis(100u32);
    }

    delay.delay_millis(500u32);
    log::info!("Place same finger again (2nd capture)...");
    let _ = sensor.set_led(LedColor::Purple, LedMode::Breathe, 100, 0);

    // === Second capture ===
    loop {
        match sensor.get_image() {
            Ok(OK) => break,
            Ok(NOFINGER) => {}
            Ok(_) => return Err("Failed to capture second image"),
            Err(_) => return Err("UART error during second capture"),
        }
        delay.delay_millis(100u32);
    }

    // Convert to template in slot 2
    match sensor.image_2_tz(2) {
        Ok(OK) => {}
        _ => return Err("Failed to convert second image"),
    }

    log::info!("Second capture OK. Creating model...");

    // Create model from both templates
    match sensor.create_model() {
        Ok(OK) => {}
        Ok(ENROLLMISMATCH) => return Err("Fingerprints don't match"),
        _ => return Err("Failed to create model"),
    }

    // Store model
    match sensor.store_model(slot, 1) {
        Ok(OK) => {}
        _ => return Err("Failed to store model"),
    }

    Ok(())
}

/// Find an empty slot in the fingerprint database
fn find_empty_slot<UART, E>(sensor: &mut FingerprintSensor<UART>) -> Option<u16>
where
    UART: embedded_io::Write<Error = E> + embedded_io::Read<Error = E>,
{
    let library_size = sensor.library_size().unwrap_or(200);
    let num_pages = (library_size as f32 / 256.0).ceil() as u8;

    for page in 0..num_pages {
        let mut bitmap = [0u8; 32];
        if sensor.read_template_page(page, &mut bitmap).ok()? == OK {
            // Find first empty slot in this page
            for byte_idx in 0..32 {
                if bitmap[byte_idx] != 0xFF {
                    for bit_idx in 0..8 {
                        if (bitmap[byte_idx] & (1 << bit_idx)) == 0 {
                            let slot = (page as u16 * 256) + (byte_idx as u16 * 8) + bit_idx as u16;
                            if slot < library_size {
                                return Some(slot);
                            }
                        }
                    }
                }
            }
        }
    }

    // If no empty slot found via bitmap, just use template count
    Some(sensor.template_count)
}
