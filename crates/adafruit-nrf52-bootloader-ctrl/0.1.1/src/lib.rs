//! Controller for Adafruit nRF52 Bootloader
//! 
//! Provides functions to reset the device into different bootloader modes.
//! This is useful for example to reset into DFU mode to update the firmware.
//! 
//! Adafruit nRF52 Bootloader repo: <https://github.com/adafruit/Adafruit_nRF52_Bootloader>
//! 
//! How to update / flash the bootloader onto your board:
//! <https://learn.adafruit.com/introducing-the-adafruit-nrf52840-feather/update-bootloader>

#![no_std]

pub mod reset;
