//! STM32 Fingerprint Sensor Example
//!
//! This example demonstrates how to use the a608_embedded library with an STM32
//! microcontroller using stm32-hal2. It shows fingerprint enrollment and matching.
//!
//! # Hardware Setup
//! - STM32F4 Discovery or similar board (example uses STM32F411)
//! - R503/R307/AS608 fingerprint sensor
//!
//! # Wiring
//! - Sensor TX  -> STM32 PA10 (USART1 RX)
//! - Sensor RX  -> STM32 PA9  (USART1 TX)
//! - Sensor VCC -> 3.3V
//! - Sensor GND -> GND
//!
//! # Dependencies (Cargo.toml)
//! ```toml
//! [dependencies]
//! a608_embedded = "0.1.0"
//! stm32-hal2 = { version = "1.8", features = ["f411", "usart1"] }
//! cortex-m = "0.7"
//! cortex-m-rt = "0.7"
//! panic-halt = "0.2"
//! ```
//!
//! # Memory.x (example for STM32F411)
//! ```
//! MEMORY
//! {
//!   FLASH : ORIGIN = 0x08000000, LENGTH = 512K
//!   RAM   : ORIGIN = 0x20000000, LENGTH = 128K
//! }
//! ```

#![no_std]
#![no_main]

use a608_embedded::{
    ENROLLMISMATCH, FingerprintSensor, IMAGEMESS, LedColor, LedMode, NOFINGER, NOMATCH, NOTFOUND,
    OK, PACKETRECIEVEERR, SensorBuffer, SystemParam,
};
use cortex_m_rt::entry;
use panic_halt as _;
use stm32_hal2::{
    clocks::Clocks,
    gpio::{Pin, PinMode, Port},
    pac,
    usart::{Usart, UsartConfig, UsartDevice},
};

/// Simple delay using busy loop
fn delay_ms(ms: u32) {
    // Approximate delay - adjust based on clock speed
    // This assumes ~100MHz clock, ~10 cycles per iteration
    let iterations = ms * 10_000;
    for _ in 0..iterations {
        cortex_m::asm::nop();
    }
}

/// Simple debug output (can be replaced with ITM, semihosting, or UART)
macro_rules! debug_print {
    ($($arg:tt)*) => {
        // Replace with your preferred debug output method:
        // - ITM (Instrumentation Trace Macrocell)
        // - Semihosting
        // - Secondary UART
        // For now, this is a no-op
        let _ = ($($arg)*);
    };
}

#[entry]
fn main() -> ! {
    // Get peripherals
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    // Configure clocks
    let clock_cfg = Clocks::default();
    clock_cfg.setup().unwrap();

    // Configure GPIO pins for USART1
    // PA9  = TX (Alternate Function 7)
    // PA10 = RX (Alternate Function 7)
    let _tx_pin = Pin::new(Port::A, 9, PinMode::Alt(7));
    let _rx_pin = Pin::new(Port::A, 10, PinMode::Alt(7));

    // Configure USART1 for fingerprint sensor
    // Default baud rate for most fingerprint sensors is 57600
    let usart_cfg = UsartConfig {
        baud_rate: 57600,
        ..Default::default()
    };

    let usart1 = Usart::new(dp.USART1, usart_cfg, &clock_cfg);

    // Create fingerprint sensor instance
    // Default password is [0, 0, 0, 0] for most sensors
    let mut sensor = FingerprintSensor::new(usart1, [0, 0, 0, 0]);

    // Initialize the sensor
    debug_print!("Initializing fingerprint sensor...");

    match sensor.init() {
        Ok(()) => {
            debug_print!("Sensor initialized successfully!");
        }
        Err(e) => {
            debug_print!("Failed to initialize sensor: {:?}", e);
            // Halt on initialization failure
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    // Print system parameters
    if let Some(params) = sensor.system_parameters() {
        debug_print!("=== System Parameters ===");
        debug_print!("Library size: {}", params.library_size);
        debug_print!("Security level: {}", params.security_level);
        debug_print!("Baud rate: {} * 9600", params.baudrate);
        debug_print!(
            "Data packet size: {} bytes",
            params.data_packet_size.byte_count()
        );
    }

    // Count existing templates
    match sensor.count_templates() {
        Ok(OK) => {
            debug_print!("Stored templates: {}", sensor.template_count);
        }
        Ok(err) => {
            debug_print!("Count templates error: 0x{:02X}", err);
        }
        Err(e) => {
            debug_print!("Count templates failed: {:?}", e);
        }
    }

    // Set LED to blue breathing to indicate ready (R503 only)
    let _ = sensor.set_led(LedColor::Blue, LedMode::Breathe, 100, 0);

    debug_print!("=== Fingerprint Demo Started ===");
    debug_print!("Place your finger on the sensor...");

    // State machine for demo
    let mut state = DemoState::Identifying;
    let mut loop_count: u32 = 0;
    let mut enrollment_slot: u16 = 0;

    loop {
        match state {
            DemoState::Identifying => {
                // Try to identify fingerprint
                match try_identify(&mut sensor) {
                    Ok(Some((finger_id, confidence))) => {
                        debug_print!("*** MATCH FOUND ***");
                        debug_print!("Finger ID: {}, Confidence: {}", finger_id, confidence);

                        // Flash LED for success
                        let _ = sensor.set_led(LedColor::Blue, LedMode::Flash, 50, 3);
                        delay_ms(1500);
                        let _ = sensor.set_led(LedColor::Blue, LedMode::Breathe, 100, 0);
                    }
                    Ok(None) => {
                        // No finger or no match
                    }
                    Err(_) => {
                        debug_print!("Identification error");
                    }
                }

                loop_count += 1;

                // After ~10 seconds, switch to enrollment mode
                if loop_count > 100 {
                    state = DemoState::WaitingForFirstCapture;
                    enrollment_slot = find_empty_slot(&mut sensor).unwrap_or(0);
                    debug_print!("=== ENROLLMENT MODE ===");
                    debug_print!("Will enroll to slot {}", enrollment_slot);
                    debug_print!("Place your finger (1st capture)...");
                    let _ = sensor.set_led(LedColor::Purple, LedMode::Breathe, 100, 0);
                }
            }

            DemoState::WaitingForFirstCapture => {
                match sensor.get_image() {
                    Ok(OK) => {
                        // Convert to template in slot 1
                        match sensor.image_2_tz(1) {
                            Ok(OK) => {
                                debug_print!("First capture OK. Remove finger...");
                                let _ = sensor.set_led(LedColor::Blue, LedMode::On, 0, 0);
                                state = DemoState::WaitingForRemoval;
                            }
                            _ => {
                                debug_print!("Failed to convert first image");
                                state = DemoState::EnrollmentFailed;
                            }
                        }
                    }
                    Ok(NOFINGER) => {}
                    _ => {}
                }
            }

            DemoState::WaitingForRemoval => match sensor.get_image() {
                Ok(NOFINGER) => {
                    delay_ms(500);
                    debug_print!("Place same finger (2nd capture)...");
                    let _ = sensor.set_led(LedColor::Purple, LedMode::Breathe, 100, 0);
                    state = DemoState::WaitingForSecondCapture;
                }
                _ => {}
            },

            DemoState::WaitingForSecondCapture => {
                match sensor.get_image() {
                    Ok(OK) => {
                        // Convert to template in slot 2
                        match sensor.image_2_tz(2) {
                            Ok(OK) => {
                                debug_print!("Second capture OK. Creating model...");
                                state = DemoState::CreatingModel;
                            }
                            _ => {
                                debug_print!("Failed to convert second image");
                                state = DemoState::EnrollmentFailed;
                            }
                        }
                    }
                    Ok(NOFINGER) => {}
                    _ => {}
                }
            }

            DemoState::CreatingModel => {
                match sensor.create_model() {
                    Ok(OK) => {
                        // Store model
                        match sensor.store_model(enrollment_slot, 1) {
                            Ok(OK) => {
                                debug_print!("*** ENROLLMENT SUCCESSFUL ***");
                                debug_print!("Stored at slot {}", enrollment_slot);

                                // Flash LED for success
                                let _ = sensor.set_led(LedColor::Purple, LedMode::Flash, 50, 5);
                                delay_ms(2000);

                                // Return to identification mode
                                loop_count = 0;
                                state = DemoState::Identifying;
                                let _ = sensor.set_led(LedColor::Blue, LedMode::Breathe, 100, 0);
                            }
                            _ => {
                                debug_print!("Failed to store model");
                                state = DemoState::EnrollmentFailed;
                            }
                        }
                    }
                    Ok(ENROLLMISMATCH) => {
                        debug_print!("Fingerprints don't match");
                        state = DemoState::EnrollmentFailed;
                    }
                    _ => {
                        debug_print!("Failed to create model");
                        state = DemoState::EnrollmentFailed;
                    }
                }
            }

            DemoState::EnrollmentFailed => {
                // Flash red LED for failure
                let _ = sensor.set_led(LedColor::Red, LedMode::Flash, 50, 3);
                delay_ms(1500);

                // Retry enrollment
                debug_print!("Retrying enrollment...");
                state = DemoState::WaitingForFirstCapture;
                let _ = sensor.set_led(LedColor::Purple, LedMode::Breathe, 100, 0);
            }
        }

        delay_ms(100);
    }
}

/// Demo state machine states
#[derive(Clone, Copy)]
enum DemoState {
    Identifying,
    WaitingForFirstCapture,
    WaitingForRemoval,
    WaitingForSecondCapture,
    CreatingModel,
    EnrollmentFailed,
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
        NOFINGER => return Ok(None),
        IMAGEMESS => return Ok(None),
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
        NOTFOUND => Ok(None),
        _ => Ok(None),
    }
}

/// Find an empty slot in the fingerprint database
fn find_empty_slot<UART, E>(sensor: &mut FingerprintSensor<UART>) -> Option<u16>
where
    UART: embedded_io::Write<Error = E> + embedded_io::Read<Error = E>,
{
    let library_size = sensor.library_size().unwrap_or(200);
    let num_pages = ((library_size + 255) / 256) as u8;

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

    // Fallback: use template count as next slot
    Some(sensor.template_count)
}

/// Delete a specific fingerprint from the database
#[allow(dead_code)]
fn delete_fingerprint<UART, E>(
    sensor: &mut FingerprintSensor<UART>,
    slot: u16,
) -> Result<bool, a608_embedded::Error<E>>
where
    UART: embedded_io::Write<Error = E> + embedded_io::Read<Error = E>,
{
    match sensor.delete_model(slot)? {
        OK => Ok(true),
        _ => Ok(false),
    }
}

/// Clear all fingerprints from the database
#[allow(dead_code)]
fn clear_all_fingerprints<UART, E>(
    sensor: &mut FingerprintSensor<UART>,
) -> Result<bool, a608_embedded::Error<E>>
where
    UART: embedded_io::Write<Error = E> + embedded_io::Read<Error = E>,
{
    match sensor.empty_library()? {
        OK => Ok(true),
        _ => Ok(false),
    }
}

/// Set security level (1-5, default is 3)
#[allow(dead_code)]
fn set_security_level<UART, E>(
    sensor: &mut FingerprintSensor<UART>,
    level: u8,
) -> Result<bool, a608_embedded::Error<E>>
where
    UART: embedded_io::Write<Error = E> + embedded_io::Read<Error = E>,
{
    let level = level.clamp(1, 5);
    match sensor.set_sysparam(SystemParam::SecurityLevel, level)? {
        OK => Ok(true),
        _ => Ok(false),
    }
}

/// Download a fingerprint template from the sensor
#[allow(dead_code)]
fn download_template<UART, E>(
    sensor: &mut FingerprintSensor<UART>,
    slot: u16,
    buffer: &mut [u8],
) -> Result<usize, a608_embedded::Error<E>>
where
    UART: embedded_io::Write<Error = E> + embedded_io::Read<Error = E>,
{
    // Load template from flash to char buffer 1
    match sensor.load_model(slot, 1)? {
        OK => {}
        err => return Err(a608_embedded::Error::CommandFailed(err)),
    }

    // Download template data
    sensor.get_fpdata(SensorBuffer::Char { slot: 1 }, buffer)
}

/// Upload a fingerprint template to the sensor
#[allow(dead_code)]
fn upload_template<UART, E>(
    sensor: &mut FingerprintSensor<UART>,
    slot: u16,
    data: &[u8],
) -> Result<bool, a608_embedded::Error<E>>
where
    UART: embedded_io::Write<Error = E> + embedded_io::Read<Error = E>,
{
    // Upload template data to char buffer 1
    sensor.send_fpdata(data, SensorBuffer::Char { slot: 1 })?;

    // Store to flash
    match sensor.store_model(slot, 1)? {
        OK => Ok(true),
        _ => Ok(false),
    }
}
