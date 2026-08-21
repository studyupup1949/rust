#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(all(not(test), not(feature = "std")), no_std)]

use thiserror::Error;

#[cfg(feature = "i2c")]
#[cfg_attr(docsrs, doc(cfg(feature = "i2c")))]
pub mod i2c;

#[cfg(feature = "spi")]
#[cfg_attr(docsrs, doc(cfg(feature = "spi")))]
pub mod spi;

#[cfg(test)]
pub(crate) mod test;

mod eeprom;
pub use eeprom::*;

pub trait Acs37800 {
    #[cfg(feature = "async")]
    fn read_reg32(
        &mut self,
        reg: Acs37800EepromRegister,
    ) -> impl Future<Output = Result<u32, Acs37800ReadError>>;

    #[cfg(not(feature = "async"))]
    fn read_reg32(&mut self, reg: Acs37800EepromRegister) -> Result<u32, Acs37800ReadError>;
}

#[derive(Debug, Error)]
pub enum Acs37800ReadError {
    #[cfg(feature = "std")]
    #[error("Bus communication error: {0}")]
    Io(String),
    #[cfg(not(feature = "std"))]
    #[error("Bus communication error")]
    Io,
}

pub mod prelude {
    pub use crate::Acs37800EepromExt as _;

    #[cfg(feature = "i2c")]
    pub use crate::i2c::Acs37800I2c;
}
