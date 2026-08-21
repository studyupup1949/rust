#[macro_use]
extern crate log;
extern crate log4rs;

use std::error::Error;
use std::thread;
use std::time::Duration;
use rppal::i2c::I2c;
extern crate adafruit_aht10;

fn main() -> Result<(), Box<dyn Error>> {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();

    let i2c_dev = I2c::new()?;
    let mut aht10 = adafruit_aht10::AdafruitAHT10::new(i2c_dev);
    // Read humidity and temperature.
    loop {
        thread::sleep(Duration::from_millis(500));
        match aht10.read_data() {
            Ok((humidity, temperature)) => {
                info!("Humidity: {}%", humidity);
                info!("Temperature: {}°C", temperature);
            }
            Err(_) => {
                error!("Error reading from sensor");
            }
        }
    }
}