#![cfg_attr(all(not(test), not(feature = "std")), no_std)]

use bon::Builder;
use thiserror::Error;

#[cfg(feature = "async")]
use embedded_hal_async::i2c::I2c;

#[cfg(not(feature = "async"))]
use embedded_hal::i2c::I2c;

pub mod defaults;

mod eeprom;
pub use eeprom::*;

#[derive(Builder)]
pub struct Acs37800<I2C: I2c> {
    i2c: I2C,
    #[builder(default = 0x60)]
    address: u8,
}

impl<I2C: I2c> Acs37800<I2C> {
    #[cfg(feature = "async")]
    pub async fn read_reg32(&mut self, reg: u8) -> Result<u32, Acs37800ReadError<I2C>> {
        let mut buf = [0u8; 4];
        self.i2c
            .write_read(self.address, &[reg], &mut buf)
            .await
            .map_err(Acs37800ReadError::I2c)?;
        Ok(u32::from_le_bytes(buf))
    }

    #[cfg(not(feature = "async"))]
    pub fn read_reg32(&mut self, reg: u8) -> Result<u32, Acs37800ReadError<I2C>> {
        let mut buf = [0u8; 4];
        self.i2c
            .write_read(self.address, &[reg], &mut buf)
            .map_err(Acs37800ReadError::I2c)?;
        Ok(u32::from_le_bytes(buf))
    }
}

#[derive(Debug, Error)]
pub enum Acs37800ReadError<I2C: I2c> {
    #[error("I2C communication error: {0}")]
    I2c(#[source] I2C::Error),
}
