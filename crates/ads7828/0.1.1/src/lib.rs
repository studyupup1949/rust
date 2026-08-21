use embedded_hal::blocking::{delay::DelayMs, i2c::{Read, Write}};

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Channel {
    S0 = 0b10000100,
    S1 = 0b11000100,
    S2 = 0b10010100,
    S3 = 0b11010100,
    S4 = 0b10100100,
    S5 = 0b11100100,
    S6 = 0b10110100,
    S7 = 0b11110100,
}

pub struct Ads7828<I2C, Delay>
where
    I2C: Read + Write,
    Delay: DelayMs<u8>,
{
    i2c: I2C,
    delay: Delay,
    address: u8,
    internal_ref: bool,
    vdd: f32,
}

impl<I2C, E, Delay> Ads7828<I2C, Delay>
where
    I2C: Read<Error = E> + Write<Error = E>,
    Delay: DelayMs<u8>
{
    pub fn new(i2c: I2C, delay: Delay, address: u8, internal_ref: bool, vdd:f32) -> Self {
        Self {
            i2c,
            delay,
            address,
            internal_ref,
            vdd
        }
    }

    pub fn read_channel(&mut self, ch: Channel) -> Result<f32, E> {
        let mut command: u8 = ch as u8;
        command |= (self.internal_ref as u8) << 3;
        self.i2c.write(self.address, &[command])?;
        self.delay.delay_ms(100);
        let mut buf = [0; 2];
        self.i2c.read(self.address, &mut buf)?;
        let val = ((buf[0] as u16) << 8) | buf[1] as u16;
        Ok(self.vdd * ((val as f32) / 4095.0))
    }
}
