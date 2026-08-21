//! This is a platform agnostic Rust driver for the adxl343 accelerometer sensor,
//! via the [`embedded-hal`] traits. The embedded-hal crate describes the 
//! behavior of common SoC peripherals (e.g. I2c, SPI, GPIO) through the Rust
//! Traits System. Compatible with a `no_std` environment.
//!
//! [`embedded-hal`]: https://github.com/rust-embedded/embedded-hal
//!
//! This driver allows you to configure:
//! - ODR ([`OutputDataRate`])
//! - Justification (i.e. left or right alignment of data) ([`Alignment`])
//! - Resolution (i.e. whether to use 10 bits or full number of bits to represent measurement) 
//! ([`FullRes`])
//! - Range of measurements ([`AccelRange`])
//!
//! ## The Device
//! 
//! This driver is compatible with the ADXL343 device from Analog Devices. The ADXL343
//! is a digital accelerometer, capable of a 3600Hz sample rate, and a +/- 16g sensing range
//! with 1/256 resolution. 
//! 
//! This particular driver is capable of interfacing with the ADXL343 device over an a bus 
//! implemeing the I2c communication protocol.To ensure proper functionality, ground the alternate address pin: 
//! Pin 12. When the alternate address pin is high, the device address is 0x1D, instead of the intended 
//! 0x53. For more information visit the link below, to the datasheet.

//!
//! Datasheet:
//! [ADXL343](https://www.analog.com/media/en/technical-documentation/data-sheets/adxl343.pdf)
//!
//! ## Usage examples
//!
//! To use this driver, import this crate and an `embedded_hal` implementation for your system.
//!
//! ### Initialize the ADXL343Interface struct with the desired settings 
//!
//! ```
//! use linux_embedded_hal::I2cdev;
//! use adxl343_rs::{ADXL343Interface, ADXL343Settings, OutputDataRate};
//!
//! let i2c = I2cdev::new("/dev/i2c-1").unwrap(); 
//! let settings = ADXL343Settings::default().set_odr(OutputDataRate::Hz1600);
//! 
//! //does not write settings to any registers and ensures the device is not in measurement mode
//! let mut sensor = ADXL343Interface::new(i2c).with_settings(settings)?; 
//! ```
//!
//! ### Configure Device Registers and Turn on Measurement Mode
//!
//! ```
//! sensor.init()?; //writes settings to their respective registers in the adxl343
//! sensor.begin_measurements()?;
//! //wait ~10ms before collecting the first sample
//! ```
//!
//! ### Collect Raw Sample
//! A full sample (3 axis readings from one moment in time) is stored, within 
//! the adxl343, in 6 registers: DATAX0, DATAX1, ..., DATAZ1. For example, DATAX0 and DATAX1, 
//! comprehesively store the (16 bit) x-axis reading. The manner in which this half word bit string
//! corresponds to the body frame x-axis reading of proper acceleration is dependent upon the configured 
//! settings. Go to section Interpreting Axis Data for more information on how to interpret DATA_0 and 
//! DATA_1.
//!
//! ```
//! //once configured and measurement mode is turned on, we can start to sample data
//! let raw_accel_data: [u8; 6] = sensor.read_full_sample()?; //use a match to handle errors
//! let accel_data_g: [f32; 3] = sensor.read_accel()?; //obtain accelerometer reading in g's
//! ```
//! ### Destroy and Change Settings
//! ```
//! //turns off measurement mode
//! let (i2c, settings) = sensor.destroy();
//! ```
//! 
//! ```text

//! 
//! 
//! // make changes to settings or create a new ADXL343Settings struct and reinitialize
//! # }
//! ```
//! ## Interpreting Axis Data
//! 
//! All data corresponding to to a given axis is found in registers DATA_0 and DATA_1 (e.g. DATAZ0 
//! and DATAZ1 for the z-axis). The way this data should be interpreted is dependent upon the values
//! specified in ([`ADXL343Settings`]). The three fields affecting the structure of DATA_0, and DATA_1 are:
//! range, justification, and resolution. A running example will help to understand how these
//! three fields affect the structure of DATA_0 and DATA_1. 
//! 
//! Suppose that the adxl343 sensor is laying flat on a (normal) table. Since it measures proper acceleration, 
//! it should read (0, 0, 1) in g's. 
//! 
//! ### Resolution and Range
//! Also suppose that the range, corresponding enum: ([`AccelRange`]), is specified as +/-8g (each axis on the
//! device saturates at a reading with absolute value greater than 8) and the resolution, corresponding enum: ([`FullRes`]),
//! is equal to full_res. 
//! (DATAZ0, DATAZ1) will not read (0x01, 0x00), since the adxl343 measures proper acceleration is lsb's (least
//! standard bits), which has a direct conversion to g's. 
//! 
//! The method g_per_lsb, within the ADXL343Settings struct, shows the conversions.
//! ```
//! pub fn g_per_lsb(&self) -> f32 {
//!     match (self.resolution, self.range) {
//!         (FullRes::full_res, _) => 1.0/256.0,
//!         (FullRes::_10bit_res, AccelRange::_2g) => 1.0/256.0,
//!         (FullRes::_10bit_res, AccelRange::_4g) => 1.0/128.0,
//!         (FullRes::_10bit_res, AccelRange::_8g) => 1.0/64.0, 
//!         (FullRes::_10bit_res, AccelRange::_16g) => 1.0/32.0
//!     }
//! }
//! ```
//! In full resolution mode, 1g corresponds to 256 lsbs. The reading on the accelerometer's z axis
//! should be 256 (0x100). If we used 10-bit resolution, the reading should be 64 (0x40). Additionally, 
//! since we used full resolution, the data is stored in 12 bits: 0000 0000 0100, instead of 10. The following
//! method, also from ADXL343Settings, shows how both the range and resolution fields affect the number of 
//! bits used.
//! 
//! ```
//! pub fn resolution_to_bits(&self) -> u8 {
//!     match (self.resolution, self.range) {
//!         (FullRes::_10bit_res, _) => 10,
//!         (FullRes::full_res, AccelRange::_2g) => 10,
//!         (FullRes::full_res, AccelRange::_4g) => 11,
//!         (FullRes::full_res, AccelRange::_8g) => 12, 
//!         (FullRes::full_res, AccelRange::_16g) => 13
//!     }
//! }
//! ```
//! 
//! ### Justifcation
//! 
//! Justification, corresponding enum: ([`Alignment`]), specifies how the axis data is aligned within the registers DATA_0 and DATA_1. 
//! DATA_0 holds the lower order bit, whereas DATA_1 holds the higher bits. Right aligned specifies 
//! that the least significant bit of the measurement is located at bit 0 of DATA_0. Left aligned 
//! specifies that the most significant bit lies at bit 7 of DATA_1. In the case of the running example, 
//! proper acceleration = (0, 0, 1) in g's, +/-8 g's range, and full resolution (12 bits):
//! 
//! #### Right Aligned
//! <table>
//! <thead>
//! <tr>
//! <th>DATAZ1</th>
//! <th>DATAZ0</th>
//! </tr>
//! </thead>
//! <tbody>
//! <tr>
//! <td>xxxx 0000</td>
//! <td>0000 0100</td>
//! </tr>
//! </tbody>
//! </table>
//! 
//! #### Left Aligned
//! <table>
//! <thead>
//! <tr>
//! <th>DATAZ1</th>
//! <th>DATAZ0</th>
//! </tr>
//! </thead>
//! <tbody>
//! <tr>
//! <td>0000 0000</td>
//! <td>0100 xxxx</td>
//! </tr>
//! </tbody>
//! </table>
//! 
//! 
#![no_std]

pub mod registers; 
pub mod utils;
pub mod adxl343_interface;

pub use adxl343_interface::*;
pub use utils::settings::ADXL343Settings;
pub use registers::accel_configs::{AccelRange, OutputDataRate, Alignment, FullRes};






