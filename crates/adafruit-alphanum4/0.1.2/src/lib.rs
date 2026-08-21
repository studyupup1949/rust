//! # adafruit-alphanum4
//! 
//! Additional features on top of the [`ht16k33` crate](https://crates.io/crates/ht16k33) to drive an [Adafruit 14-segment LED Alphanumeric Backpack](https://learn.adafruit.com/adafruit-led-backpack/0-54-alphanumeric) using traits from `embedded-hal`.
//! 
//! ## Features
//! * Sending a `u8` to one of the 4 segments
//! * Sending an `AsciiChar` to one of the 4 segments
//! * Setting or unsetting the dot associated with one of the 4 segments
//! * Formatting a `f32` to 1 to 4 segments
//!
//! ## Example
//!
//!```
//!    const DISP_I2C_ADDR: u8 = 112;
//!
//!    let freq: Hertz = 20.khz().into();
//!
//!    let mut i2c2 = blocking_i2c(
//!        I2c::i2c2(
//!            device.I2C2, 
//!            (scl_pin, sda_pin),
//!            Mode::Standard { frequency: freq.0 },
//!            clocks,
//!            &mut rcc.apb1,
//!        ),
//!        clocks,
//!        500,
//!        2,
//!        500,
//!        500,
//!    );
//!
//!    let mut ht16k33 = HT16K33::new(i2c2, DISP_I2C_ADDR);
//!    ht16k33.initialize().expect("Failed to initialize ht16k33");
//!
//!    // Sending individual digits
//!    ht16k33.update_buffer_with_digit(Index::One, 1);
//!    ht16k33.update_buffer_with_digit(Index::Two, 2);
//!    ht16k33.update_buffer_with_digit(Index::Three, 3);
//!    ht16k33.update_buffer_with_digit(Index::Four, 4);
//!
//!    // Sending ascii
//!    ht16k33.update_buffer_with_char(Index::One, AsciiChar::new('A'));
//!    ht16k33.update_buffer_with_char(Index::Two, AsciiChar::new('B'));
//!
//!    // Setting the decimal point 
//!    ht16k33.update_buffer_with_dot(Index::Two, true);
//!
//!    // Formatting a float using the whole display
//!    ht16k33.update_buffer_with_float(Index::One, -3.14, 2, 10).unwrap();
//!
//!    // Putting a character in front of a float
//!    ht16k33.update_buffer_with_char(Index::One, AsciiChar::new('X'));
//!    ht16k33.update_buffer_with_float(Index::Two, -3.14, 2, 10).unwrap(); //Display will read "X-3.1"
//!
//!    // This will panic because there aren't enough digits to display this number
//!    ht16k33.update_buffer_with_float(Index::One, 12345., 0, 10).expect("Oops"); 
//!
//!    // Note: none of the above methods actually commit the buffer to the display, call write_display_buffer to actually send it to the display
//!    ht16k33.write_display_buffer().unwrap()
//!```
//! 
//! ## Performance warning
//!
//! Due to the api of the ht16k33 crate the display buffer is not directly accessible so each LED that makes up the character is updated sequentially. The way the hardware on this backpack is set up allows a character to be updated by setting a single 16-bit value in the buffer. Iterating over each bit of the 16 every update is clearly not optimal but it's sufficiently fast for my current usage. If the ht16k33 crate is updated to grant mut access to the buffer this can be improved.

#![no_std]
#![deny(warnings, missing_docs)]

mod fonts;
use fonts::*;

use embedded_hal::blocking::i2c::{Write, WriteRead};
use ht16k33::{
    DisplayDataAddress,
    DisplayData,
    LedLocation, 
    HT16K33,
    COMMONS_SIZE,
};
pub use ascii::AsciiChar;

/// Possible errors returned by this crate
#[derive(Debug)]
pub enum Error {
    /// Error indicating there aren't enough digits to display the given float value
    InsufficientDigits,
}

/// Trait enabling using the Adafruit 14-segment LED Alphanumeric Backpack
pub trait AlphaNum4<E> {
    /// Update the buffer with a digit value (0 to 9) at the specified index
    fn update_buffer_with_digit(&mut self, index: Index, value: u8);
    /// Update the buffer to turn the . on or off at the specified index
    fn update_buffer_with_dot(&mut self, index: Index, dot_on: bool);
    /// Update the buffer with an acii character at the specified index
    fn update_buffer_with_char(&mut self, index: Index, value: AsciiChar);
    /// Update the buffer with a formatted float not starting before the specified index
    fn update_buffer_with_float(&mut self, index: Index, value: f32, fractional_digits: u8, base: u8) -> Result<(), Error>;
}

/// The index of a segment
#[derive(Clone, Copy, PartialEq)]
pub enum Index {
    /// First digit
    One,
    /// Second digit
    Two,
    /// Third digit
    Three,
    /// Fourth digit
    Four,
}

impl From<Index> for u8 {
    fn from(i: Index) -> u8 {
        match i {
            Index::One => 0,
            Index::Two => 1,
            Index::Three => 2,
            Index::Four => 3,
        }
    }
}

impl From<u8> for Index {
    fn from(v: u8) -> Index {
        match v {
            0 => Index::One,
            1 => Index::Two,
            2 => Index::Three,
            3 => Index::Four,
            _ => panic!("Invalid index > 3"),
        }
    }
}

fn set_bit<I2C, E>(display: &mut HT16K33<I2C>, index: Index, bit: u8, on: bool) 
where
    I2C: 
        Write<Error = E> + 
        WriteRead<Error = E> +  
{
    debug_assert!((bit as usize) < (COMMONS_SIZE * 2));
    let index = u8::from(index) * 2;
    let row = DisplayDataAddress::from_bits_truncate(if bit < 8 {
        index
    } else {
        index + 1
    });
    let common = DisplayData::from_bits_truncate(1 << (bit % 8));
    display.update_display_buffer(
        LedLocation { row, common },
        on,
    );
}

fn update_bits<I2C, E>(display: &mut HT16K33<I2C>, index: Index, bits: u16) 
where
    I2C: 
        Write<Error = E> + 
        WriteRead<Error = E> +  
{
    for i in 0..16 {
        let on = ((bits >> i) & 1) == 1;
        set_bit(display, index, i, on);
    }
}

impl<I2C, E> AlphaNum4<E> for HT16K33<I2C>
where
    I2C: 
        Write<Error = E> + 
        WriteRead<Error = E> +
{
    /// Update the buffer with a digit value (0 to 9) at the specified index
    fn update_buffer_with_digit(&mut self, index: Index, value: u8) {
        let value = value as usize;
        assert!(value < NUMBER_FONT_TABLE.len());
        let bits = NUMBER_FONT_TABLE[value];
        update_bits(self, index, bits);
    }
    /// Update the buffer to turn the . on or off at the specified index
    fn update_buffer_with_dot(&mut self, index: Index, dot_on: bool) {
        set_bit(self, index, DOT_BIT, dot_on);
    }
    /// Update the buffer with an acii character at the specified index
    fn update_buffer_with_char(&mut self, index: Index, value: AsciiChar) {
        let bits = ASCII_FONT_TABLE[value.as_byte() as usize];
        update_bits(self, index, bits);
    }
    /// Update the buffer with a formatted float not starting before the specified index
    /// The logic for this is copied mostly from from the adafruit library. Only difference is this allows the start index to be > 0
    fn update_buffer_with_float(&mut self, index: Index, mut value: f32, mut fractional_digits: u8, base: u8) -> Result<(), Error> {
        let index = u8::from(index);

        // Available digits on display
        let mut numeric_digits = 4 - index;
        
        let is_negative = if value < 0. {
            // The sign will take up one digit
            numeric_digits -= 1;
            // Flip the sign to do the rest of the formatting
            value *= -1.;
            true
        } else {
            false
        };

        let base = base as u32;
        let basef = base as f32;

        // Work out the multiplier needed to get all fraction digits into an integer
        let mut to_int_factor = base.pow(fractional_digits as u32) as f32;

        // Get an integer containing digits to be displayed 
        let mut display_number = ((value * to_int_factor) + 0.5) as u32;

        // Calculate the upper bound given the number of digits available
        let too_big = base.pow(numeric_digits as u32);

        // If the number is too large, reduce fractional digits
        while display_number >= too_big {
            fractional_digits -= 1;
            to_int_factor /= basef;
            display_number = ((value * to_int_factor) + 0.5) as u32;
        }

        // Did we lose the decimal?
        if to_int_factor < 1. {
            return Err(Error::InsufficientDigits)
        }

        // Digit we're working on, less the start position
        let mut display_pos = (3 - index) as i8;

        if display_number == 0 {
            // Write out the 0
            self.update_buffer_with_digit(
                (index + (display_pos as u8)).into(), 
                0,
            );
            // Move the current pos along
            display_pos -= 1;
        } else {
            let mut i = 0;
            while display_number != 0 || i <= fractional_digits {
                let digit_index = (index + (display_pos as u8)).into();
                // Write out the current digit
                self.update_buffer_with_digit(
                    digit_index,
                    (display_number % base) as u8,
                );
                // Add the decimal if necessary
                if fractional_digits != 0 && i == fractional_digits {
                    self.update_buffer_with_dot(
                        digit_index,
                        true,
                    );
                }
                // Move the current pos along
                display_pos -= 1;
                // Move the number along
                display_number /= base;
                i += 1;
            }
        }

        if is_negative {
            // Add the minus sign
            self.update_buffer_with_char(
                (index + (display_pos as u8)).into(),
                AsciiChar::new('-'),
            );
            // Move the current pos along
            display_pos -= 1;
        }

        // Clear any remaining segments
        while display_pos >= 0 {
            update_bits(self, (index + (display_pos as u8)).into(), 0);
            // Move the current pos along
            display_pos -= 1;
        }

        Ok(())
    }
}