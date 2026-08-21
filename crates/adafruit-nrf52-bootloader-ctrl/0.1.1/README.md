# Adafruit nrf52 bootloader control library

This library provides functions to control the bootloader of Adafruit nrf52 boards.

[Adafruit nRF52 Bootloader Repo on Github](https://github.com/adafruit/Adafruit_nRF52_Bootloader)

## Features

```rust
/// Resets the device into Device Firmware Update mode (DFU).
pub fn reset_into_app() -> !

/// Resets into OTA mode, which will wait for a new firmware to be writte.
pub fn reset_into_ota() -> !

/// Resets into serial only mode.
pub fn reset_into_serial_only() -> !

/// Resets the device into Device Firmware Update mode (UF2).
pub fn reset_into_uf2() -> !

/// Resets and skips the bootloader. This will reset into the application
pub fn reset_skip_bootloader() -> !
```

## Usage

Add this to your `Cargo.toml`:
```toml
[dependencies]
adafruit-nrf52-bootloader-ctrl = "0.1.0"
```

switch into Uf2 mode for the next upload:
```rust
use adafruit_nrf52_bootloader_ctrl::reset;

reset::reset_into_uf2();
```
