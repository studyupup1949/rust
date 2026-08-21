use embedded_hal::blocking::i2c::{Write, Read};

pub struct AdafruitAHT10<I2C> {
    i2c: I2C,
}

#[derive(Debug)]
pub enum Aht10Error {
    CommunicationError,
    CalibrationFailed,
    OtherError,
}

const AHT10_I2CADDR_DEFAULT: u8 = 0x38;
const AHT10_CMD_SOFTRESET: u8 = 0xBA;
const AHT10_CMD_CALIBRATE: u8 = 0xE1;
const AHT10_CMD_TRIGGER: u8 = 0xAC;
const AHT10_STATUS_BUSY: u8 = 0x80;
const AHT10_STATUS_CALIBRATED: u8 = 0x08;

impl<I2C, E> AdafruitAHT10<I2C>
where
    I2C: Write<Error = E> + Read<Error = E>,
{
    pub fn new(i2c: I2C) -> Self {
        AdafruitAHT10 { i2c }
    }

    pub fn begin(&mut self) -> Result<(), Aht10Error> {
        self.soft_reset()?;
        self.calibrate()?;
        Ok(())
    }

    fn soft_reset(&mut self) -> Result<(), Aht10Error> {
        let cmd = [AHT10_CMD_SOFTRESET];
        match self.i2c.write(AHT10_I2CADDR_DEFAULT, &cmd) {
            Ok(_) => {}
            Err(_) => return Err(Aht10Error::CommunicationError),
        }
        Ok(())
    }

    fn calibrate(&mut self) -> Result<(), Aht10Error> {
        let cmd = [AHT10_CMD_CALIBRATE, 0x08, 0x00];
        match self.i2c.write(AHT10_I2CADDR_DEFAULT, &cmd) {
            Ok(_) => {}
            Err(_) => return Err(Aht10Error::CommunicationError),
        }
        while self.get_status()? & AHT10_STATUS_BUSY != 0 {}
        if self.get_status()? & AHT10_STATUS_CALIBRATED == 0 {
            return Err(Aht10Error::CalibrationFailed);
        }
        Ok(())
    }

    fn get_status(&mut self) -> Result<u8, Aht10Error> {
        let mut status = [0u8; 1];
        match self.i2c.read(AHT10_I2CADDR_DEFAULT, &mut status) {
            Ok(_) => {}
            Err(_) => return Err(Aht10Error::CommunicationError),
        }
        Ok(status[0])
    }

    pub fn read_data(&mut self) -> Result<(f32, f32), Aht10Error> {
        let cmd = [AHT10_CMD_TRIGGER, 0x33, 0x00];
        match self.i2c.write(AHT10_I2CADDR_DEFAULT, &cmd) {
            Ok(_) => {}
            Err(_) => return Err(Aht10Error::CommunicationError),
        }

        while self.get_status()? & AHT10_STATUS_BUSY != 0 {}

        let mut data = [0u8; 6];
        match self.i2c.read(AHT10_I2CADDR_DEFAULT, &mut data) {
            Ok(_) => {}
            Err(_) => return Err(Aht10Error::CommunicationError),
        }

        let mut hata: u32 = data[1] as u32;
        hata <<= 8;
        hata |= data[2] as u32;
        hata <<= 4;
        hata |= (data[3] >> 4) as u32;
        let humidity = (hata as f32 * 100.0) / (0x100000 as f32);
        
        let mut tdata: u32 = (data[3] & 0x0F) as u32;
        tdata <<= 8;
        tdata |= data[4] as u32;
        tdata <<= 8;
        tdata |= data[5] as u32;
        let temperature = ((tdata as f32) * 200.0 / 0x100000 as f32) - 50.0;


        Ok((humidity, temperature))
    }
}