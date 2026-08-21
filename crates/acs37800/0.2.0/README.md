# Rust Drivers for the ACS37800 Energy Metering IC

[![CI](https://github.com/sbruton/acs37800/actions/workflows/ci.yml/badge.svg)](https://github.com/sbruton/acs37800/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/acs37800.svg)](https://crates.io/crates/acs37800)
[![Docs](https://docs.rs/acs37800/badge.svg)](https://docs.rs/acs37800)
[![MSRV](https://img.shields.io/badge/rustc-1.85%2B-blue.svg)](#minimum-supported-rust-version)

> [!IMPORTANT]
> This driver is a work in progress. Only the I²C variants of the IC are currently supported. Reading the EEPROM registers is the only currently supported operation.

## Crate Features

In most every case you will need to enable at least one feature depending on the variant of the ACS37800 you are targeting.

| Feature | Description        |
| ------- | ------------------ |
| `i2c`   | Enables I²C driver |
| `spi`   | Enables SPI driver |

## Example

```rust
// Instantiate the I²C bus device
let i2c = I2cdev::new("/dev/i2c-1").unwrap();

// Instantiate the ACS37800 I²C driver
let mut acs37800 = Acs37800I2c::builder().i2c(i2c).build();

// Read the EEPROM data
match acs37800.read_eeprom() {
    Ok(eeprom) => {
        println!("{eeprom:#?}");
    }
    Err(cause) => {
        eprintln!("Reading EEPROM failed: {cause}");
    }
}
```

The code above will print out:

```shell
Acs37800Eeprom {
    qvo_fine_codes: 30,
    qvo_fine_icodes_offset: 1920,
    sns_fine_codes: -454,
    crs_sns: 4,
    iavgsel_enabled: false,
    pavgsel_enabled: false,
    rms_avg_1: 0,
    rms_avg_2: 0,
    vchan_offset_codes: 0,
    ichan_delay_enabled: false,
    chan_delay_sel: 0,
    fault_threshold_codes: 70,
    fault_delay_setting: 0,
    vevent_cycles: 0,
    overvoltage_threshold_codes: 32,
    undervoltage_threshold_codes: 32,
    zerocross_pulse_width_us: 32,
    halfcycle_en: false,
    squarewave_en: false,
    zerocross_current_channel: false,
    zerocross_rising_edge: false,
    i2c_address_7bit: 127,
    i2c_address_disabled: false,
    dio0_sel_raw: 0,
    dio1_sel_raw: 0,
    n_cycles: 0,
    bypass_n_en: false,
}
```

Additional examples, including async usage, can be found in the [`examples`](./examples) folder
