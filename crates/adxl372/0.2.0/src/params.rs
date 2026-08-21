//! Strongly typed parameter enumerations for the ADXL372 driver.
//!
//! These enums map directly to datasheet field encodings and are used across
//! [`Config`](crate::config::Config) and the high-level driver APIs. Prefer these
//! types over raw integers to keep configuration values valid and explicit.
//!
//! # Examples
//!
//! ```rust
//! use adxl372::params::{Bandwidth, OutputDataRate, PowerMode};
//!
//! let odr = OutputDataRate::Od6400Hz;
//! let bw = Bandwidth::Bw1600Hz;
//! let mode = PowerMode::Measure;
//! let _ = (odr, bw, mode);
//! ```

use modular_bitfield::prelude::Specifier;

/// Available output data rate (ODR) selections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Specifier)]
#[repr(u8)]
#[bits = 3]
pub enum OutputDataRate {
    /// 400 Hz output data rate.
    Od400Hz = 0b000,
    /// 800 Hz output data rate.
    Od800Hz = 0b001,
    /// 1600 Hz output data rate.
    Od1600Hz = 0b010,
    /// 3200 Hz output data rate.
    Od3200Hz = 0b011,
    /// 6400 Hz output data rate.
    Od6400Hz = 0b100,
}

impl OutputDataRate {
    /// Returns the ODR in hertz as an integer value.
    pub const fn hz(self) -> u32 {
        match self {
            Self::Od400Hz => 400,
            Self::Od800Hz => 800,
            Self::Od1600Hz => 1_600,
            Self::Od3200Hz => 3_200,
            Self::Od6400Hz => 6_400,
        }
    }
}

/// Available analog bandwidth selections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Specifier)]
#[repr(u8)]
#[bits = 3]
pub enum Bandwidth {
    /// 200 Hz bandwidth.
    Bw200Hz = 0b000,
    /// 400 Hz bandwidth.
    Bw400Hz = 0b001,
    /// 800 Hz bandwidth.
    Bw800Hz = 0b010,
    /// 1600 Hz bandwidth.
    Bw1600Hz = 0b011,
    /// 3200 Hz bandwidth.
    Bw3200Hz = 0b100,
}

impl Bandwidth {
    /// Returns the maximum supported frequency in hertz.
    pub const fn max_hz(self) -> u32 {
        match self {
            Self::Bw200Hz => 200,
            Self::Bw400Hz => 400,
            Self::Bw800Hz => 800,
            Self::Bw1600Hz => 1_600,
            Self::Bw3200Hz => 3_200,
        }
    }
}

/// Low-pass filter disable bit (`FILTER_CTL.LPF_DISABLE`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Specifier)]
#[repr(u8)]
#[bits = 1]
pub enum LpfDisable {
    /// Low-pass filter enabled (default signal path).
    Enabled = 0,
    /// Low-pass filter disabled.
    Disabled = 1,
}

/// FIFO packing formats encoded in `FIFO_CTL.FIFO_FORMAT`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Specifier)]
#[repr(u8)]
#[bits = 3]
pub enum FifoFormat {
    /// All axes interleaved (X, Y, Z).
    XYZ = 0b000,
    /// X-axis samples only.
    X = 0b001,
    /// Y-axis samples only.
    Y = 0b010,
    /// X and Y axes interleaved.
    XY = 0b011,
    /// Z-axis samples only.
    Z = 0b100,
    /// X and Z axes interleaved.
    XZ = 0b101,
    /// Y and Z axes interleaved.
    YZ = 0b110,
    /// Peak acceleration reporting.
    Peak = 0b111,
}

impl FifoFormat {
    /// Returns the number of axes encoded in each FIFO sample.
    pub const fn axis_count(self) -> u8 {
        match self {
            Self::X | Self::Y | Self::Z => 1,
            Self::XY | Self::XZ | Self::YZ => 2,
            Self::XYZ | Self::Peak => 3,
        }
    }
}

/// FIFO operating modes encoded in `FIFO_CTL.FIFO_MODE`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Specifier)]
#[repr(u8)]
#[bits = 2]
pub enum FifoMode {
    /// FIFO disabled; bypassed.
    Bypass = 0b00,
    /// Streaming mode (circular buffer).
    Stream = 0b01,
    /// Trigger mode.
    Trigger = 0b10,
    /// Oldest-saved mode.
    OldestSaved = 0b11,
}

/// Wake-up timer selections encoded in `TIMING[4:2]` (milliseconds).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Specifier)]
#[repr(u8)]
#[bits = 3]
pub enum WakeUpRate {
    /// 52 ms.
    Ms52 = 0b000,
    /// 104 ms.
    Ms104 = 0b001,
    /// 208 ms.
    Ms208 = 0b010,
    /// 512 ms.
    Ms512 = 0b011,
    /// 2048 ms.
    Ms2048 = 0b100,
    /// 4096 ms.
    Ms4096 = 0b101,
    /// 8192 ms.
    Ms8192 = 0b110,
    /// 24576 ms.
    Ms24576 = 0b111,
}

impl WakeUpRate {
    /// Returns the interval expressed in milliseconds.
    pub const fn millis(self) -> u32 {
        match self {
            Self::Ms52 => 52,
            Self::Ms104 => 104,
            Self::Ms208 => 208,
            Self::Ms512 => 512,
            Self::Ms2048 => 2_048,
            Self::Ms4096 => 4_096,
            Self::Ms8192 => 8_192,
            Self::Ms24576 => 24_576,
        }
    }
}

/// External clock enable bit (`TIMING.EXT_CLK`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Specifier)]
#[repr(u8)]
#[bits = 1]
pub enum ExtClk {
    /// External clock disabled.
    Disabled = 0,
    /// External clock enabled.
    Enabled = 1,
}

/// External sync selection bit (`TIMING.EXT_SYNC`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Specifier)]
#[repr(u8)]
#[bits = 1]
pub enum ExtSync {
    /// External sync disabled.
    Disabled = 0,
    /// External sync enabled.
    Enabled = 1,
}

/// I2C high-speed mode enable bit (`TIMING.I2C_HSM_EN`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Specifier)]
#[repr(u8)]
#[bits = 1]
pub enum I2cHsmEn {
    /// High-speed mode disabled (standard I2C timing).
    Disabled = 0,
    /// High-speed mode enabled (up to 3.4 MHz).
    Enabled = 1,
}

/// User overrange disable flag (`MEASURE.USER_OR_DISABLE`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Specifier)]
#[repr(u8)]
#[bits = 1]
pub enum UserOrDisable {
    /// Overrange detection is enabled (bit cleared, default).
    Enabled = 0,
    /// Overrange detection is disabled (bit set).
    Disabled = 1,
}

/// Autosleep control bit (`MEASURE.AUTOSLEEP`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Specifier)]
#[repr(u8)]
#[bits = 1]
pub enum AutoSleep {
    /// Autosleep disabled.
    Disabled = 0,
    /// Autosleep enabled.
    Enabled = 1,
}

/// Low-noise modes encoded in `MEASURE.LOW_NOISE`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Specifier)]
#[repr(u8)]
#[bits = 1]
pub enum LowNoise {
    /// Normal noise performance.
    Normal = 0,
    /// Low-noise mode.
    LowNoise = 1,
}

/// Filter settle durations in `POWER_CTL.FILTER_SETTLE`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Specifier)]
#[repr(u8)]
#[bits = 1]
pub enum SettleFilter {
    /// 370 ms settle time.
    Ms370 = 0,
    /// 16 ms settle time.
    Ms16 = 1,
}

impl SettleFilter {
    /// Returns the nominal settle time in milliseconds.
    pub const fn millis(self) -> u16 {
        match self {
            Self::Ms370 => 370,
            Self::Ms16 => 16,
        }
    }
}

/// Instant-on thresholds encoded in `POWER_CTL.INSTANT_ON_THRESH`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Specifier)]
#[repr(u8)]
#[bits = 1]
pub enum InstantOnThreshold {
    /// 10 g ±5 g threshold.
    Low = 0,
    /// 30 g ±10 g threshold.
    High = 1,
}

/// Link/loop interaction modes encoded in `MEASURE[5:4]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Specifier)]
#[repr(u8)]
#[bits = 2]
pub enum LinkLoopMode {
    /// Default (unlinked) mode.
    Default = 0b00,
    /// Linked activity/inactivity detectors.
    Linked = 0b01,
    /// Loop mode.
    Loop = 0b10,
}

/// Operating power modes encoded in `POWER_CTL[1:0]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Specifier)]
#[repr(u8)]
#[bits = 2]
pub enum PowerMode {
    /// Standby mode.
    Standby = 0b00,
    /// Wake-up mode.
    WakeUp = 0b01,
    /// Instant-on mode.
    InstantOn = 0b10,
    /// Full measurement mode.
    Measure = 0b11,
}

/// High-pass filter corner selections encoded in `HPF[1:0]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Specifier)]
#[repr(u8)]
#[bits = 2]
pub enum HighPassCorner {
    /// Corner selection 0.
    Corner0 = 0b00,
    /// Corner selection 1.
    Corner1 = 0b01,
    /// Corner selection 2.
    Corner2 = 0b10,
    /// Corner selection 3.
    Corner3 = 0b11,
}

impl HighPassCorner {
    /// Returns the corner frequency in hertz for the supplied ODR.
    pub const fn hz(self, odr: OutputDataRate) -> f32 {
        match (odr, self) {
            (OutputDataRate::Od400Hz, Self::Corner0) => 1.90,
            (OutputDataRate::Od400Hz, Self::Corner1) => 0.97,
            (OutputDataRate::Od400Hz, Self::Corner2) => 0.49,
            (OutputDataRate::Od400Hz, Self::Corner3) => 0.24,
            (OutputDataRate::Od800Hz, Self::Corner0) => 3.81,
            (OutputDataRate::Od800Hz, Self::Corner1) => 1.94,
            (OutputDataRate::Od800Hz, Self::Corner2) => 0.98,
            (OutputDataRate::Od800Hz, Self::Corner3) => 0.49,
            (OutputDataRate::Od1600Hz, Self::Corner0) => 7.61,
            (OutputDataRate::Od1600Hz, Self::Corner1) => 3.89,
            (OutputDataRate::Od1600Hz, Self::Corner2) => 1.97,
            (OutputDataRate::Od1600Hz, Self::Corner3) => 0.99,
            (OutputDataRate::Od3200Hz, Self::Corner0) => 15.24,
            (OutputDataRate::Od3200Hz, Self::Corner1) => 7.79,
            (OutputDataRate::Od3200Hz, Self::Corner2) => 3.94,
            (OutputDataRate::Od3200Hz, Self::Corner3) => 1.98,
            (OutputDataRate::Od6400Hz, Self::Corner0) => 30.48,
            (OutputDataRate::Od6400Hz, Self::Corner1) => 15.58,
            (OutputDataRate::Od6400Hz, Self::Corner2) => 7.88,
            (OutputDataRate::Od6400Hz, Self::Corner3) => 3.96,
        }
    }

    /// Returns `true` when the corner frequency is ≤ 10 Hz for the supplied ODR.
    pub const fn is_activity_lp_compatible(self, odr: OutputDataRate) -> bool {
        self.hz(odr) <= 10.0
    }
}

/// High-pass filter disable bit (`FILTER_CTL.HPF_DISABLE`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Specifier)]
#[repr(u8)]
#[bits = 1]
pub enum HpfDisable {
    /// High-pass filter enabled.
    Enabled = 0,
    /// High-pass filter disabled (dc coupling).
    Disabled = 1,
}
