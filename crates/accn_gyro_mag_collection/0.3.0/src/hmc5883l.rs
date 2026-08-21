use embedded_hal::i2c::I2c;
use nalgebra::Point3;

use crate::Error::{self, *};

pub struct HMC5883L<I2C: I2c> {
    i2c: I2C,
    config: Config,
}

const ADDRS: u8 = 0x1E;

impl<I2C: I2c> HMC5883L<I2C> {
    pub fn new(i2c: I2C, config: Config) -> Self {
        Self { i2c, config }
    }
    pub fn init(&mut self) -> Result<(), Error> {
        // Register Config A
        self.write_register(
            0,
            (self.config.measure_avg as u8) << 5
                | (self.config.data_out_rate as u8) << 2
                | self.config.measure_mode as u8,
        )?;
        // Register Config B
        self.write_register(1, (self.config.range as u8) << 5)?;
        // Mode Register
        self.write_register(
            2,
            (self.config.range as u8) << 5 | (self.config.high_speed as u8).reverse_bits(),
        )?;

        Ok(())
    }
    fn read_magraw(&mut self) -> Result<Point3<i16>, Error> {
        let mut buf = [0u8; 6];

        self.i2c
            .write_read(ADDRS, &[3], &mut buf)
            .map_err(|_| I2cRWError)?;

        Ok(Point3::new(
            i16::from_be_bytes([buf[0], buf[1]]), // X
            i16::from_be_bytes([buf[4], buf[5]]), // Y
            i16::from_be_bytes([buf[2], buf[3]]), // Z (in between!)
        ))
    }
    pub fn read_mag(&mut self) -> Result<Point3<f32>, Error> {
        let raw = self.read_magraw()?;

        let mut buf = [0u8];
        self.i2c
            .write_read(ADDRS, &[1], &mut buf)
            .map_err(|_| I2cRWError)?;
        Ok(raw.map(|x| x as f32 * f32::from(Range::from(buf[0]))))
    }

    fn write_register(&mut self, register: u8, value: u8) -> Result<(), Error> {
        self.i2c
            .write(ADDRS, &[register, value])
            .map_err(|_| I2cWriteError)?;
        Ok(())
    }
}

pub struct Config {
    pub measure_avg: MeasureAverage,
    pub measure_mode: MeasureMode,
    pub data_out_rate: DataOutRate,
    pub range: Range,
    pub operating_mode: OperatingMode,
    pub high_speed: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            high_speed: true,
            measure_avg: Default::default(),
            measure_mode: Default::default(),
            data_out_rate: Default::default(),
            range: Default::default(),
            operating_mode: Default::default(),
        }
    }
}

// Shift Left by 5
#[repr(u8)]
#[derive(Default, Clone, Copy)]
pub enum MeasureAverage {
    #[default]
    One = 0,
    Two = 0b01,
    Four = 0b10,
    Eight = 0b11,
}

// Shift Left by 2
#[repr(u8)]
#[derive(Default, Clone, Copy)]
pub enum DataOutRate {
    Hz0_75 = 0,
    Hz1_5 = 0b001,
    Hz3 = 0b010,
    Hz7_5 = 0b011,
    #[default]
    Hz15 = 0b100,
    Hz30 = 0b101,
    Hz75 = 0b110,
}

#[repr(u8)]
#[derive(Default, Clone, Copy)]
pub enum MeasureMode {
    #[default]
    Normal = 0,
    Positive = 0b01,
    Negative = 0b10,
}

/// Magnetometer gain configuration.
///
/// | GN2 | GN1 | GN0 | Range     | Gain (LSB/Gauss) | Resolution (mG/LSB)  | Output Range               |
/// |-----|-----|-----|-----------|------------------|----------------------|----------------------------|
/// | 0   | 0   | 0   | ±0.88 Ga  | 1370             | 0.73                 | 0xF800–0x07FF (-2048–2047) |
/// | 0   | 0   | 1   | ±1.3 Ga   | 1090 (default)   | 0.92                 | 0xF800–0x07FF (-2048–2047) |
/// | 0   | 1   | 0   | ±1.9 Ga   | 820              | 1.22                 | 0xF800–0x07FF (-2048–2047) |
/// | 0   | 1   | 1   | ±2.5 Ga   | 660              | 1.52                 | 0xF800–0x07FF (-2048–2047) |
/// | 1   | 0   | 0   | ±4.0 Ga   | 440              | 2.27                 | 0xF800–0x07FF (-2048–2047) |
/// | 1   | 0   | 1   | ±4.7 Ga   | 390              | 2.56                 | 0xF800–0x07FF (-2048–2047) |
/// | 1   | 1   | 0   | ±5.6 Ga   | 330              | 3.03                 | 0xF800–0x07FF (-2048–2047) |
/// | 1   | 1   | 1   | ±8.1 Ga   | 230              | 4.35                 | 0xF800–0x07FF (-2048–2047) |
#[repr(u8)]
#[derive(Default, Clone, Copy)]
pub enum Range {
    Gauss0_88 = 0b000,
    #[default]
    Gauss1_3 = 0b001,
    Gauss1_9 = 0b010,
    Gauss2_5 = 0b011,
    Gauss4_0 = 0b100,
    Gauss4_7 = 0b101,
    Gauss5_6 = 0b110,
    Gauss8_1 = 0b111,
}

use Range::*;
impl From<Range> for f32 {
    fn from(value: Range) -> Self {
        match value {
            Gauss0_88 => 0.00073,
            Gauss1_3 => 0.00092,
            Gauss1_9 => 0.00122,
            Gauss2_5 => 0.00152,
            Gauss4_0 => 0.00227,
            Gauss4_7 => 0.00256,
            Gauss5_6 => 0.00303,
            Gauss8_1 => 0.00435,
        }
    }
}

impl From<u8> for Range {
    fn from(value: u8) -> Self {
        match value >> 5 {
            0b001 => Gauss1_3,
            0b010 => Gauss1_9,
            0b011 => Gauss2_5,
            0b100 => Gauss4_0,
            0b101 => Gauss4_7,
            0b110 => Gauss5_6,
            0b111 => Gauss8_1,
            _ => Gauss0_88,
        }
    }
}

#[repr(u8)]
#[derive(Default, Clone, Copy)]
pub enum OperatingMode {
    Continuous = 0,
    #[default]
    Single = 0b01,
    Idle = 0b10,
}
