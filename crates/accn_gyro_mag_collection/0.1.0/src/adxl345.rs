use embedded_hal::{digital::OutputPin, i2c::I2c, spi::SpiBus};
use nalgebra::Point3;

const ADRRS: u8 = 0x53;

pub struct ADXL345<COMM> {
    comm: COMM,
    config: Config,
}

pub struct ADXL345SPI<SPI: SpiBus, CS: OutputPin>(ADXL345<SPI>, CS);

pub struct ADXL345I2c<I2C: I2c>(ADXL345<I2C>, [u8; 6]);

pub struct Config {
    pub range: Range,
    pub data_rate: DataRate,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            range: G8,
            data_rate: DataRate::Hz1600,
        }
    }
}

impl<I2C: I2c> ADXL345I2c<I2C> {
    pub fn new(i2c: I2C, config: Config) -> Self {
        Self(ADXL345 { comm: i2c, config }, [0u8; 6])
    }

    pub fn init(&mut self) -> Result<(), &'static str> {
        // Combine POWER_CTL writes
        self.0
            .comm
            .write(ADRRS, &[0x2d, 0x08])
            .map_err(|_| "Failed to enable measurement mode")?;
        self.0
            .comm
            .write(ADRRS, &[0x2c, self.0.config.data_rate as u8])
            .map_err(|_| "Failed to set data rate")?;
        self.0
            .comm
            .write(ADRRS, &[0x31, self.0.config.range as u8])
            .map_err(|_| "Failed to set data format")?;
        Ok(())
    }
    pub fn read_rawaccel(&mut self) -> Result<Point3<i16>, &'static str> {
        self.0
            .comm
            .write_read(ADRRS, &[0x32 | 0x80], &mut self.1)
            .map_err(|_| "I2C comm E")?;

        Ok(Point3::new(
            i16::from_le_bytes([self.1[0], self.1[1]]),
            i16::from_le_bytes([self.1[2], self.1[3]]),
            i16::from_le_bytes([self.1[4], self.1[5]]),
        ))
    }
    pub fn read_accel(&mut self) -> Result<Acceln, &'static str> {
        let mut range = [0u8];
        self.0
            .comm
            .write_read(ADRRS, &[0x2c | 0x80], &mut range)
            .map_err(|_| "Failed to set data format")?;
        Ok(self
            .read_rawaccel()?
            .map(|x| 9.8 * x as f32 * f32::from(Range::from(range[0]))))
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum DataRate {
    Hz3200 = 0x0F,
    Hz1600 = 0x0E,
    Hz800 = 0x0D,
    Hz400 = 0x0C,
    Hz200 = 0x0B,
    Hz100 = 0x0A,
    Hz50 = 0x09,
    Hz25 = 0x08,
    Hz12_5 = 0x07,
    Hz6_25 = 0x06,
    Hz3_13 = 0x05,
    Hz1_56 = 0x04,
    Hz0_78 = 0x03,
    Hz0_39 = 0x02,
    Hz0_20 = 0x01,
    Hz0_10 = 0x00,
}

// convert Range to f32 for scale factor calculation
use Range::*;
impl From<Range> for f32 {
    fn from(value: Range) -> Self {
        match value {
            G2 => 0.0039,
            G4 => 0.0078,
            G8 => 0.0156,
            G16 => 0.0313,
        }
    }
}

/// WARN: Don't use this, There's some issue with this
impl<SPI: SpiBus, CS: OutputPin> ADXL345SPI<SPI, CS> {
    pub fn new(comm: SPI, cs: CS, config: Config) -> Self {
        ADXL345SPI(ADXL345 { comm, config }, cs)
    }
    pub fn init(&mut self) -> Result<(), &'static str> {
        // Check the device ID
        let mut buffer = [0u8; 2];
        self.read_register(0x00, &mut buffer)?;

        if buffer[1] == 0xe5 {
            // Send the command to set the Power_ctl  addr 0x2d data 0x08
            self.write_register(0x2d, 0x00)?;
            self.write_register(0x2d, 0x08)?;
        } else {
            return Err("Wrong ID");
        }

        // Set the Data Rate / BW_RATE
        self.write_register(0x2c, self.0.config.data_rate as u8)?;

        // Send the command to set the data format addr 0x31, data 0x0b
        self.write_register(0x31, 0x01)?;

        Ok(())
    }

    pub fn read_rawaccel(&mut self) -> Result<Point3<i16>, &'static str> {
        let mut data = [0u8; 6];

        self.read_register(0x32, &mut data)?;

        // if data.iter().sum::<u8>() == 0 {
        //     return Err("No data");
        // }

        Ok(Point3::new(
            i16::from_le_bytes([data[0], data[1]]),
            i16::from_le_bytes([data[2], data[3]]),
            i16::from_le_bytes([data[4], data[5]]),
        ))
    }
    pub fn read_accel(&mut self) -> Result<Acceln, &'static str> {
        let raw = self
            .read_rawaccel()?
            .map(|x| x as f32 * f32::from(self.0.config.range));
        Ok(raw)
    }

    pub fn write_register(&mut self, register: u8, value: u8) -> Result<(), &'static str> {
        self.1.set_low().map_err(|_| "CS set low F")?;
        self.0
            .comm
            .write(&[register | 0x40, value])
            .map_err(|_| "SPI write F")?;
        self.1.set_high().map_err(|_| "CS set high F")?;

        Ok(())
    }

    pub fn read_register(&mut self, register: u8, read: &mut [u8]) -> Result<(), &'static str> {
        self.1.set_low().map_err(|_| "CS set low F")?;
        self.0
            .comm
            .transfer(read, &[register | 0x80 | 0x40])
            .map_err(|_| "SPI write F")?;
        self.1.set_high().map_err(|_| "CS set high F")?;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Range {
    G2 = 0,
    G4 = 1,
    G8 = 2,
    G16 = 3,
}

impl From<u8> for Range {
    fn from(value: u8) -> Self {
        match value & 0b11 {
            1 => Range::G4,
            2 => Range::G8,
            3 => Range::G16,
            _ => Range::G2,
        }
    }
}

pub type Acceln = Point3<f32>;
