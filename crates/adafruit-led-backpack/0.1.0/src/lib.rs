//! # adafruit-lcd-backpack
//!
//! Additional features on top of the [`ht16k33` crate](https://crates.io/crates/ht16k33) to drive an Adafruit 8x8 bicolor LED backpack (using traits from `embedded-hal`).
//!
//! It basically enables to address LED by color with the coordinate where `(0,0)` is at the bottom-left corner.
//!
#![no_std]
#![deny(warnings, missing_docs)]

use embedded_hal::blocking::i2c::{Write, WriteRead};
use ht16k33::{LedLocation, HT16K33};

/// Operate a Bicolor 8x8 Matrix LED backpack
pub trait BicolorMatrix8x8<E> {
    /// Updates a single LED in the 8x8 grid, where (0,0) is at the bottom-left corner
    fn update_bicolor_led(&mut self, x: u8, y: u8, color: Color) -> Result<(), E>;
}

/// LEDs can be 3 possible colors (and off)
#[derive(PartialEq, Clone)]
pub enum Color {
    /// Off state
    Off = 0x00,
    /// Only the green LED is on
    Green = 0x01,
    /// Only the red LED is on
    Red = 0x02,
    /// Both green and red LEDs are on, which makes it yellow
    Yellow = 0x03,
}

impl Into<(bool, bool)> for Color {
    fn into(self) -> (bool, bool) {
        match self {
            Color::Green => (true, false),
            Color::Red => (false, true),
            Color::Yellow => (true, true),
            Color::Off => (false, false),
        }
    }
}

impl<I2C, E> BicolorMatrix8x8<E> for HT16K33<I2C>
where
    I2C: Write<Error = E> + WriteRead<Error = E>,
{
    /// Update a specific LED with color, optionally switching on both red and green in-order to look yellow.
    ///
    /// # Arguments
    ///
    /// * `x` - x coordinate, from 0 to 8
    /// * `y` - y coordinate, from 0 to 8
    /// * `color` - color of the LED (or off)
    ///
    /// # Examples
    ///
    /// ```
    /// use ht16k33::i2c_mock::I2cMock;
    /// use ht16k33::{HT16K33, Display};
    /// use adafruit_led_backpack::{BicolorMatrix8x8, Color};
    /// # fn main() {
    ///
    /// // Create an I2C device.
    /// let mut i2c = I2cMock::new();
    ///
    /// // Create a HT16K33 instance, the implementation of BicolorMatrix8x8 is in scope.
    /// let mut ht16k33 = HT16K33::new(i2c, 0x70);
    /// ht16k33.initialize();
    /// ht16k33.update_bicolor_led(4, 4, Color::Red);
    /// ht16k33.set_display(Display::ON);
    /// ht16k33.write_display_buffer();
    ///
    /// # }
    /// ```
    fn update_bicolor_led(&mut self, x: u8, y: u8, color: Color) -> Result<(), E> {
        let (on1, on2) = color.into();
        // red LED
        let coord = y * 16 + x;
        self.update_display_buffer(LedLocation::new(coord / 8, coord % 8).unwrap(), on1);
        // green LED
        let coord = y * 16 + x + 8;
        self.update_display_buffer(LedLocation::new(coord / 8, coord % 8).unwrap(), on2);
        self.write_display_buffer()
    }
}
