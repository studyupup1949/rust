#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{clock::ClockControl, gpio, i2c::I2C, peripherals, prelude::*, Delay};
use esp_println::println;

use adxl345_eh_driver;

#[entry]
fn main() -> ! {
    let peripherals = peripherals::Peripherals::take();
    let system = peripherals.SYSTEM.split();

    let clocks = ClockControl::max(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);

    let io = gpio::IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let sda = io.pins.gpio5;
    let scl = io.pins.gpio6;
    let i2c = I2C::new(peripherals.I2C0, sda, scl, 100u32.kHz(), &clocks);
    delay.delay_ms(100u32);

    let mut accel = adxl345_eh_driver::Driver::new(i2c, None).unwrap();

    loop {
        println!("Loop...");
        let (x, y, z) = accel.get_accel_raw().unwrap();
        println!("ADXL345: x: {}, y: {}, z: {}", x, y, z);
        delay.delay_ms(1000u32);
    }
}
