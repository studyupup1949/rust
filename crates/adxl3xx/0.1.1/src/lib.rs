//! ADXL375 Accelerometer Driver
//!
//! Using embedded-hal
//! Supports I2C / SPI communications though a RegisterBus Trait
//!
//! Reference: <https://www.analog.com/media/en/technical-documentation/data-sheets/adxl375.pdf>

//! ## Example I2C:
//!
//! ```rust
//! 
//! use adxl3xx as adxl;
//! ...
//!
//! // RP2040 Hal Boilerplate
//! ...
//!
//! // Setting I2C Pins
//! let sda_pin: gpio::Pin<_, gpio::FunctionI2c, _> = pins.gpio4.reconfigure();
//! let scl_pin: gpio::Pin<_, gpio::FunctionI2c, _> = pins.gpio5.reconfigure();
//!
//! // HAL I2C0 Init
//! let i2c = hal::I2C::i2c0(
//!     pac.I2C0,
//!     sda_pin,
//!     scl_pin,
//!     400.kHz(),
//!     &mut pac.RESETS,
//!     &clocks.system_clock,
//! );
//!
//! // Adxl Bus
//! let adxlbus = adxl::AdxlBusI2c {
//!     i2c:  i2c,
//!     addr: adxl::reg::ADXL_ADDR,
//! };
//!
//! // Adxl Device
//! let mut adxl: adxl::Adxl345<_, adxl::G16Adxl345, adxl::FullRes> = match adxl::Adxl345::new(adxlbus) {
//!     Ok(device) => device,
//!     Err(e) => {
//!         println!("Device init error: {:?}", e);
//!         return;
//!     }
//! };
//!
//! // Adxl Init
//! if let Err(e) = adxl.init_defaults() {
//!     println!("Init Err: {:?}", e);
//!     return;
//! }
//!
//! // Adxl Calibrate
//! println!("\n Running Calibration ...");
//! match adxl.calibrate_axis_offsets() {
//!     Ok((x,y,z)) => {
//!         println!("Calibration values | X: {x} | Y: {y} | Z: {z} |\n");
//!     }
//!     Err(e) => {
//!         println!("Err: {:?}", e);
//!     }
//! }
//!
//! // Read Axis in m/s^2
//! match adxl.read_axis() {
//!     Ok((x, y, z)) => println!("Axis | X: {x:6.3} | Y: {y:6.3} | Z: {z:6.3} |"),
//!     Err(e) => println!("Err: {:?}", e),
//! }
//!
//! // Read Axis in raw LSB units
//! match adxl.read_axis_lsb_units() {
//!     Ok((x, y, z)) => println!("Axis | X: {x:6} | Y: {y:6} | Z: {z:6} |"),
//!     Err(e) => println!("Err: {:?}", e),
//! }
//! ```

//! Example SPI Setup:
//!
//! ``` rust
//! 
//! use adxl3xx as adxl;
//! ...
//!
//! // RP2040 Hal Boilerplate
//! ...
//!
//! // ADXL Four wire SPI wiring:
//! // SCL >> CLK
//! // SDA >> MOSI
//! // SDO >> MISO
//! // CS  >> GPIO | Note: For power-up in SPI Mode, the ADXL CS Pin needs to be pulled to GND with a resistor.
//!
//! // SPI Pins
//! let spi_mosi = pins.gpio19.into_function::<hal::gpio::FunctionSpi>();
//! let spi_miso = pins.gpio16.into_function::<hal::gpio::FunctionSpi>();
//! let spi_sclk = pins.gpio18.into_function::<hal::gpio::FunctionSpi>();
//! let spi_cs   = pins.gpio17.into_push_pull_output_in_state(gpio::PinState::High);
//!
//! // HAL SPI Bus Init
//! let spi = hal::spi::Spi::<_, _, _, 8>::new(pac.SPI0, (spi_mosi, spi_miso, spi_sclk));
//! let spi = spi.init(
//!     &mut pac.RESETS,
//!     clocks.peripheral_clock.freq(),
//!     2.MHz(),
//!     embedded_hal::spi::MODE_3,
//! );
//!
//! // SPI Adxl Bus
//! let adxlbus = adxl::AdxlBusSpi {
//!     spi:    spi,
//!     spi_cs: spi_cs,
//! };
//!
//! // Adxl Device
//! let mut adxl: adxl::Adxl345<_, adxl::G16Adxl345, adxl::FullRes> = match adxl::Adxl345::new(adxlbus) {
//!     Ok(device) => device,
//!     Err(e) => {
//!         println!("Device init error: {:?}", e);
//!         return;
//!     }
//! };
//!
//! ...
//! ```

#![no_std]
#![allow(clippy::too_many_arguments)]

pub mod reg;
mod reg_helper;

use core::fmt::{Debug, Display};
use core::result::Result;
use core::marker::PhantomData;

pub use reg_helper::{ErrorReg, RegisterBus};

use embedded_hal::digital::StatefulOutputPin;
use embedded_hal::{i2c, spi};

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Globals
// —————————————————————————————————————————————————————————————————————————————————————————————————

/// Gravity m/s^2
pub const G: f32 = 9.80665; // Gravity m/s^2

/// Temp write buffer for appending reg addr.
pub const MAX_BUF_LEN: usize = 16;

/// Resolution
pub trait Resolution {
    const RESOLUTION_BIT: u8;
    const IS_FULL_RES: bool;
    const STANDARD_RES_BITS: Option<u8>;
}
pub struct FullRes;
impl Resolution for FullRes {
    const RESOLUTION_BIT: u8 = 1;
    const IS_FULL_RES: bool = true;
    const STANDARD_RES_BITS: Option<u8> = None;
}

pub struct TenBitRes;
impl Resolution for TenBitRes {
    const RESOLUTION_BIT: u8 = 0;
    const IS_FULL_RES: bool = false;
    const STANDARD_RES_BITS: Option<u8> = Some(10);
}

/// Justify
pub struct Justify {
    is_left_justified: bool,
}

/// Trait to define the compile-time properties of a G-Range
pub trait GRange {
    const RANGE_BITS: u8;
    const FULL_RES_BITS: u8;
    const G_MAX: f32;
}

pub struct G1_5Adxl312;
pub struct G3Adxl312;
pub struct G6Adxl312;
pub struct G12Adxl312;

pub struct G0_5Adxl313;
pub struct G1Adxl313;
pub struct G2Adxl313;
pub struct G4Adxl313;

pub struct G2Adxl345;
pub struct G4Adxl345;
pub struct G8Adxl345;
pub struct G16Adxl345;

pub struct G200Adxl375;

impl GRange for G1_5Adxl312 {
    const RANGE_BITS: u8 = 0b00;
    const FULL_RES_BITS: u8 = 10;
    const G_MAX: f32 = 1.5;
}
impl GRange for G3Adxl312 {
    const RANGE_BITS: u8 = 0b01;
    const FULL_RES_BITS: u8 = 11;
    const G_MAX: f32 = 3.0;
}
impl GRange for G6Adxl312 {
    const RANGE_BITS: u8 = 0b10;
    const FULL_RES_BITS: u8 = 12;
    const G_MAX: f32 = 6.0;
}
impl GRange for G12Adxl312 {
    const RANGE_BITS: u8 = 0b11;
    const FULL_RES_BITS: u8 = 13;
    const G_MAX: f32 = 12.0;
}

impl GRange for G0_5Adxl313 {
    const RANGE_BITS: u8 = 0b00;
    const FULL_RES_BITS: u8 = 10;
    const G_MAX: f32 = 0.5;
}
impl GRange for G1Adxl313 {
    const RANGE_BITS: u8 = 0b01;
    const FULL_RES_BITS: u8 = 11;
    const G_MAX: f32 = 1.0;
}
impl GRange for G2Adxl313 {
    const RANGE_BITS: u8 = 0b10;
    const FULL_RES_BITS: u8 = 12;
    const G_MAX: f32 = 2.0;
}
impl GRange for G4Adxl313 {
    const RANGE_BITS: u8 = 0b11;
    const FULL_RES_BITS: u8 = 13;
    const G_MAX: f32 = 4.0;
}

impl GRange for G2Adxl345 {
    const RANGE_BITS: u8 = 0b00;
    const FULL_RES_BITS: u8 = 10;
    const G_MAX: f32 = 2.0;
}
impl GRange for G4Adxl345 {
    const RANGE_BITS: u8 = 0b01;
    const FULL_RES_BITS: u8 = 11;
    const G_MAX: f32 = 4.0;
}
impl GRange for G8Adxl345 {
    const RANGE_BITS: u8 = 0b10;
    const FULL_RES_BITS: u8 = 12;
    const G_MAX: f32 = 8.0;
}
impl GRange for G16Adxl345 {
    const RANGE_BITS: u8 = 0b11;
    const FULL_RES_BITS: u8 = 13;
    const G_MAX: f32 = 16.0;
}

impl GRange for G200Adxl375 {
    const RANGE_BITS: u8 = 0b11;
    const FULL_RES_BITS: u8 = 13;
    const G_MAX: f32 = 200.0;
}

/// Wakeup Bits
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Wakeup {
    Hz8 = 0b_00,
    Hz4 = 0b_01,
    Hz2 = 0b_10,
    Hz1 = 0b_11,
}

/// Fifo Mode
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum FifoMode {
    Bypass  = 0b_00,
    Fifo    = 0b_01,
    Stream  = 0b_10,
    Trigger = 0b_11,
}

/// Data Rate
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Rate {
    R3200 = 0b_1111,
    R1600 = 0b_1110,
    R800  = 0b_1101,
    R400  = 0b_1100,
    R200  = 0b_1011,
    R100  = 0b_1010,
    R50   = 0b_1001,
    R25   = 0b_1000,
    R12_5 = 0b_0111,
    R6_25 = 0b_0110,
    R3_13 = 0b_0101,
    R1_56 = 0b_0100,
    R0_78 = 0b_0011,
    R0_39 = 0b_0010,
    R0_20 = 0b_0001,
    R0_10 = 0b_0000,
}

/// Interrupt Source
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IntSource {
    pub data_ready: bool,
    pub single_tap: bool,
    pub double_tap: bool,
    pub activity:   bool,
    pub inactivity: bool,
    pub free_fall:  bool,
    pub watermark:  bool,
    pub overrun:    bool,
}

pub type AdxlResult<T, B> = Result<T, Error<ErrorReg<<B as RegisterBus>::Error>>>;

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                              Adxl
// —————————————————————————————————————————————————————————————————————————————————————————————————

pub struct AdxlBusI2c<I2C> {
    pub i2c:  I2C,
    pub addr: u8,
}

pub struct AdxlBusSpi<SPI, CS> {
    pub spi:    SPI,
    pub spi_cs: CS,
}

pub type Adxl312<B, R = G12Adxl312, Res = FullRes> = Adxl3xx<IC312, B, R, Res>;
pub type Adxl313<B, R = G4Adxl313, Res = FullRes> = Adxl3xx<IC313, B, R, Res>;
pub type Adxl343<B, R = G16Adxl345, Res = FullRes> = Adxl3xx<IC343, B, R, Res>;
pub type Adxl344<B, R = G16Adxl345, Res = FullRes> = Adxl3xx<IC344, B, R, Res>;
pub type Adxl345<B, R = G16Adxl345, Res = FullRes> = Adxl3xx<IC345, B, R, Res>;
pub type Adxl346<B, R = G16Adxl345, Res = FullRes> = Adxl3xx<IC346, B, R, Res>;
pub type Adxl375<B> = Adxl3xx<IC375, B, G200Adxl375, FullRes>;
pub type Adxl314<B> = Adxl3xx<IC314, B, G12Adxl312, FullRes>;

// Marker types for the hardware models
pub struct IC312;
pub struct IC313;
pub struct IC345;
pub type IC343 = IC345; // ADXL343 is fully intercompatible with ADXL345, so I'm just aliasing it here.
pub struct IC346;
pub type IC344 = IC346;
pub struct IC375;
pub type IC314 = IC375; // basically just an automotive-rated clone

pub trait AdxlModel {
    const ID: u8;
    const G_MAX: f32;
    const BITS_PER_AXIS_MAX: u8;
    const CALIBRATION_SCALE: u8;
}

impl AdxlModel for IC312 { const ID: u8 = 0xE5; const G_MAX: f32 = 12.0; const BITS_PER_AXIS_MAX: u8 = 13; const CALIBRATION_SCALE: u8 = 4; }
impl AdxlModel for IC313 { const ID: u8 = 0xCB; const G_MAX: f32 = 4.0; const BITS_PER_AXIS_MAX: u8 = 13; const CALIBRATION_SCALE: u8 = 4; }
impl AdxlModel for IC345 { const ID: u8 = 0xE5; const G_MAX: f32 = 16.0; const BITS_PER_AXIS_MAX: u8 = 13; const CALIBRATION_SCALE: u8 = 4; }
impl AdxlModel for IC346 { const ID: u8 = 0xE6; const G_MAX: f32 = 16.0; const BITS_PER_AXIS_MAX: u8 = 13; const CALIBRATION_SCALE: u8 = 4; }
impl AdxlModel for IC375 { const ID: u8 = 0xE5; const G_MAX: f32 = 200.0; const BITS_PER_AXIS_MAX: u8 = 13; const CALIBRATION_SCALE: u8 = 32; }

/// If a type implements this, that specific combination is legal.
pub trait AdxlConfig<R: GRange, Res: Resolution>: AdxlModel {}

impl<Res: Resolution> AdxlConfig<G1_5Adxl312, Res> for IC312 {}
impl<Res: Resolution> AdxlConfig<G3Adxl312, Res> for IC312 {}
impl<Res: Resolution> AdxlConfig<G6Adxl312, Res> for IC312 {}
impl<Res: Resolution> AdxlConfig<G12Adxl312, Res> for IC312 {}

impl<Res: Resolution> AdxlConfig<G0_5Adxl313, Res> for IC313 {}
impl<Res: Resolution> AdxlConfig<G1Adxl313, Res> for IC313 {}
impl<Res: Resolution> AdxlConfig<G2Adxl313, Res> for IC313 {}
impl<Res: Resolution> AdxlConfig<G4Adxl313, Res> for IC313 {}

// The ADXL345 allows any resolution with its specific ranges
impl<Res: Resolution> AdxlConfig<G2Adxl345, Res> for IC345 {}
impl<Res: Resolution> AdxlConfig<G4Adxl345, Res> for IC345 {}
impl<Res: Resolution> AdxlConfig<G8Adxl345, Res> for IC345 {}
impl<Res: Resolution> AdxlConfig<G16Adxl345, Res> for IC345 {}

// The ADXL346 is the same as the ADXL345, but with a different Device ID
impl<Res: Resolution> AdxlConfig<G2Adxl345, Res> for IC346 {}
impl<Res: Resolution> AdxlConfig<G4Adxl345, Res> for IC346 {}
impl<Res: Resolution> AdxlConfig<G8Adxl345, Res> for IC346 {}
impl<Res: Resolution> AdxlConfig<G16Adxl345, Res> for IC346 {}

// The ADXL375 ONLY allows G200 and FullRes
impl AdxlConfig<G200Adxl375, FullRes> for IC375 {}

pub struct Adxl3xx<Model: AdxlConfig<R, Res>, B: RegisterBus, R: GRange, Res: Resolution = FullRes> {
    bus: B,
    bits_per_axis: u8,
    lsb_per_g: f32,
    is_left_justified: bool,
    _marker: PhantomData<(Model, R, Res)>,
}

impl<Model: AdxlConfig<Range, Res>, Bus: RegisterBus, Range: GRange, Res: Resolution> Adxl3xx<Model, Bus, Range, Res> {
    pub fn new(mut bus: Bus) -> AdxlResult<Self, Bus> {
        let bits_per_axis: u8;
        if Res::IS_FULL_RES {
            bits_per_axis = Range::FULL_RES_BITS;
        } else {
            bits_per_axis = Res::STANDARD_RES_BITS.unwrap_or(10);
        }
        let lsb_per_g =  Self::calculate_lsb_per_g(bits_per_axis, Range::G_MAX);

        // Validate Device ID
        let id = reg::DEVID.read(&mut bus)?;
        if Model::ID != id as u8 {
            return Err(Error::Init);
        }

        let justify_val = reg::DATA_FORMAT.read_field(&mut bus, reg::FL_DF::JUSTIFY)?;
        let is_left_justified = justify_val != 0;

        reg::DATA_FORMAT.write_multiple_fields(&mut bus, &[
            (reg::FL_DF::FULL_RES, Res::RESOLUTION_BIT as u32),
            (reg::FL_DF::RANGE, Range::RANGE_BITS as u32),
        ])?;

        Ok(Self {
            bus,
            bits_per_axis,
            lsb_per_g,
            is_left_justified,
            _marker: PhantomData,
        })
    }

    // —————————————————————————————————————————— Init —————————————————————————————————————————————

    /// Resets the registers to the datasheet values, excluding axes offsets.
    pub fn reset(&mut self) -> AdxlResult<(), Bus> {
        // Reset POWER CTL
        self.write_reg_addr_raw(0x2D, &[0x00])?;
        // Reset THRESH TAP
        self.write_reg_addr_raw(0x1D, &[0x00])?;

        // Reset range registers to 0x00
        for reg in 0x21..=0x2B {
            self.write_reg_addr_raw(reg, &[0x00])?;
        }

        // BW_RATE set to 0x0A (100Hz)
        self.write_reg_addr_raw(0x2C, &[0x0A])?;

        // Remaining registers
        for reg in 0x2E..=0x31 {
            self.write_reg_addr_raw(reg, &[0x00])?;
        }

        // Restore configured format settings
        reg::DATA_FORMAT.write_multiple_fields(&mut self.bus, &[
            (reg::FL_DF::FULL_RES, Res::RESOLUTION_BIT as u32),
            (reg::FL_DF::RANGE, Range::RANGE_BITS as u32),
        ])?;

        // Set Right Justify
        self.set_left_justify(false)?;

        // FIFO_CTL
        self.write_reg_addr_raw(0x38, &[0x00])?;
        Ok(())
    }

    /// Initalize some sane defaults and validates the device id.
    pub fn init_defaults(&mut self) -> AdxlResult<(), Bus> {
        self.reset()?;

        // Fifo:  Stream | Trigger 0 | Watermark Samples
        self.set_fifo_ctl(FifoMode::Stream, false, 24)?;

        // Set Rate
        self.set_rate(Rate::R800)?;

        // Start measuring
        self.set_measure(true)?;

        Ok(())
    }

    /// Sets the justification mode of the accelerometer to left-justified or right-justified
    pub fn set_left_justify(&mut self, left_justify: bool) -> AdxlResult<(), Bus> {
        self.is_left_justified = left_justify;
        reg::DATA_FORMAT.write_field(&mut self.bus,
                                     reg::FL_DF::JUSTIFY,
                                     self.is_left_justified as u32
        )?;
        Ok(())
    }

    /// Change the base configuration of the accelerometer
    pub fn into_format<NewRes: Resolution, NewRange: GRange>(self) -> AdxlResult<Adxl3xx<Model, Bus, NewRange, NewRes>, Bus>
        where Model: AdxlConfig<NewRange, NewRes> {
        Adxl3xx::new(self.bus)
    }

    // ——————————————————————————————————————————— Raw —————————————————————————————————————————————

    /// Raw register read
    pub fn read_reg_addr_raw(&mut self, reg_addr: u8, buf: &mut [u8]) -> AdxlResult<(), Bus> {
        self.bus
            .read_register(reg_addr, buf)
            .map_err(ErrorReg::Bus)?;
        Ok(())
    }

    /// Raw register write    
    pub fn write_reg_addr_raw(&mut self, reg_addr: u8, buf: &[u8]) -> AdxlResult<(), Bus> {
        self.bus
            .write_register(reg_addr, buf)
            .map_err(ErrorReg::Bus)?;
        Ok(())
    }

    // —————————————————————————————————————————— Read —————————————————————————————————————————————

    /// Device ID
    pub fn read_device_id(&mut self) -> AdxlResult<u8, Bus> {
        Ok(reg::DEVID.read(&mut self.bus)? as u8)
    }

    /// Fifo Status: Trigger
    pub fn read_fifo_status_trig(&mut self) -> AdxlResult<u8, Bus> {
        Ok(reg::FIFO_STATUS.read_field(&mut self.bus, reg::FL_FS::FIFO_TRIG)? as u8)
    }

    /// Fifo Status: Num of samples available in the FIFO buffer
    pub fn read_fifo_status_entries(&mut self) -> AdxlResult<u8, Bus> {
        Ok(reg::FIFO_STATUS.read_field(&mut self.bus, reg::FL_FS::ENTRIES)? as u8)
    }

    /// Read XYZ Axis data as raw byte buffer
    #[inline]
    pub fn read_axis_raw_buf(&mut self) -> AdxlResult<[u8; 6], Bus> {
        let mut buf = [0_u8; 6];
        reg::DATAX0.read_raw_buffer(&mut self.bus, &mut buf)?;

        Ok(buf)
    }

    /// Read XYZ Axis as a tuple of i16 values in LSB units
    pub fn read_axis_lsb_units(&mut self) -> AdxlResult<(i16, i16, i16), Bus> {
        // Burst read 6 bytes starting from DATAX0
        let buf = self.read_axis_raw_buf()?;

        // Extracting data
        let mut x = i16::from_le_bytes([buf[0], buf[1]]);
        let mut y = i16::from_le_bytes([buf[2], buf[3]]);
        let mut z = i16::from_le_bytes([buf[4], buf[5]]);

        // Handle left justified data format
        if self.is_left_justified {
            let shift: u8 = 16 - self.bits_per_axis;

            x >>= shift;
            y >>= shift;
            z >>= shift;
        }

        Ok((x, y, z))
    }

    /// Read XYZ Axis as a tuple of f32 values as Acceleration units in m/s^2
    pub fn read_axis(&mut self) -> AdxlResult<(f32, f32, f32), Bus> {
        let data = self.read_axis_lsb_units()?;

        // Convert to m/s²
        Ok(self.convert_lsb_to_accel(data))
    }

    /// Convert from LSB to Acceleration units in m/s^s
    #[inline]
    pub fn convert_lsb_to_accel(&self, data: (i16, i16, i16)) -> (f32, f32, f32) {
        // Convert to m/s²
        (
            data.0 as f32 / self.lsb_per_g * G,
            data.1 as f32 / self.lsb_per_g * G,
            data.2 as f32 / self.lsb_per_g * G,
        )
    }

    /// Read ACT_TAP_STATUS: Asleep
    pub fn is_asleep(&mut self) -> AdxlResult<bool, Bus> {
        Ok(reg::ACT_TAP_STATUS.read_field(&mut self.bus, reg::FL_ATS::ASLEEP)? as u8 != 0)
    }

    /// Read ACT_TAP_STATUS: ACT (X Y Z)
    pub fn read_act_source_status(&mut self) -> AdxlResult<(bool, bool, bool), Bus> {
        let mut buf = [0_u32; 3];

        reg::ACT_TAP_STATUS.read_multiple_fields(
            &mut self.bus,
            &[
                reg::FL_ATS::ACT_X_SRC,
                reg::FL_ATS::ACT_Y_SRC,
                reg::FL_ATS::ACT_Z_SRC,
            ],
            &mut buf,
        )?;

        let (x, y, z) = (buf[0] != 0, buf[1] != 0, buf[2] != 0);

        Ok((x, y, z))
    }

    /// Read ACT_TAP_STATUS: TAP (X Y Z)
    pub fn read_tap_source_status(&mut self) -> AdxlResult<(bool, bool, bool), Bus> {
        let mut buf = [0_u32; 3];

        reg::ACT_TAP_STATUS.read_multiple_fields(
            &mut self.bus,
            &[
                reg::FL_ATS::TAP_X_SRC,
                reg::FL_ATS::TAP_Y_SRC,
                reg::FL_ATS::TAP_Z_SRC,
            ],
            &mut buf,
        )?;

        let (x, y, z) = (buf[0] != 0, buf[1] != 0, buf[2] != 0);

        Ok((x, y, z))
    }

    #[rustfmt::skip]
    /// Read INT_SOURCE:
    /// (data_ready, single_tap, double_tap, activity, inactivity, free_fall, watermark, overrun)
    /// Interrupts are cleared once reading this register, hence why they are provided all at once!
    /// Output is a IntSource struct holding the values of the respective fields
    pub fn read_int_source(&mut self) -> AdxlResult<IntSource, Bus> {
        let value = reg::INT_SOURCE.read(&mut self.bus)? as u8;

        let data_ready = (value & 0b_1000_0000) != 0;
        let single_tap = (value & 0b_0100_0000) != 0;
        let double_tap = (value & 0b_0010_0000) != 0;
        let activity   = (value & 0b_0001_0000) != 0;
        let inactivity = (value & 0b_0000_1000) != 0;
        let free_fall  = (value & 0b_0000_0100) != 0;
        let watermark  = (value & 0b_0000_0010) != 0;
        let overrun    = (value & 0b_0000_0001) != 0;

        Ok(IntSource {
            data_ready,
            single_tap,
            double_tap,
            activity,
            inactivity,
            free_fall,
            watermark,
            overrun,
        })
    }

    // ——————————————————————————————————————————— Set —————————————————————————————————————————————

    /// MEASURE flag has to be set to 1 to start collecting data
    pub fn set_measure(&mut self, on: bool) -> AdxlResult<(), Bus> {
        reg::POWER_CTL.write_field(&mut self.bus, reg::FL_PC::MEASURE, on as u32)?;
        Ok(())
    }

    // #[rustfmt::skip]
    /// Set POWER CTL register
    pub fn set_power_ctl(
        &mut self,
        link: bool,
        auto_sleep: bool,
        measure: bool,
        sleep: bool,
        wakeup: Wakeup,
    ) -> AdxlResult<(), Bus> {
        let value = (link as u8) << 5
            | (auto_sleep as u8) << 4
            | (measure as u8) << 3
            | (sleep as u8) << 2
            | (wakeup as u8);

        reg::POWER_CTL.write(&mut self.bus, value as u32)?;
        Ok(())
    }

    /// Set BW_RATE - Low Power mode
    pub fn set_low_power(&mut self, on: bool) -> AdxlResult<(), Bus> {
        reg::BW_RATE.write_field(&mut self.bus, reg::FL_BR::LOW_POWER, on as u32)?;
        Ok(())
    }

    /// Set BW_RATE - RATE Code
    pub fn set_rate(&mut self, rate: Rate) -> AdxlResult<(), Bus> {
        reg::BW_RATE.write_field(&mut self.bus, reg::FL_BR::RATE, rate as u32)?;
        Ok(())
    }

    /// Start or stop DATA_FORMAT - SELF TEST
    pub fn set_self_test(&mut self, on: bool) -> AdxlResult<(), Bus> {
        reg::DATA_FORMAT.write_field(&mut self.bus, reg::FL_DF::SELF_TEST, on as u32)?;
        Ok(())
    }

    /// Set SPI wire mode: three wires (true) / four wires(false - default)
    pub fn set_spi_wire_mode(&mut self, three_wire_mode: bool) -> AdxlResult<(), Bus> {
        reg::DATA_FORMAT.write_field(&mut self.bus, reg::FL_DF::SPI, three_wire_mode as u32)?;
        Ok(())
    }

    /// Set FIFO CTL register
    pub fn set_fifo_ctl(
        &mut self,
        fifo_mode: FifoMode,
        trigger: bool,
        watermark_samples_num: u8,
    ) -> AdxlResult<(), Bus> {
        let watermark_samples_num = watermark_samples_num.min(31);

        reg::FIFO_CTL.write_multiple_fields(&mut self.bus, &[
            (reg::FL_FC::FIFO_MODE, fifo_mode as u32),
            (reg::FL_FC::TRIGGER, trigger as u32),
            (reg::FL_FC::SAMPLES, watermark_samples_num as u32),
        ])?;

        Ok(())
    }

    /// Calibrates the built-in axis offsets.
    /// Returns the calibration values for reference.
    /// Runs `init_defaults` after completion.
    ///
    /// Procedure: Place the ADXL level with the Z axis oriented downwards and run this function to calibrate.
    pub fn calibrate_axis_offsets(&mut self) -> AdxlResult<(i8, i8, i8), Bus> {
        // Init
        self.set_measure(false)?;
        self.clear_axis_offsets()?;
        self.set_fifo_ctl(FifoMode::Stream, false, 24)?;
        self.set_rate(Rate::R400)?;
        reg::DATA_FORMAT.write_multiple_fields(&mut self.bus, &[
            (reg::FL_DF::FULL_RES, FullRes::RESOLUTION_BIT as u32), // full resolution
            (reg::FL_DF::JUSTIFY, true as u32), // right justified
            (reg::FL_DF::RANGE, G16Adxl345::RANGE_BITS as u32), // full range (maps to 0b11, which works on everything)
        ])?;

        // Start measuring
        self.set_measure(true)?;

        // Buffer
        let mut buf = [(0_i16, 0_i16, 0_i16); 32];

        // Waiting for buffer to fill to read the contents
        loop {
            match self.read_fifo_status_entries() {
                Ok(s) if s >= 31 => {
                    // Reading values
                    for el in buf.iter_mut() {
                        *el = self.read_axis_lsb_units()?;
                    }
                    break;
                }
                Err(e) => {
                    return Err(e);
                }
                _ => {
                    continue;
                }
            }
        }

        let lsb_per_g = Self::calculate_lsb_per_g(Model::BITS_PER_AXIS_MAX, Model::G_MAX);
        let scale = Model::CALIBRATION_SCALE as i32;

        // Finding means
        let sum_x: i32 = buf.iter().map(|&(x, ..)| x as i32).sum();
        let sum_y: i32 = buf.iter().map(|&(_, y, _)| y as i32).sum();
        let sum_z: i32 = buf.iter().map(|&(_, _, z)| z as i32).sum();

        let n = buf.len() as i32;
        let m_x = (sum_x + n / 2) / n; // Integer rounding
        let m_y = (sum_y + n / 2) / n;
        let m_z = (sum_z + n / 2) / n;

        let x_off = -(m_x / scale) as i8;
        let y_off = -(m_y / scale) as i8;
        let z_off = -(((m_z as f32) / (scale as f32)) - (lsb_per_g / 4.0)) as i8; // remove 1g
        // lsb_per_g always needs to be divided by 4 in calibration, even for the ADXL375 which uses scale=32.
        // I don't know why, but this works.

        // Writing offsets
        self.set_axis_offsets(x_off, y_off, z_off)?;

        // Restoring Defaults
        self.init_defaults()?;

        Ok((x_off, y_off, z_off))
    }

    #[inline]
    fn calculate_lsb_per_g(total_bits_per_axis: u8, total_g: f32) -> f32 {
        (1 << (total_bits_per_axis - 1)) as f32 / total_g
    }

    /// Set the X Y Z axis offsets
    /// Can run `calibrate_axis_offsets()` for auto offset calibration
    ///
    /// User-set offset adjustments. Signed with a scale
    /// factor of 15.6 mg/LSB (that is, 0x7F = 2 g). The value stored in
    /// the offset registers is automatically added to the acceleration data
    #[inline]
    pub fn set_axis_offsets(&mut self, x: i8, y: i8, z: i8) -> AdxlResult<(), Bus> {
        reg::OFSX.write(&mut self.bus, x as u32)?;
        reg::OFSY.write(&mut self.bus, y as u32)?;
        reg::OFSZ.write(&mut self.bus, z as u32)?;
        Ok(())
    }

    /// Clear the X Y Z axis offsets
    #[inline]
    pub fn clear_axis_offsets(&mut self) -> AdxlResult<(), Bus> {
        self.set_axis_offsets(0, 0, 0)?;
        Ok(())
    }

    /// Set THRESH_TAP. Unsigned with a scale factor of 62.5 mg/LSB (that is, 0xFF = 16 g)
    /// A value of 0 may result in undesirable behavior if
    /// single tap/double tap interrupts are enabled.
    pub fn set_thresh_tap(&mut self, val: u8) -> AdxlResult<(), Bus> {
        reg::THRESH_TAP.write(&mut self.bus, val as u32)?;
        Ok(())
    }

    /// Set DUR.
    /// Unsigned time value representing the maximum time that an event must be
    /// above the THRESH_TAP threshold to qualify as a tap event. The scale factor
    /// is 625 µs/LSB. A value of 0 disables the single tap/ double tap
    /// functions.
    pub fn set_duration(&mut self, val: u8) -> AdxlResult<(), Bus> {
        reg::DUR.write(&mut self.bus, val as u32)?;
        Ok(())
    }

    /// Set LATENT.
    /// Unsigned time value representing the wait time from the detection of a tap event to the
    /// start of the time window (defined by the window register) during
    /// which a possible second tap event can be detected. The scale
    /// factor is 1.25 ms/LSB. A value of 0 disables the double tap function.
    pub fn set_latent(&mut self, val: u8) -> AdxlResult<(), Bus> {
        reg::LATENT.write(&mut self.bus, val as u32)?;
        Ok(())
    }

    /// Set WINDOW.
    /// Unsigned time value representing the amount of time after the expiration of the
    /// latency time (determined by the latent register) during which a
    /// second valid tap can begin. The scale factor is 1.25 ms/LSB. A
    /// value of 0 disables the double tap function.
    /// A value of 0 disables the double tap function.
    pub fn set_window(&mut self, val: u8) -> AdxlResult<(), Bus> {
        reg::WINDOW.write(&mut self.bus, val as u32)?;
        Ok(())
    }

    /// Set THRESH_ACT.
    /// Threshold value for detecting activity.
    /// Unsigned, so the magnitude of the activity event is compared with the value in the
    /// THRESH_ACT register. The scale factor is 62.5 mg/LSB. A value
    /// of 0 may result in undesirable behavior if the activity interrupt is enabled.
    pub fn set_thresh_act(&mut self, val: u8) -> AdxlResult<(), Bus> {
        reg::THRESH_ACT.write(&mut self.bus, val as u32)?;
        Ok(())
    }

    /// Set THRESH_INACT.
    /// Threshold value for detecting inactivity.
    /// Unsigned, so the magnitude of the activity event is compared with the value in the
    /// THRESH_ACT register. The scale factor is 62.5 mg/LSB. A value
    /// of 0 may result in undesirable behavior if the activity interrupt is enabled.
    pub fn set_thresh_inact(&mut self, val: u8) -> AdxlResult<(), Bus> {
        reg::THRESH_INACT.write(&mut self.bus, val as u32)?;
        Ok(())
    }

    /// Set TIME_INACT.
    /// Unsigned time value representing the amount of time that acceleration must
    /// be less than the value in the THRESH_INACT register for inactivity
    /// to be declared. The scale factor is 1 sec/LSB.
    /// The function appears unresponsive if the TIME_INACT register is set to
    /// a value less than the time constant of the output data rate.
    /// A value of 0 results in an interrupt when the output data is less than the value
    /// in the THRESH_INACT register.
    pub fn set_time_inact(&mut self, val: u8) -> AdxlResult<(), Bus> {
        reg::TIME_INACT.write(&mut self.bus, val as u32)?;
        Ok(())
    }

    /// Set ACT_INACT_CTL register
    /// DC - The current acceleration magnitude is compared directly with THRESH_ACT
    /// and THRESH_INACT to determine whether activity or inactivity is detected.
    /// AC - The acceleration value at the start of activity detection is taken as a reference value.
    /// New samples of acceleration are then compared to this reference value.
    ///
    /// ACT/INACT - A setting of 1 enables x-, y-, or z-axis participation in detecting activity or inactivity.
    pub fn set_act_inact_ctl(
        &mut self,
        act_acdc: bool,
        act_x_en: bool,
        act_y_en: bool,
        act_z_en: bool,
        inact_acdc: bool,
        inact_x_en: bool,
        inact_y_en: bool,
        inact_z_en: bool,
    ) -> AdxlResult<(), Bus> {
        let value = (act_acdc as u8) << 7
            | (act_x_en as u8) << 6
            | (act_y_en as u8) << 5
            | (act_z_en as u8) << 4
            | (inact_acdc as u8) << 3
            | (inact_x_en as u8) << 2
            | (inact_y_en as u8) << 1
            | (inact_z_en as u8);

        reg::ACT_INACT_CTL.write(&mut self.bus, value as u32)?;

        Ok(())
    }

    /// Set THRESH_FF
    /// Unsigned format, for free-fall detection. The acceleration
    /// on all axes is compared with the value in THRESH_FF to determine
    /// if a free-fall event occurred. The scale factor is 62.5 mg/LSB.
    /// Values between 300 mg and 600 mg (0x05 to 0x09) are recommended.
    pub fn set_thresh_ff(&mut self, val: u8) -> AdxlResult<(), Bus> {
        reg::THRESH_FF.write(&mut self.bus, val as u32)?;
        Ok(())
    }

    /// Set TIME_FF
    /// Unsigned time value representing the minimum time that the value of all axes
    /// must be less than THRESH_FF to generate a free-fall interrupt.
    /// The scale factor is 5 ms/LSB. Values between 100 / 350 ms (0x14 to 0x46) are recommended.
    pub fn set_time_ff(&mut self, val: u8) -> AdxlResult<(), Bus> {
        reg::TIME_FF.write(&mut self.bus, val as u32)?;
        Ok(())
    }

    /// Set TAP_AXES: Supress
    /// Setting the suppress bit suppresses double tap detection if acceleration greater
    ///  than the value in THRESH_TAP is present between taps.
    pub fn set_tap_supress(&mut self, on: bool) -> AdxlResult<(), Bus> {
        reg::TAP_AXES.write_field(&mut self.bus, reg::FL_TA::SUPPRESS, on as u32)?;
        Ok(())
    }

    /// Set TAP_AXES: TAP X Y Z
    /// A setting of 1 in the TAP_X enable, TAP_Y enable, or TAP_Z
    /// enable bit enables x-, y-, or z-axis participation in tap detection.
    /// A setting of 0 excludes the selected axis from participation in tap detection.
    pub fn set_tap_axes(&mut self, tap_x: bool, tap_y: bool, tap_z: bool) -> AdxlResult<(), Bus> {
        reg::TAP_AXES.write_multiple_fields(&mut self.bus, &[
            (reg::FL_TA::TAP_X_EN, tap_x as u32),
            (reg::FL_TA::TAP_Y_EN, tap_y as u32),
            (reg::FL_TA::TAP_Z_EN, tap_z as u32),
        ])?;

        Ok(())
    }

    #[inline]
    /// Set INT_ENABLE register
    /// Enables their respective functions to generate interrupts
    /// It is recommended that interrupts be configured before enabling their outputs.
    pub fn set_int_enable_direct(&mut self, value: u8) -> AdxlResult<(), Bus> {
        reg::INT_ENABLE.write(&mut self.bus, value as u32)?;

        Ok(())
    }

    /// Set INT_ENABLE register
    /// Enables their respective functions to generate interrupts
    /// It is recommended that interrupts be configured before enabling their outputs.
    pub fn set_int_enable(
        &mut self,
        data_ready: bool,
        single_tap: bool,
        double_tap: bool,
        activity: bool,
        inactivity: bool,
        free_fall: bool,
        watermark: bool,
        overrun: bool,
    ) -> AdxlResult<(), Bus> {
        let value = (data_ready as u8) << 7
            | (single_tap as u8) << 6
            | (double_tap as u8) << 5
            | (activity as u8) << 4
            | (inactivity as u8) << 3
            | (free_fall as u8) << 2
            | (watermark as u8) << 1
            | (overrun as u8);

        reg::INT_ENABLE.write(&mut self.bus, value as u32)?;

        Ok(())
    }

    #[inline]
    /// Set INT_MAP register
    /// Any bits set to 0 in this register send their respective interrupts to
    /// the INT1 pin, whereas bits set to 1 send their respective interrupts
    /// to the INT2 pin. All selected interrupts for a given pin are OR’ed
    pub fn set_int_map_direct(&mut self, value: u8) -> AdxlResult<(), Bus> {
        reg::INT_MAP.write(&mut self.bus, value as u32)?;

        Ok(())
    }

    /// Set INT_MAP register
    /// Any bits set to 0 in this register send their respective interrupts to
    /// the INT1 pin, whereas bits set to 1 send their respective interrupts
    /// to the INT2 pin. All selected interrupts for a given pin are OR’ed
    pub fn set_int_map(
        &mut self,
        data_ready: bool,
        single_tap: bool,
        double_tap: bool,
        activity: bool,
        inactivity: bool,
        free_fall: bool,
        watermark: bool,
        overrun: bool,
    ) -> AdxlResult<(), Bus> {
        let value = (data_ready as u8) << 7
            | (single_tap as u8) << 6
            | (double_tap as u8) << 5
            | (activity as u8) << 4
            | (inactivity as u8) << 3
            | (free_fall as u8) << 2
            | (watermark as u8) << 1
            | (overrun as u8);

        reg::INT_MAP.write(&mut self.bus, value as u32)?;

        Ok(())
    }
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                              Error
// —————————————————————————————————————————————————————————————————————————————————————————————————

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error<E: Debug> {
    Init,
    Config,
    BufferLen,
    Inner(E),
}

impl<E: Debug + Display> Display for Error<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        match &self {
            Error::Init => write!(f, "device id mismatch"),
            Error::Config => write!(f, "configuration error"),
            Error::BufferLen => write!(f, "buffer len error"),
            Error::Inner(e) => Display::fmt(e, f),
        }
    }
}

impl<E: Debug> From<E> for Error<E> {
    fn from(error: E) -> Self {
        Error::Inner(error)
    }
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Traits
// —————————————————————————————————————————————————————————————————————————————————————————————————

// ————————————————————————————————————————————— I2C ———————————————————————————————————————————————

/// I2C RegisterBus implementation
impl<I2C: i2c::I2c> RegisterBus for AdxlBusI2c<I2C> {
    type Error = I2C::Error;

    fn read_register(&mut self, reg_addr: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.i2c.write_read(self.addr, &[reg_addr], buffer)
    }

    fn write_register(&mut self, reg_addr: u8, data: &[u8]) -> Result<(), Self::Error> {
        assert!(data.len() <= MAX_BUF_LEN, "buffer too large");

        let mut buf = [0u8; MAX_BUF_LEN + 1];
        buf[0] = reg_addr;
        buf[1..data.len() + 1].copy_from_slice(data);

        self.i2c.write(self.addr, &buf[..data.len() + 1])
    }
}

// ————————————————————————————————————————————— SPI ———————————————————————————————————————————————

/// SPI RegisterBus implementation
impl<SPI: spi::SpiBus, CS: StatefulOutputPin> RegisterBus for AdxlBusSpi<SPI, CS> {
    type Error = SPI::Error;

    fn read_register(&mut self, reg_addr: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        assert!(buffer.len() <= MAX_BUF_LEN, "buffer too large");

        let addr_byte = if buffer.len() > 1 {
            reg_addr | 0xC0 // multi-byte read
        }
        else {
            reg_addr | 0x80 // single-byte read
        };

        // Temp buffer to store read data
        let mut temp_buf = [0u8; MAX_BUF_LEN + 1];

        self.spi_cs.set_low().expect("SPI CS pin");
        let result = self
            .spi
            .transfer(&mut temp_buf[..buffer.len() + 1], &[addr_byte]);
        self.spi_cs.set_high().expect("SPI CS pin");

        // Copy received data into the user buffer (skip the first byte)
        buffer.copy_from_slice(&temp_buf[1..buffer.len() + 1]);

        result
    }

    fn write_register(&mut self, reg_addr: u8, data: &[u8]) -> Result<(), Self::Error> {
        assert!(data.len() <= MAX_BUF_LEN, "buffer too large");

        let addr_byte = if data.len() > 1 {
            reg_addr | 0x40 // multi-byte write
        }
        else {
            reg_addr // single-byte write
        };

        // Prepare buffer: first byte = address, rest = data
        let mut buf = [0u8; MAX_BUF_LEN + 1];
        buf[0] = addr_byte;
        buf[1..data.len() + 1].copy_from_slice(data);

        self.spi_cs.set_low().expect("SPI CS pin");
        let result = self.spi.write(&buf[..data.len() + 1]);
        self.spi_cs.set_high().expect("SPI CS pin");

        result
    }
}
