# `ad9850` - Embedded driver for the AD9850 DDS synthesizer chip

[![Crates.io](https://img.shields.io/crates/v/ad9850?style=flat-square)](https://crates.io/crates/ad9850)
[![LICENSE-MIT](https://img.shields.io/crates/l/ad9850?style=flat-square)](./LICENSE-MIT)

The AD9850 is a DDS Synthesizer chip sold by Analog Devices. Check the [datasheet](https://www.analog.com/media/en/technical-documentation/data-sheets/AD9850.pdf) for general information about it.

This Rust crate implements an interface for embedded devices to control such an AD9850 chip.

## Supported features

- [x] Reset the device
- [x] Program in Serial mode
- [ ] Program in Parallel mode
- [ ] Power down / wakeup

## Usage example

This example uses the [`arduino-hal`](https://github.com/Rahix/avr-hal). The `ad9850` library is not device specific though, so
it should be easy to adapt the example to other devices. The only requirement is that the device's pins have an implementation
of the `OutputPin` trait from `embedded_hal`.

```rust
#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // Initialize the device
    let ad9850 = ad9850::Ad9850::new(
        pins.d4.into_output(), // Connect D4 (Arduino) to RESET (AD9850)
        pins.d5.into_output(), // Connect D5 (Arduino) to DATA (AD9850)
        pins.d6.into_output(), // Connect D6 (Arduino) to FQ_UD (AD9850)
        pins.d7.into_output(), // Connect D7 (Arduino) to W_CLK (AD9850)
    ).into_serial_mode().unwrap();
    //                   ^^^^ unwrap is ok here, since `set_low`/`set_high`
    //                        are infallible in the arduino-hal.

    // Set the output frequency to 1 MHz
    ad9850.set_frequency(1000000.0);
}
```

## Documentation

For further information, please refer to the generated documentation at https://docs.rs/ad9850/latest/ad9850/

## Contribution

Contributions are welcome!

For feature requests, issues or anything else, please [open an issue on github](https://github.com/nilclass/ad9850-rs/issues/new).

## 
