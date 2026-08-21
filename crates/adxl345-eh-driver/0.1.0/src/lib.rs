// Credits to https://github.com/adafruit/Adafruit_ADXL345
#![no_std]
use embedded_hal::i2c::{ErrorType, I2c};

pub mod address {
    pub const PRIMARY: u8 = 0x1D;
    pub const SECONDARY: u8 = 0x53;
}

const ID_VAL: u8 = 0b11100101;
const EARTH_GRAVITY: f32 = 9.80665;
const LSB_SCALE_FACTOR_FULL_RES: f32 = 0.0039;

#[derive(Debug)]
pub enum Error<E> {
    IdMismatch,
    BusError(E),
}

impl<E> From<E> for Error<E> {
    fn from(error: E) -> Self {
        Error::BusError(error)
    }
}

pub struct Driver<Bus> {
    bus: Bus,
    addr: u8,
}

impl<Bus> Driver<Bus>
where
    Bus: I2c + ErrorType,
{
    pub fn new(bus: Bus, addr: Option<u8>) -> Result<Driver<Bus>, Error<Bus::Error>> {
        let addr = addr.unwrap_or(address::PRIMARY);
        let mut driver = Driver { bus, addr };
        driver.get_id()?;
        driver.power_on()?;

        return Ok(driver);
    }

    fn get_id(&mut self) -> Result<u8, Error<Bus::Error>> {
        let tx_buf = [registers::DEVID; 1];
        let mut rx_buf = [0u8; 1];
        self.bus.write_read(self.addr, &tx_buf, &mut rx_buf)?;
        if rx_buf[0] != ID_VAL {
            return Err(Error::IdMismatch);
        }

        return Ok(rx_buf[0]);
    }

    fn power_on(&mut self) -> Result<(), Error<Bus::Error>> {
        let tx_buf = [registers::POWER_CTL, 0x08];
        self.bus.write(self.addr, &tx_buf)?;

        return Ok(());
    }

    pub fn get_accel_raw(&mut self) -> Result<(i16, i16, i16), Error<Bus::Error>> {
        let tx_buf = [registers::DATAX0; 1];
        let mut rx_buf = [0u8; 6]; // Read all axes
        self.bus.write_read(self.addr, &tx_buf, &mut rx_buf)?;
        let datax: i16 = ((rx_buf[1] as i16) << 8) | rx_buf[0] as i16;
        let datay: i16 = ((rx_buf[3] as i16) << 8) | rx_buf[2] as i16;
        let dataz: i16 = ((rx_buf[5] as i16) << 8) | rx_buf[4] as i16;

        return Ok((datax, datay, dataz));
    }

    pub fn set_range(&mut self, range: GRange) -> Result<(), Error<Bus::Error>> {
        let mut rx_buf = [0u8; 1];
        self.bus.read(self.addr, &mut rx_buf)?;

        // Clear least significant nibble
        let mut format = rx_buf[0] & 0xF0;
        format |= range as u8;

        // Make sure that the FULL-RES bit is enabled for range scaling
        format |= 0x08;

        self.bus
            .write(self.addr, &[registers::DATA_FORMAT, format])?;

        return Ok(());
    }

    pub fn set_datarate(&mut self, rate: OutputDataRate) -> Result<(), Error<Bus::Error>> {
        let tx_buf = [registers::BW_RATE, rate as u8];
        self.bus.write(self.addr, &tx_buf)?;

        return Ok(());
    }

    pub fn get_accel(&mut self) -> Result<(f32, f32, f32), Error<Bus::Error>> {
        let accel = self.get_accel_raw()?;
        let accel_g: (f32, f32, f32) = (
            (accel.0 as f32) * EARTH_GRAVITY * LSB_SCALE_FACTOR_FULL_RES,
            (accel.1 as f32) * EARTH_GRAVITY * LSB_SCALE_FACTOR_FULL_RES,
            (accel.2 as f32) * EARTH_GRAVITY * LSB_SCALE_FACTOR_FULL_RES,
        );

        return Ok(accel_g);
    }
}

mod registers {
    pub const DEVID: u8 = 0x00; // Device ID
    pub const THRESH_TAP: u8 = 0x1D; // Tap threshold
    pub const OFSX: u8 = 0x1E; // X-axis offset
    pub const OFSY: u8 = 0x1F; // Y-axis offset
    pub const OFSZ: u8 = 0x20; // Z-axis offset
    pub const DUR: u8 = 0x21; // Tap duration
    pub const LATENT: u8 = 0x22; // Tap latency
    pub const WINDOW: u8 = 0x23; // Tap window
    pub const THRESH_ACT: u8 = 0x24; // Activity threshold
    pub const THRESH_INACT: u8 = 0x25; // Inactivity threshold
    pub const TIME_INACT: u8 = 0x26; // Inactivity time
    pub const ACT_INACT_CTL: u8 = 0x27; // Axis enable control for activity and inactivity detection
    pub const THRESH_FF: u8 = 0x28; // Free-fall threshold
    pub const TIME_FF: u8 = 0x29; // Free-fall time
    pub const TAP_AXES: u8 = 0x2A; // Axis control for single/double tap
    pub const ACT_TAP_STATUS: u8 = 0x2B; // Source for single/double tap
    pub const BW_RATE: u8 = 0x2C; // Data rate and power mode control
    pub const POWER_CTL: u8 = 0x2D; // Power-saving features control
    pub const INT_ENABLE: u8 = 0x2E; // Interrupt enable control
    pub const INT_MAP: u8 = 0x2F; // Interrupt mapping control
    pub const INT_SOURCE: u8 = 0x30; // Source of interrupts
    pub const DATA_FORMAT: u8 = 0x31; // Data format control
    pub const DATAX0: u8 = 0x32; // X-axis data 0
    pub const DATAX1: u8 = 0x33; // X-axis data 1
    pub const DATAY0: u8 = 0x34; // Y-axis data 0
    pub const DATAY1: u8 = 0x35; // Y-axis data 1
    pub const DATAZ0: u8 = 0x36; // Z-axis data 0
    pub const DATAZ1: u8 = 0x37; // Z-axis data 1
    pub const FIFO_CTL: u8 = 0x38; // FIFO control
    pub const FIFO_STATUS: u8 = 0x39; // FIFO status
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum GRange {
    Two = 0,     // +/- 2g (default value)
    Four = 1,    // +/- 4g
    Eight = 2,   // +/- 8g
    Sixteen = 3, // +/- 16g
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum OutputDataRate {
    Hz0_10 = 0,
    Hz0_20 = 1,
    Hz0_39 = 2,
    Hz0_78 = 3,
    Hz1_56 = 4,
    Hz3_13 = 5,
    Hz6_25 = 6,
    Hz12_5 = 7,
    Hz25 = 8,
    Hz50 = 9,
    Hz100 = 10,
    Hz200 = 11,
    Hz400 = 12,
    Hz800 = 13,
    Hz1600 = 14,
    Hz3200 = 15,
}
