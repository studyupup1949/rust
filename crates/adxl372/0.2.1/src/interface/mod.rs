//! Bus interface abstraction for the ADXL372 driver.

pub mod spi;

/// Abstraction over the low-level bus access required by the driver.
pub trait Adxl372Interface {
    /// Error type produced by the concrete bus implementation.
    type Error;

    /// Writes a single register.
    fn write_register(&mut self, register: u8, value: u8) -> core::result::Result<(), Self::Error>;

    /// Reads a single register.
    fn read_register(&mut self, register: u8) -> core::result::Result<u8, Self::Error>;

    /// Reads multiple consecutive registers into the provided buffer.
    fn read_many(&mut self, register: u8, buf: &mut [u8]) -> core::result::Result<(), Self::Error>;

    /// Writes multiple consecutive registers from the provided buffer.
    fn write_many(&mut self, register: u8, data: &[u8]) -> core::result::Result<(), Self::Error>;
}
