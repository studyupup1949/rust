use embedded_hal::i2c::I2c;
use nalgebra::Point3;

pub struct MPU60X0<I2C>
where
    I2C: I2c,
{
    i2c: I2C,
    config: Config,
}
use crate::Error::{self, *};

impl<I2C: I2c> MPU60X0<I2C> {
    pub fn new(i2c: I2C, config: Config) -> Self {
        MPU60X0 { i2c, config }
    }
    pub fn init(&mut self) -> Result<(), Error> {
        // Wack up the device
        self.send_register(0x6B, 0x00)?;

        // Set Accelerometer Range
        self.send_register(0x1C, self.config.afs as u8)?;

        // Set Gyroscope Range
        self.send_register(0x1B, self.config.gfs as u8)?;

        // Set DLPF_CFG
        self.send_register(0x1A, self.config.dlpf as u8)?;

        Ok(())
    }

    fn read_rawaccel(&mut self) -> Result<Point3<i16>, Error> {
        let mut buf = [0u8; 6];

        self.i2c
            .write_read(self.config.addr, &[0x3B | 0x80], &mut buf)
            .map_err(|_| I2cRWError)?;

        let axis = |nth| i16::from_le_bytes([buf[nth + 1], buf[nth]]);
        Ok(Point3::new(axis(0), axis(2), axis(4)))
    }

    pub fn read_accel(&mut self) -> Result<Point3<f32>, Error> {
        let raw = self.read_rawaccel()?;

        let scale_range = self.read_register(0x1C)?;

        let f32point = raw.map(|x| x as f32 * f32::from(AFSsel::from(scale_range)));

        Ok(f32point)
    }

    fn read_rawtemp(&mut self) -> Result<i16, Error> {
        let mut buf = [0u8; 2];

        self.i2c
            .write_read(self.config.addr, &[0x41 | 0x80], &mut buf)
            .map_err(|_| I2cRWError)?;

        buf.reverse();

        Ok(i16::from_le_bytes(buf))
    }

    pub fn read_temp_celsius(&mut self) -> Result<f32, Error> {
        let temp_out = self.read_rawtemp()? as f32;

        Ok((temp_out / 340f32) + 36.53)
    }
    pub fn read_temp_fahrenheit(&mut self) -> Result<f32, Error> {
        let temp_celsius = self.read_temp_celsius()?;
        Ok((temp_celsius * 1.8) + 32f32)
    }
    pub fn read_temp_kelvin(&mut self) -> Result<f32, Error> {
        let temp_celsius = self.read_temp_celsius()?;
        Ok(temp_celsius + 273.15)
    }

    fn read_rawgyro(&mut self) -> Result<Point3<i16>, Error> {
        let mut buf = [0u8; 6];

        self.i2c
            .write_read(self.config.addr, &[0x43 | 0x80], &mut buf)
            .map_err(|_| I2cRWError)?;

        let axis = |nth| i16::from_le_bytes([buf[nth + 1], buf[nth]]);
        Ok(Point3::new(axis(0), axis(2), axis(4)))
    }

    pub fn read_gyro(&mut self) -> Result<Point3<f32>, Error> {
        let raw = self.read_rawgyro()?;

        let scale_range = self.read_register(0x1C)?;

        let f32point = raw.map(|x| x as f32 * f32::from(GFSsel::from(scale_range)));

        Ok(f32point)
    }

    fn send_register(&mut self, reg: u8, value: u8) -> Result<(), Error> {
        self.i2c
            .write(self.config.addr, &[reg, value])
            .map_err(|_| I2cWriteError)?;
        Ok(())
    }

    fn read_register(&mut self, reg: u8) -> Result<u8, Error> {
        let mut buf = [0u8; 1];
        self.i2c
            .write_read(self.config.addr, &[reg | 0x80], &mut buf)
            .map_err(|_| I2cRWError)?;
        Ok(buf[0])
    }
}

pub struct Config {
    pub dlpf: DlpfCfg,
    pub afs: AFSsel,
    pub gfs: GFSsel,
    pub addr: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            addr: 0x68,
            dlpf: Default::default(),
            afs: Default::default(),
            gfs: Default::default(),
        }
    }
}

/**
## Accelerometer Full Scale Selection

- `G2`  ± 2g

- `G4`  ± 4g

- `G8`  ± 8g

- `G16` ± 16g

*/
#[derive(Default, Clone, Copy)]
#[repr(u8)]
pub enum AFSsel {
    #[default]
    G2 = 0b0,
    G4 = 0b00001,
    G8 = 0b00010,
    G16 = 0b00011,
}

impl From<u8> for AFSsel {
    fn from(value: u8) -> Self {
        match value & 0b11 {
            1 => G4,
            2 => G8,
            3 => G16,
            _ => G2,
        }
    }
}

use AFSsel::*;
impl From<AFSsel> for f32 {
    fn from(value: AFSsel) -> Self {
        match value {
            G2 => 1f32 / 16384f32,
            G4 => 1f32 / 8192f32,
            G8 => 1f32 / 4096f32,
            G16 => 1f32 / 2048f32,
        }
    }
}
/**
## Gyroscope Full Scale Selection

- `D250 ` ± 250 °/s

- `D500 ` ± 500 °/s

- `D1000` ± 1000 °/s

- `D2000` ± 2000 °/s

*/

#[derive(Default, Clone, Copy)]
#[repr(u8)]
pub enum GFSsel {
    D250 = 0b0,
    #[default]
    D500 = 0b00001,
    D1000 = 0b00010,
    D2000 = 0b00011,
}

impl From<u8> for GFSsel {
    fn from(value: u8) -> Self {
        match value & 0b11 {
            1 => D500,
            2 => D1000,
            3 => D2000,
            _ => D250,
        }
    }
}

use GFSsel::*;
impl From<GFSsel> for f32 {
    fn from(value: GFSsel) -> Self {
        match value {
            D250 => 1f32 / 250f32,
            D500 => 1f32 / 500f32,
            D1000 => 1f32 / 1000f32,
            D2000 => 1f32 / 2000f32,
        }
    }
}

/**

|  DLPF_CFG | Accel Bandwidth | Delay (ms) | Gyro Bandwidth | Delay (ms) | Fs (kHz) |
| --------- | --------------- | ---------- | -------------- | ---------- | -------- |
| 0         | 260 Hz          | 0 ms       | 256 Hz         | 0.98 ms    | 8 kHz    |
| 1         | 184 Hz          | 2.0 ms     | 188 Hz         | 1.9 ms     | 1 kHz    |
| 2         | 94 Hz           | 3.0 ms     | 98 Hz          | 2.8 ms     | 1 kHz    |
| 3         | 44 Hz           | 4.9 ms     | 42 Hz          | 4.8 ms     | 1 kHz    |
| 4         | 21 Hz           | 8.5 ms     | 20 Hz          | 8.3 ms     | 1 kHz    |
| 5         | 10 Hz           | 13.8 ms    | 10 Hz          | 13.4 ms    | 1 kHz    |
| 6         | 5 Hz            | 19.0 ms    | 5 Hz           | 18.6 ms    | 1 kHz    |
| 7         | RESERVED        | —          | RESERVED       | —          | 8 kHz    |

*/

#[derive(Default, Clone, Copy)]
#[repr(u8)]
pub enum DlpfCfg {
    #[default]
    Bw260Hz = 0, // 8kHz Fs
    Bw184Hz = 1, // 1kHz Fs
    Bw94Hz = 2,
    Bw44Hz = 3,
    Bw21Hz = 4,
    Bw10Hz = 5,
    Bw5Hz = 6,
}
