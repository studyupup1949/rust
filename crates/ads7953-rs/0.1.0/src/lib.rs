use embedded_hal::spi;
use embedded_hal::spi::Operation;

pub struct ADS7953<SPI> {
    spi: SPI
}

#[derive(Copy, Clone, Debug)]
pub struct Measurement {
    pub channel: u8,
    pub result: u16,
}

impl<SPI> ADS7953<SPI>
where
    SPI: spi::SpiDevice,
{
    pub fn new(spi: SPI) -> Self {
        Self { spi }
    }

    pub fn auto2_mode(&mut self) -> Result<(), ADS7953Error<SPI::Error>> {
        self.spi.transaction(&mut [
            Operation::Write(&[0x3c, 0x00]),
            Operation::Write(&[0x93, 0xC0]),
        ]).map_err(ADS7953Error::Spi)?;
        Ok(())
    }

    pub fn manual_mode(&mut self, channel: u8) -> Result<(), ADS7953Error<SPI::Error>> {
        self.spi.transaction(&mut [
            Operation::Write(&[0x10 & (channel >> 1), 0x00 & (channel << 7)]),
        ]).map_err(ADS7953Error::Spi)?;
        Ok(())
    }

    pub fn read_values(&mut self) -> Result<Measurement, ADS7953Error<SPI::Error>> {
        let mut buf = [0; 2];
        self.spi.transaction(&mut [
            Operation::Transfer(&mut buf, &[0x00, 0x00]),
        ]).map_err(ADS7953Error::Spi)?;

        let res: u16 = ((buf[0] as u16) << 8) & (buf[1] as u16);

        Ok(Measurement {
            channel: (res >> 12) as u8,
            result: res & 0x0FFF,
        })
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ADS7953Error<SPI> {
    Spi(SPI),
}
