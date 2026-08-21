//! Board support package for the Adafruit Feather RP2040 ThinkInk.
//!
//! <https://learn.adafruit.com/adafruit-rp2040-feather-thinkink/overview>
#![no_std]

pub use hal::pac;
pub use rp2040_hal as hal;

#[cfg(feature = "rt")]
pub use rp2040_hal::entry;

/// The linker will place this boot block at the start of our program image. We
/// need this to help the ROM bootloader get our code up and running.
#[cfg(feature = "boot2")]
#[link_section = ".boot2"]
#[no_mangle]
#[used]
pub static BOOT2_FIRMWARE: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

hal::bsp_pins!(
    Gpio0 { name: tx },
    Gpio1 { name: rx },
    Gpio2 { name: sda },
    Gpio3 { name: scl },
    Gpio14 { name: sck },
    Gpio15 { name: mosi },
    Gpio8 { name: miso },

    Gpio4 { name: d4 },
    Gpio5 { name: d5 },
    Gpio6 { name: d6 },
    Gpio9 { name: d9 },
    Gpio10 { name: d10 },
    Gpio11 { name: d11 },
    Gpio12 { name: d12 },
    Gpio24 { name: d24 },
    Gpio25 { name: d25 },
    Gpio26 { name: a0 },
    Gpio27 { name: a1 },
    Gpio28 { name: a2 },
    Gpio29 { name: a3 },

    /// Connected to D13 pin; active high
    Gpio13 { name: led },
    /// Supplies neopixel LED
    Gpio20 { name: neopixel_power },
    Gpio21 { name: neopixel },

    // e-paper display pins
    Gpio16 { name: epd_busy },
    Gpio17 { name: epd_reset },
    Gpio18 { name: epd_dc },
    Gpio19 { name: epd_cs },
    Gpio22 { name: epd_sck },
    Gpio23 { name: epd_mosi },
);

pub const XOSC_CRYSTAL_FREQ: u32 = 12_000_000;
