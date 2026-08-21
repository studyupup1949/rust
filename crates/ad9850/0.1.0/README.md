# ad9850

## `ad9850` - Embedded driver for the AD9850 DDS synthesizer chip

The AD9850 is a DDS Synthesizer chip sold by Analog Devices. Check the [datasheet](https://www.analog.com/media/en/technical-documentation/data-sheets/AD9850.pdf) for general information about it.

This crate implements an interface for embedded devices to control such an AD9850 chip.

The only dependency of this crate is that your device has 4 digital output pins which implement the [`embedded_hal::digital::v2::OutputPin`] trait.

### Usage example

This example uses the [`arduino-hal`](https://github.com/Rahix/avr-hal). The `ad9850` library is not device specific though, so
it should be easy to adapt the example to other devices.

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
    ad9850.set_frequency(1.0);
}
```

### Supported features

- [x] Reset the device
- [x] Program in Serial mode
- [ ] Program in Parallel mode
- [ ] Power down / wakeup

### A note about timing

Communication with the Ad9850 involves sending "pulses" on the
RESET, W_CLK and FQ_UD lines. According to the datasheet, these
pulses must be at least $3.5ns$ long on RESET and W_CLK and at
least $7ns$ on the FQ_UD line.

This implementation does not insert any "delay" between toggling
the pins high and low, so the pulse width depends on the CPU frequency
of the device this code is run on.

Example: if the MCU runs at 16 MHz, the minimum pulse width attained
this way is $\frac{1}{16MHz} = 62ns$, which is way above the
required width of $7ns$.

If your MCU runs at a significantly higher frequency, this approach
may fail and you'll have to modify the code to insert delays.

Feel free to open an issue, if you run into timing issues.
