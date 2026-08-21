# adafruit-alphanum4

Additional features on top of the [`ht16k33` crate](https://crates.io/crates/ht16k33) to drive an [Adafruit 14-segment LED Alphanumeric Backpack](https://learn.adafruit.com/adafruit-led-backpack/0-54-alphanumeric) using traits from `embedded-hal`.

[![Crate](https://img.shields.io/crates/v/adafruit-alphanum4.svg)](https://crates.io/crates/adafruit-alphanum4)
[![Documentation](https://docs.rs/adafruit-alphanum4/badge.svg)](https://docs.rs/adafruit-alphanum4)

## Features
* Sending a `u8` to one of the 4 segments
* Sending an `AsciiChar` to one of the 4 segments
* Setting or unsetting the dot associated with one of the 4 segments
* Formatting a `f32` to 1 to 4 segments

## Example:
```
    const DISP_I2C_ADDR: u8 = 112;

    let freq: Hertz = 20.khz().into();

    let mut i2c2 = blocking_i2c(
        I2c::i2c2(
            device.I2C2, 
            (scl_pin, sda_pin),
            Mode::Standard { frequency: freq.0 },
            clocks,
            &mut rcc.apb1,
        ),
        clocks,
        500,
        2,
        500,
        500,
    );

    let mut ht16k33 = HT16K33::new(i2c2, DISP_I2C_ADDR);
    ht16k33.initialize().expect("Failed to initialize ht16k33");

    // Sending individual digits
    ht16k33.update_buffer_with_digit(Index::One, 1);
    ht16k33.update_buffer_with_digit(Index::Two, 2);
    ht16k33.update_buffer_with_digit(Index::Three, 3);
    ht16k33.update_buffer_with_digit(Index::Four, 4);

    // Sending ascii
    ht16k33.update_buffer_with_char(Index::One, AsciiChar::new('A'));
    ht16k33.update_buffer_with_char(Index::Two, AsciiChar::new('B'));

    // Setting the decimal point 
    ht16k33.update_buffer_with_dot(Index::Two, true);

    // Formatting a float using the whole display
    ht16k33.update_buffer_with_float(Index::One, -3.14, 2, 10).unwrap();

    // Putting a character in front of a float
    ht16k33.update_buffer_with_char(Index::One, AsciiChar::new('X'));
    ht16k33.update_buffer_with_float(Index::Two, -3.14, 2, 10).unwrap(); //Display will read "X-3.1"

    // This will panic because there aren't enough digits to display this number
    ht16k33.update_buffer_with_float(Index::One, 12345., 0, 10).expect("Oops"); 

    // Note: none of the above methods actually commit the buffer to the display, call write_display_buffer to actually send it to the display
    ht16k33.write_display_buffer().unwrap()
```

## Performance warning

Due to the api of the ht16k33 crate the display buffer is not directly accessible so each LED that makes up the character is updated sequentially. The way the hardware on this backpack is set up allows a character to be updated by setting a single 16-bit value in the buffer. Iterating over each bit of the 16 every update is clearly not optimal but it's sufficiently fast for my current usage. If the ht16k33 crate is updated to grant mut access to the buffer this can be improved.

## License

Free and open source software distributed under the terms of both the [MIT License][lm] and the [Apache License 2.0][la].

[lm]: LICENSE-MIT
[la]: LICENSE-APACHE