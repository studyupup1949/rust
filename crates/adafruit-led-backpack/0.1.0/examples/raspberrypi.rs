use adafruit_led_backpack::*;
use ht16k33::{Display, HT16K33};
use rppal::i2c::I2c;
use std::{thread::sleep, time::Duration};

fn main() {
    let i2c = I2c::new()
        .expect("could not initialize I2c on your RPi, is the interface enabled in raspi-config?");
    let mut ht16k33 = HT16K33::new(i2c, 0x70);
    ht16k33.initialize().expect("failed to initialize HT16K33");
    ht16k33
        .set_display(Display::ON)
        .expect("could not switch the display on");

    for x in 0..8 {
        for y in 0..8 {
            let color = match (x, y) {
                (x, y) if x % 2 == 0 && y % 2 == 0 => Color::Yellow,
                (x, y) if x % 2 == 1 && y % 2 == 0 => Color::Green,
                _ => Color::Red,
            };
            ht16k33
                .update_bicolor_led(x, y, color)
                .expect("failed to update LED");

            sleep(Duration::from_millis(100));
        }
        sleep(Duration::from_millis(100));
    }
}
