//! SPI interface implementation built on top of `embedded-hal` `SpiDevice`.

use embedded_hal::spi::{Operation, SpiDevice};

use super::Adxl372Interface;

/// SPI-based interface implementation for the ADXL372 driver.
pub struct SpiInterface<SPI> {
    spi: SPI,
}

impl<SPI> SpiInterface<SPI> {
    /// Creates a new interface from the provided SPI device abstraction.
    pub const fn new(spi: SPI) -> Self {
        Self { spi }
    }

    /// Builds the command byte used to address registers over SPI.
    fn command_byte(register: u8, is_read: bool) -> u8 {
        let mut command = (register & 0x7F) << 1;
        if is_read {
            command |= 0x01;
        }
        command
    }

    /// Provides mutable access to the wrapped SPI device.
    pub fn spi_mut(&mut self) -> &mut SPI {
        &mut self.spi
    }

    /// Consumes the interface and returns the owned SPI device.
    pub fn release(self) -> SPI {
        self.spi
    }
}

impl<SPI> Adxl372Interface for SpiInterface<SPI>
where
    SPI: SpiDevice,
{
    type Error = SPI::Error;

    fn write_register(&mut self, register: u8, value: u8) -> core::result::Result<(), Self::Error> {
        self.write_many(register, core::slice::from_ref(&value))
    }

    fn read_register(&mut self, register: u8) -> core::result::Result<u8, Self::Error> {
        let mut value = [0u8; 1];
        self.read_many(register, &mut value)?;
        Ok(value[0])
    }

    fn read_many(&mut self, register: u8, buf: &mut [u8]) -> core::result::Result<(), Self::Error> {
        if buf.is_empty() {
            return Ok(());
        }

        let command = [Self::command_byte(register, true)];
        let mut operations = [Operation::Write(&command), Operation::Read(buf)];
        self.spi.transaction(&mut operations)
    }

    fn write_many(&mut self, register: u8, data: &[u8]) -> core::result::Result<(), Self::Error> {
        if data.is_empty() {
            return Ok(());
        }

        let command = [Self::command_byte(register, false)];
        let mut operations = [Operation::Write(&command), Operation::Write(data)];
        self.spi.transaction(&mut operations)
    }
}

#[cfg(test)]
mod tests {
    use super::SpiInterface;
    use crate::interface::Adxl372Interface;
    use core::convert::Infallible;
    use embedded_hal::spi::{ErrorType, Operation, SpiDevice};

    struct MockDevice<'a> {
        expectations: &'a [TransactionExpectation<'a>],
        index: usize,
    }

    impl<'a> MockDevice<'a> {
        fn new(expectations: &'a [TransactionExpectation<'a>]) -> Self {
            Self {
                expectations,
                index: 0,
            }
        }
    }

    impl<'a> Drop for MockDevice<'a> {
        fn drop(&mut self) {
            assert_eq!(
                self.index,
                self.expectations.len(),
                "not all SPI expectations consumed"
            );
        }
    }

    impl<'a> ErrorType for MockDevice<'a> {
        type Error = Infallible;
    }

    impl<'a> SpiDevice for MockDevice<'a> {
        fn transaction<'b>(
            &mut self,
            operations: &mut [Operation<'b, u8>],
        ) -> Result<(), Self::Error> {
            let expected = self
                .expectations
                .get(self.index)
                .expect("unexpected SPI transaction");
            self.index += 1;

            match *expected {
                TransactionExpectation::Read { command, response } => {
                    assert_eq!(operations.len(), 2, "expected write+read operations");
                    let (first, rest) = operations.split_first_mut().expect("missing first op");
                    match first {
                        Operation::Write(data) => {
                            assert_eq!(data.len(), 1, "command length mismatch");
                            assert_eq!(data[0], command, "command byte mismatch");
                        }
                        _ => panic!("first operation must be write"),
                    }

                    let second = rest.first_mut().expect("missing second op");
                    match second {
                        Operation::Read(buf) => {
                            assert_eq!(buf.len(), response.len(), "response length mismatch");
                            buf.copy_from_slice(response);
                        }
                        _ => panic!("second operation must be read"),
                    }
                }
                TransactionExpectation::Write { command, payload } => {
                    assert_eq!(operations.len(), 2, "expected write+write operations");
                    let (first, rest) = operations.split_first_mut().expect("missing first op");
                    match first {
                        Operation::Write(data) => {
                            assert_eq!(data.len(), 1, "command length mismatch");
                            assert_eq!(data[0], command, "command byte mismatch");
                        }
                        _ => panic!("first operation must be write"),
                    }

                    let second = rest.first_mut().expect("missing second op");
                    match second {
                        Operation::Write(data) => {
                            assert_eq!(*data, payload, "payload mismatch");
                        }
                        _ => panic!("second operation must be write"),
                    }
                }
            }

            Ok(())
        }
    }

    #[derive(Clone, Copy)]
    enum TransactionExpectation<'a> {
        Read { command: u8, response: &'a [u8] },
        Write { command: u8, payload: &'a [u8] },
    }

    /// Ensures read_many emits the encoded command and fills the destination buffer.
    #[test]
    fn read_many_transfers_command_and_fills_buffer() {
        let expectations = [TransactionExpectation::Read {
            command: 0x11,
            response: &[0xAA, 0x55],
        }];
        let mock = MockDevice::new(&expectations);
        let mut interface = SpiInterface::new(mock);

        let mut buffer = [0u8; 2];
        interface.read_many(0x08, &mut buffer).unwrap();
        assert_eq!(buffer, [0xAA, 0x55]);
    }

    /// Verifies write_many prefixes payload bytes with the correct command.
    #[test]
    fn write_many_transfers_command_and_payload() {
        let expectations = [TransactionExpectation::Write {
            command: 0x82,
            payload: &[0x12, 0x34, 0x56],
        }];
        let mock = MockDevice::new(&expectations);
        let mut interface = SpiInterface::new(mock);

        interface.write_many(0x41, &[0x12, 0x34, 0x56]).unwrap();
    }

    /// Confirms read_register is implemented via read_many to avoid duplicated logic.
    #[test]
    fn read_register_reuses_read_many() {
        let expectations = [TransactionExpectation::Read {
            command: 0x03,
            response: &[0x5A],
        }];
        let mock = MockDevice::new(&expectations);
        let mut interface = SpiInterface::new(mock);

        let value = interface.read_register(0x01).unwrap();
        assert_eq!(value, 0x5A);
    }

    /// Confirms write_register delegates to write_many so command framing stays consistent.
    #[test]
    fn write_register_reuses_write_many() {
        let expectations = [TransactionExpectation::Write {
            command: 0x02,
            payload: &[0x7E],
        }];
        let mock = MockDevice::new(&expectations);
        let mut interface = SpiInterface::new(mock);

        interface.write_register(0x01, 0x7E).unwrap();
    }

    /// Verifies read_many short-circuits when given an empty slice.
    #[test]
    fn read_many_ignores_empty_buffer() {
        let expectations: [TransactionExpectation; 0] = [];
        let mock = MockDevice::new(&expectations);
        let mut interface = SpiInterface::new(mock);

        interface.read_many(0x08, &mut []).unwrap();
    }

    /// Verifies write_many skips the transaction when payload is empty.
    #[test]
    fn write_many_ignores_empty_payload() {
        let expectations: [TransactionExpectation; 0] = [];
        let mock = MockDevice::new(&expectations);
        let mut interface = SpiInterface::new(mock);

        interface.write_many(0x08, &[]).unwrap();
    }
}
