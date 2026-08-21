//! # I²C Driver for ACS37800 Power Monitoring IC
//!
//! <div style="border-left: 4px solid #812ae4ff; padding: 0.5em 1em;">
//! <strong>IMPORTANT</strong><br/>
//! This module provides a driver for the I²C variants of the Allegro Microsystems ACS37800 power
//! monitoring IC. If you are using the SPI variant, please refer to the <code>spi</code> module.
//! </div>
//!
//! ## I²C Peripheral Addressing
//!
//! The ACS37800 has a flexible I²C addressing scheme that allows it to operate with any valid 7-bit
//! I²C address. From the factory, the device is configured to measure the analog voltage levels on
//! its two DIO pins during power-on to determine its I²C address.
//!
//! The base I²C address for the ACS37800 is `0x60`. The final I²C address is calculated by adding
//! an offset derived from the voltage levels on the DIO pins to this base address.
//!
//! Alternatively, if both DIO pins are pulled to `Vcc` during power-on, the device uses the I²C
//! address stored in its EEPROM. The default factory-programmed address is `0x7F`.
//!
//! ### I²C Address Calculation
//!
//! The voltage levels on each DIO pin are converted into a 2-bit binary value scaled to `Vcc`. The
//! resulting 2-bit values from `DIO_0` and `DIO_1` are concatenated to form a 4-bit address offset.
//! This offset is then added to the base I²C address `0x60` to determine the final I²C address of
//! the device.
//!
//! <div style="border-left: 4px solid #812ae4ff; padding: 0.5em 1em;">
//! <strong>IMPORTANT</strong><br/>
//! If the DIO pins are left floating, their state is undefined, which may lead to unpredictable I²C
//! address selection. It is recommended to either set the DIO pins to known voltage levels or
//! disable DIO-based address selection by setting the <code>I2C_DIS_SLV_ADDR</code> bit.
//! </div>
//!
//! | `DIO_0` Voltage | `DIO_1` Voltage | `DIO_0` Value | `DIO_1` Value | Address Offset | Final I²C Address |
//! |-----------------|-----------------|---------------|---------------|----------------|-------------------|
//! | `GND`           | `GND`           | 0             | 0             | `0b0000`       | `0x60`            |
//! | `GND`           | 1 × `Vcc` / 3   | 0             | 1             | `0b0001`       | `0x61`            |
//! | `GND`           | 2 × `Vcc` / 3   | 0             | 2             | `0b0010`       | `0x62`            |
//! | `GND`           | `Vcc`           | 0             | 3             | `0b0011`       | `0x63`            |
//! | 1 × `Vcc` / 3   | `GND`           | 1             | 0             | `0b0100`       | `0x64`            |
//! | 1 × `Vcc` / 3   | 1 × `Vcc` / 3   | 1             | 1             | `0b0101`       | `0x65`            |
//! | 1 × `Vcc` / 3   | 2 × `Vcc` / 3   | 1             | 2             | `0b0110`       | `0x66`            |
//! | 1 × `Vcc` / 3   | `Vcc`           | 1             | 3             | `0b0111`       | `0x67`            |
//! | 2 × `Vcc` / 3   | `GND`           | 2             | 0             | `0b1000`       | `0x68`            |
//! | 2 × `Vcc` / 3   | 1 × `Vcc` / 3   | 2             | 1             | `0b1001`       | `0x69`            |
//! | 2 × `Vcc` / 3   | 2 × `Vcc` / 3   | 2             | 2             | `0b1010`       | `0x6A`            |
//! | 2 × `Vcc` / 3   | `Vcc`           | 2             | 3             | `0b1011`       | `0x6B`            |
//! | `Vcc`           | `GND`           | 3             | 0             | `0b1100`       | `0x6C`            |
//! | `Vcc`           | 1 × `Vcc` / 3   | 3             | 1             | `0b1101`       | `0x6D`            |
//! | `Vcc`           | 2 × `Vcc` / 3   | 3             | 2             | `0b1110`       | `0x6E`            |
//! | `Vcc`           | `Vcc`           | 3             | 3             | n/a            | From EEPROM       |

use bon::Builder;

#[cfg(feature = "async")]
use embedded_hal_async::i2c::I2c;

#[cfg(not(feature = "async"))]
use embedded_hal::i2c::I2c;

use super::{Acs37800, Acs37800EepromRegister, Acs37800ReadError};

/// ## Default I²C base address for DIO pin voltage addressing.
///
/// This is the base address used when the DIO voltage levels are used for addressing.
///
/// Address calculation behaviour is described in the [module-level documentation](crate::i2c).
///
/// ### See Also
/// - [`I2C_ADDRESS_PROGRAMMED_DEFAULT`]
///
/// ### References
/// - [ACS37800 Datasheet, rev 4, page 31](https://www.allegromicro.com/-/media/files/datasheets/acs37800-datasheet.pdf)
pub const I2C_ADDRESS_MEASURED_BASE: u8 = 0x60;

/// ## Default I²C address programmed in the EEPROM at the factory.
///
/// If both DIO pins are pulled to `Vcc` during power-on, the device uses the I²C address stored in
/// its EEPROM. The default factory-programmed address is `0x7F` (127 in decimal), but it can be
/// reprogrammed.
///
/// ### See Also
/// - [`I2C_ADDRESS_MEASURED_BASE`]
///
/// ### References
/// - [ACS37800 Datasheet, rev 4, page 31](https://www.allegromicro.com/-/media/files/datasheets/acs37800-datasheet.pdf)
pub const I2C_ADDRESS_PROGRAMMED_DEFAULT: u8 = 0x7F;

#[derive(Builder)]
pub struct Acs37800I2c<I2C: I2c> {
    i2c: I2C,
    #[builder(default = 0x60)]
    address: u8,
}

impl<I2C: I2c> Acs37800 for Acs37800I2c<I2C> {
    #[cfg(feature = "async")]
    fn read_reg32(
        &mut self,
        reg: Acs37800EepromRegister,
    ) -> impl Future<Output = Result<u32, Acs37800ReadError>> {
        async move {
            let mut buf = [0u8; 4];

            let result = self
                .i2c
                .write_read(self.address, &[reg as u8], &mut buf)
                .await;

            #[cfg(feature = "std")]
            {
                result.map_err(|cause| Acs37800ReadError::Io(format!("{cause:?}")))?;
            }

            #[cfg(not(feature = "std"))]
            {
                result.map_err(|_| Acs37800ReadError::Io)?;
            }

            Ok(u32::from_le_bytes(buf))
        }
    }

    #[cfg(not(feature = "async"))]
    fn read_reg32(&mut self, reg: Acs37800EepromRegister) -> Result<u32, Acs37800ReadError> {
        let mut buf = [0u8; 4];

        let result = self.i2c.write_read(self.address, &[reg as u8], &mut buf);

        #[cfg(feature = "std")]
        {
            result.map_err(|cause| Acs37800ReadError::Io(format!("{cause:?}")))?;
        }

        #[cfg(not(feature = "std"))]
        {
            result.map_err(|_| Acs37800ReadError::Io)?;
        }

        Ok(u32::from_le_bytes(buf))
    }
}

#[cfg(all(test, not(feature = "async")))]
mod tests {
    use embedded_hal::i2c::ErrorKind;
    use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    use crate::test::assert_is_bus_error;

    use super::*;

    fn new_driver(expectations: &[I2cTransaction]) -> Acs37800I2c<I2cMock> {
        let i2c = I2cMock::new(expectations);
        Acs37800I2c::builder().i2c(i2c).address(0x60).build()
    }

    #[test]
    fn read_reg32_returns_le_word() {
        let expectations = [I2cTransaction::write_read(
            0x60,
            vec![Acs37800EepromRegister::R0C as u8],
            vec![0x78, 0x56, 0x34, 0x12],
        )];
        let mut driver = new_driver(&expectations);

        let value = driver
            .read_reg32(Acs37800EepromRegister::R0C)
            .expect("read value");
        assert_eq!(value, 0x1234_5678);

        driver.i2c.done();
    }

    #[test]
    fn read_reg32_maps_bus_errors() {
        let expectations =
            [
                I2cTransaction::write_read(
                    0x60,
                    vec![Acs37800EepromRegister::R0B as u8],
                    vec![0; 4],
                )
                .with_error(ErrorKind::Other),
            ];
        let mut driver = new_driver(&expectations);

        let err = driver
            .read_reg32(Acs37800EepromRegister::R0B)
            .expect_err("should propagate error");
        assert_is_bus_error(&err);

        driver.i2c.done();
    }
}

#[cfg(all(test, feature = "async"))]
mod async_tests {
    use embedded_hal::i2c::ErrorKind;
    use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    use crate::test::assert_is_bus_error;

    use super::*;

    fn new_driver(expectations: &[I2cTransaction]) -> Acs37800I2c<I2cMock> {
        let i2c = I2cMock::new(expectations);
        Acs37800I2c::builder().i2c(i2c).address(0x60).build()
    }

    #[tokio::test]
    async fn read_reg32_returns_le_word_async() {
        let expectations = [I2cTransaction::write_read(
            0x60,
            vec![Acs37800EepromRegister::R0C as u8],
            vec![0x78, 0x56, 0x34, 0x12],
        )];
        let mut driver = new_driver(&expectations);

        let value = driver
            .read_reg32(Acs37800EepromRegister::R0C)
            .await
            .expect("read value");
        assert_eq!(value, 0x1234_5678);

        driver.i2c.done();
    }

    #[tokio::test]
    async fn read_reg32_maps_bus_errors_async() {
        let expectations =
            [
                I2cTransaction::write_read(
                    0x60,
                    vec![Acs37800EepromRegister::R0B as u8],
                    vec![0; 4],
                )
                .with_error(ErrorKind::Other),
            ];
        let mut driver = new_driver(&expectations);

        let err = driver
            .read_reg32(Acs37800EepromRegister::R0B)
            .await
            .expect_err("should propagate error");
        assert_is_bus_error(&err);

        driver.i2c.done();
    }
}
