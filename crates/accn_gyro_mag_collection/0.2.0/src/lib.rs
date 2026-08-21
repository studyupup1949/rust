#![no_std]

// TODO: Auto Range, change the Range either axis reach the max or min

#[cfg(feature = "adxl345")]
pub mod adxl345;

#[cfg(feature = "mpu60x0")]
pub mod mpu60x0;

#[derive(Clone, Copy)]
pub enum Error {
    I2cReadError,
    I2cWriteError,
    I2cRWError,
    SPIError,
    CSLowError,
    CSHighError,
    WrongID,
}

use Error::*;
impl From<Error> for &'static str {
    fn from(value: Error) -> Self {
        match value {
            I2cReadError => "I2c is failed to Read",
            I2cWriteError => "I2c is failed to Write",
            I2cRWError => "I2c is failed to Read/Write",
            SPIError => "SPI Error",
            CSLowError => "CS is failed to Low",
            CSHighError => "CS is failed to High",
            WrongID => "Wrong ID",
        }
    }
}

impl ufmt::uDebug for Error {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        f.write_str(match self {
            I2cReadError => "I2c is failed to Read",
            I2cWriteError => "I2c is failed to Write",
            I2cRWError => "I2c is failed to Read/Write",
            SPIError => "SPI Error",
            CSLowError => "CS is failed to Low",
            CSHighError => "CS is failed to High",
            WrongID => "Wrong ID",
        })?;
        Ok(())
    }
}

impl ufmt::uDisplay for Error {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        f.write_str(match self {
            I2cReadError => "I2c is failed to Read",
            I2cWriteError => "I2c is failed to Write",
            I2cRWError => "I2c is failed to Read/Write",
            SPIError => "SPI Error",
            CSLowError => "CS is failed to Low",
            CSHighError => "CS is failed to High",
            WrongID => "Wrong ID",
        })?;
        Ok(())
    }
}
