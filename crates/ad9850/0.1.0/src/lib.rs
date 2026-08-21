#![no_std]
//! # `ad9850` - Embedded driver for the AD9850 DDS synthesizer chip
//!
//! The AD9850 is a DDS Synthesizer chip sold by Analog Devices. Check the [datasheet](https://www.analog.com/media/en/technical-documentation/data-sheets/AD9850.pdf) for general information about it.
//!
//! This crate implements an interface for embedded devices to control such an AD9850 chip.
//!
//! The only dependency of this crate is that your device has 4 digital output pins which implement the [`embedded_hal::digital::v2::OutputPin`] trait.
//!
//! ## Usage example
//!
//! This example uses the [`arduino-hal`](https://github.com/Rahix/avr-hal). The `ad9850` library is not device specific though, so
//! it should be easy to adapt the example to other devices.
//!
//! ```ignore
//! #[arduino_hal::entry]
//! fn main() -> ! {
//!     let dp = arduino_hal::Peripherals::take().unwrap();
//!     let pins = arduino_hal::pins!(dp);
//!
//!     // Initialize the device
//!     let ad9850 = ad9850::Ad9850::new(
//!         pins.d4.into_output(), // Connect D4 (Arduino) to RESET (AD9850)
//!         pins.d5.into_output(), // Connect D5 (Arduino) to DATA (AD9850)
//!         pins.d6.into_output(), // Connect D6 (Arduino) to FQ_UD (AD9850)
//!         pins.d7.into_output(), // Connect D7 (Arduino) to W_CLK (AD9850)
//!     ).into_serial_mode().unwrap();
//!     //                   ^^^^ unwrap is ok here, since `set_low`/`set_high`
//!     //                        are infallible in the arduino-hal.
//!
//!     // Set output frequency to 1 MHz
//!     ad9850.set_frequency(1e6);
//! }
//! ```
//!
//! ## Supported features
//!
//! - [x] Reset the device
//! - [x] Program in Serial mode
//! - [ ] Program in Parallel mode
//! - [ ] Power down / wakeup
//!
//! ## A note about timing
//!
//! Communication with the Ad9850 involves sending "pulses" on the
//! RESET, W_CLK and FQ_UD lines. According to the datasheet, these
//! pulses must be at least $3.5ns$ long for RESET and W_CLK and at
//! least $7ns$ for the FQ_UD line.
//!
//! This implementation does not insert any "delay" between toggling
//! the pins high and low, so the pulse width depends on the CPU frequency
//! of the device this code is run on.
//!
//! Example: if the MCU runs at 16 MHz, the minimum pulse width attained
//! this way is $ \frac{1}{16MHz} = 62ns $, which is way above the
//! required width of $7ns$.
//!
//! If your MCU runs at a significantly higher frequency, this approach
//! may fail and you'll have to modify the code to insert delays.
//!
//! Feel free to [open an issue](https://github.com/nilclass/ad9850-rs/issues/new), if you run into timing issues.

use embedded_hal::digital::v2::OutputPin;
use core::marker::PhantomData;

/// Default frequency of the oscillator connected to the AD9850.
pub const DEFAULT_OSCILLATOR_FREQUENCY: f32 = 125e6;

/// Represents a connection to a AD9850 device.
///
/// See [crate level documentation](crate), or check the [`new`](Ad9850::new) method for an entry point.
pub struct Ad9850<Mode, Reset, Data, FqUd, WClk> {
    osc_freq: f32,
    reset: Reset,
    data: Data,
    fq_ud: FqUd,
    w_clk: WClk,
    marker: PhantomData<Mode>,
}

impl<Reset, Data, FqUd, WClk, E> Ad9850<mode::Init, Reset, Data, FqUd, WClk>
where
    Reset: OutputPin<Error = E>,
    Data: OutputPin<Error = E>,
    FqUd: OutputPin<Error = E>,
    WClk: OutputPin<Error = E>,
{
    /// Construct a new Ad9850 instance, in inital mode.
    ///
    /// This call does not communicate with the device yet. You need to call `into_serial_mode` to initiate a reset before you can send any data.
    ///
    /// Example:
    /// ```ignore
    /// let ad9850 = Ad9850::new(
    ///     pins.d4.into_output(),
    ///     pins.d5.into_output(),
    ///     pins.d6.into_output(),
    ///     pins.d7.into_output(),
    /// ).into_serial_mode().unwrap();
    /// ```
    ///
    /// The four parameters correspond to the digital pins connected to the AD9850:
    ///
    /// | Signal | AD9850 Pin |
    /// |--------|------------|
    /// | reset  |     22     |
    /// | data   |     25     |
    /// | fq_ud  |      7     |
    /// | w_clk  |      8     |
    ///
    /// NOTE: the "data" pin is labeled "D7" in the AD9850 datasheet. It's used either as the MSB in parallel mode, or as the single data pin in serial mode.
    ///   Breakout boards usually expose this pin twice: once as D7 on the parallel interface, and again as a pin labeled DATA. For this reason, make sure
    ///   not to ground the D7 pin if you are planning to use serial mode.
    ///
    pub fn new(reset: Reset, data: Data, fq_ud: FqUd, w_clk: WClk) -> Self {
        Self { reset, data, fq_ud, w_clk, osc_freq: DEFAULT_OSCILLATOR_FREQUENCY, marker: PhantomData }
    }

    /// Same as [`new`](Ad9850::new), but allows the oscillator frequency to be specified.
    ///
    /// Use this if your board's oscillator is **not** 125 MHz.
    ///
    /// This value is used in calculations done by [`set_frequency`](Ad9850::set_frequency) and [`set_frequency_and_phase`](Ad9850::set_frequency_and_phase).
    pub fn new_with_osc_freq(reset: Reset, data: Data, fq_ud: FqUd, w_clk: WClk, osc_freq: f32) -> Self {
        Self { reset, data, fq_ud, w_clk, osc_freq, marker: PhantomData }
    }

    /// Reset the ad9850 device into serial mode.
    ///
    /// This is the only supported mode at the moment.
    ///
    /// Returns an error, if any of the `set_low` / `set_high` calls on one of the pins fail.
    pub fn into_serial_mode(mut self) -> Result<Ad9850<mode::Serial, Reset, Data, FqUd, WClk>, E> {
        // RESET pulse resets the registers & mode to default
        self.reset.set_high()?;
        self.reset.set_low()?;

        // single W_CLK pulse followed by FQ_UD pulse switches to serial input mode
        self.w_clk.set_high()?;
        self.w_clk.set_low()?;
        self.fq_ud.set_high()?;
        self.fq_ud.set_low()?;

        Ok(Ad9850 {
            reset: self.reset,
            data: self.data,
            fq_ud: self.fq_ud,
            w_clk: self.w_clk,
            osc_freq: self.osc_freq,
            marker: PhantomData,
        })
    }
}

/// Utility functions
pub mod util {
    pub fn frequency_to_tuning_word(output_frequency: f32, oscillator_freq: f32) -> u32 {
        (output_frequency / (oscillator_freq / 4294967296.0)) as u32
    }

    /// Turns phase into AD9850 register format.
    ///
    /// This function turns the given `phase` (a 5-bit unsigned integer) into
    /// a value suitable to be passed to [`crate::Ad9850::update`] as a control
    /// byte.
    ///
    /// If the given `phase` overflows 5 bits, `None` is returned.
    ///
    /// Example:
    /// ```
    /// # use ad98509::util::phase_to_control_byte;
    /// assert_eq!(Some(0b00001000), phase_to_control_byte(1));
    /// assert_eq!(Some(0b10000000), phase_to_control_byte(1 << 4));
    /// assert_eq!(None, phase_to_control_byte(1 << 5));
    /// ```
    pub fn phase_to_control_byte(phase: u8) -> Option<u8> {
        if phase > 0x1F {
            None
        } else {
            Some(phase << 3 & 0xF8)
        }
    }
}

impl<Reset, Data, FqUd, WClk, E> Ad9850<mode::Serial, Reset, Data, FqUd, WClk>
where
    Reset: OutputPin<Error = E>,
    Data: OutputPin<Error = E>,
    FqUd: OutputPin<Error = E>,
    WClk: OutputPin<Error = E>,
{
    /// Set output frequency to the given value (in Hz)
    ///
    /// Convenience function to set only the frequency, with a phase of $0$.
    /// Computes the proper tuning word (assuming an oscillator frequency of 125MHz)
    /// and calls `update`.
    pub fn set_frequency(&mut self, frequency: f32) -> Result<(), E> {
        self.update(util::frequency_to_tuning_word(frequency, DEFAULT_OSCILLATOR_FREQUENCY), 0)
    }

    /// Set output frequency and phase to given values
    ///
    /// 
    ///
    ///
    /// # Panics
    ///
    /// The given `phase` must not exceed 5 bits (decimal 31, 0x1F). Otherwise this function panics.
    ///
    pub fn set_frequency_and_phase(&mut self, frequency: f32, phase: u8) -> Result<(), E> {
        self.update(
            util::frequency_to_tuning_word(frequency, DEFAULT_OSCILLATOR_FREQUENCY),
            util::phase_to_control_byte(phase).expect("invalid phase"),
        )
    }

    /// Update device register with given data.
    ///
    /// This is a low-level interface. See the `set_*` methods for a high-level wrapper.
    ///
    /// The Ad9850 register is 40-bit wide:
    /// - The first 32 bit are the "tuning word", which determines the frequency
    /// - Next two bits are "control bits", which should always be zero (enforced by this function)
    /// - This is followed by a single "power down" bit
    /// - The remaining 5 bits determine the "phase" by which the output signal is shifted.
    ///
    /// To find the correct tuning word for a given frequency that you want to set, you'll need
    /// to know the frequency of the oscillator connected to the Ad9850.
    /// Assuming an oscillator frequency of 125 MHz (the default on most breakout boards),
    /// then the tuning word $tw$ for a given frequency $f$ can be found with this formula:
    ///
    /// $ tw = \frac{f}{2^{32} * 125MHz} $
    pub fn update(&mut self, tuning_word: u32, control_and_phase: u8) -> Result<(), E> {
        self.shift_out((tuning_word & 0xFF) as u8)?;
        self.shift_out((tuning_word >> 8 & 0xFF) as u8)?;
        self.shift_out((tuning_word >> 16 & 0xFF) as u8)?;
        self.shift_out((tuning_word >> 23 & 0xFF) as u8)?;
        // force control bits to be zero, to avoid surprises
        self.shift_out(control_and_phase & 0b11111100)?;
        self.fq_ud.set_high()?;
        self.fq_ud.set_low()?;
        Ok(())
    }

    fn shift_out(&mut self, byte: u8) -> Result<(), E> {
        // shift out to DATA, at rising edge of W_CLK (LSB first)
        for i in 0..8 {
            self.data.set_state(((byte >> i) & 1 == 1).into())?;
            self.w_clk.set_high()?;
            self.w_clk.set_low()?;
        }
        Ok(())
    }
}

/// Marker types for different modes.
///
/// These types are used for the `Mode` type parameter of [`Ad9850`].
pub mod mode {
    /// Initial mode. No communication has happened.
    pub struct Init;
    /// Serial mode. Device is reset and put into the proper mode. Updates can happen.
    pub struct Serial;
}
