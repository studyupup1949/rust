use acs37800::prelude::*;
use linux_embedded_hal::I2cdev;

fn main() {
    // Instantiate the I2C bus device
    let i2c = I2cdev::new("/dev/i2c-1").unwrap();

    // Instantiate the ACS37800 driver
    let mut acs37800 = Acs37800I2c::builder().i2c(i2c).build();

    // Read the EEPROM data
    let eeprom = match acs37800.read_eeprom() {
        Ok(eeprom) => eeprom,
        Err(e) => {
            eprintln!("Error reading IRMS raw value: {}", e);
            std::process::exit(1);
        }
    };

    // Print the EEPROM data
    println!("{eeprom:#?}");
}
