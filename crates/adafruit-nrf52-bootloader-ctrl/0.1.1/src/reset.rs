//! Bootloader modules for nrf52 boards
//! Provides functions to reset the decive into different bootloader modes

use nrf52840_hal as hal;

/// Magic numbers for bootloader configuration
/// see https://github.com/adafruit/Adafruit_nRF52_Bootloader/blob/master/src/main.c#L96
enum BootloaderMagic {
    /// Magic value for jump to OTA application
    OtaApplication = 0xB1,
    /// Magic value to reset into OTA mode
    OtaReset = 0xA8,
    /// Magic value to reset into serial mode
    SerialOnlyReset = 0x4e,
    /// Magic value to reset into UF2 mode
    Uf2Reset = 0x57,
    /// Magic value to skip bootloader
    SkipBootloader = 0x6d,
}

/// helper function, writes Magic number into POWER:GPREGRET and executes reset
fn reset_magic(magic: BootloaderMagic) -> ! {
    unsafe {
        (*hal::pac::POWER::PTR)
            .gpregret
            .write(|w| w.bits(magic as u32))
    };
    hal::pac::SCB::sys_reset();
}

/// Resets the device into Device Firmware Update mode (DFU).
pub fn reset_into_app() -> ! {
    reset_magic(BootloaderMagic::OtaApplication);
}

/// Resets into OTA mode, which will wait for a new firmware to be writte.
pub fn reset_into_ota() -> ! {
    reset_magic(BootloaderMagic::OtaReset);
}

/// Resets into serial only mode.
pub fn reset_into_serial_only() -> ! {
    reset_magic(BootloaderMagic::SerialOnlyReset);
}

/// Resets the device into Device Firmware Update mode (UF2).
pub fn reset_into_uf2() -> ! {
    reset_magic(BootloaderMagic::Uf2Reset);
}

/// Resets and skips the bootloader. This will reset into the application
pub fn reset_skip_bootloader() -> ! {
    reset_magic(BootloaderMagic::SkipBootloader);
}
