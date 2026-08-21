//! Register map definitions for the ADXL372 accelerometer.
#![allow(unused_parens)]

use modular_bitfield::prelude::*;

use crate::params::{
    Bandwidth, ExtClk, ExtSync, FifoFormat, FifoMode, InstantOnThreshold, LinkLoopMode, LowNoise,
    OutputDataRate, PowerMode, SettleFilter, WakeUpRate,
};

/// Register address of `DEVID_AD`.
pub const REG_DEVID_AD: u8 = 0x00;
/// Register address of `DEVID_MST`.
pub const REG_DEVID_MST: u8 = 0x01;
/// Register address of `PARTID`.
pub const REG_PARTID: u8 = 0x02;
/// Register address of `REVID`.
pub const REG_REVID: u8 = 0x03;
/// Register address of `STATUS`.
pub const REG_STATUS: u8 = 0x04;
/// Register address of `STATUS2`.
pub const REG_STATUS2: u8 = 0x05;
/// Register address of `FIFO_ENTRIES2`.
pub const REG_FIFO_ENTRIES2: u8 = 0x06;
/// Register address of `FIFO_ENTRIES`.
pub const REG_FIFO_ENTRIES: u8 = 0x07;
/// Register address of `XDATA_H`.
pub const REG_XDATA_H: u8 = 0x08;
/// Register address of `XDATA_L`.
pub const REG_XDATA_L: u8 = 0x09;
/// Register address of `YDATA_H`.
pub const REG_YDATA_H: u8 = 0x0A;
/// Register address of `YDATA_L`.
pub const REG_YDATA_L: u8 = 0x0B;
/// Register address of `ZDATA_H`.
pub const REG_ZDATA_H: u8 = 0x0C;
/// Register address of `ZDATA_L`.
pub const REG_ZDATA_L: u8 = 0x0D;
/// Register address of `TEMP_DATA`.
pub const REG_TEMP_DATA: u8 = 0x0E;
/// Register address of `FIFO_DATA`.
pub const REG_FIFO_DATA: u8 = 0x42;
/// Register address of `FIFO_SAMPLES`.
pub const REG_FIFO_SAMPLES: u8 = 0x39;
/// Register address of `FIFO_CTL`.
pub const REG_FIFO_CTL: u8 = 0x3A;
/// Register address of `HPF`.
pub const REG_HPF: u8 = 0x3C;
/// Register address of `TIMING`.
pub const REG_TIMING: u8 = 0x3D;
/// Register address of `MEASURE`.
pub const REG_MEASURE: u8 = 0x3E;
/// Register address of `POWER_CTL`.
pub const REG_POWER_CTL: u8 = 0x3F;
/// Register address of `SELF_TEST`.
pub const REG_SELF_TEST: u8 = 0x40;
/// Register address of `RESET`.
pub const REG_RESET: u8 = 0x41;

/// Access permissions encoded for each register.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterAccess {
    /// Read-only register.
    ReadOnly,
    /// Write-only register.
    WriteOnly,
    /// Read/write register.
    ReadWrite,
}

/// Minimal metadata exposed by every register value type.
pub trait Register {
    /// Raw storage backing the register payload.
    type Raw: Copy;
    /// Register address as documented in the datasheet.
    const ADDRESS: u8;
    /// Access permission classification.
    const ACCESS: RegisterAccess;
    /// Optional reset/default value defined by the datasheet.
    const RESET_VALUE: Option<Self::Raw>;
}

/// Bitfield representation of the `STATUS` register (address `0x04`).
#[allow(unused_parens)]
#[bitfield]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Status {
    // Data ready flag (bit 0).
    pub data_ready: bool,
    // FIFO ready flag (bit 1).
    pub fifo_ready: bool,
    // FIFO full flag (bit 2).
    pub fifo_full: bool,
    // FIFO overrun flag (bit 3).
    pub fifo_overrun: bool,
    #[skip]
    __: B1,
    // Internal non-volatile memory busy indicator (bit 5).
    pub user_nvm_busy: bool,
    // Device is awake (bit 6).
    pub awake: bool,
    // User register checksum error detected (bit 7).
    pub err_user_regs: bool,
}

impl From<u8> for Status {
    fn from(value: u8) -> Self {
        Self::from_bytes([value])
    }
}

impl From<Status> for u8 {
    fn from(value: Status) -> Self {
        value.into_bytes()[0]
    }
}

/// Bitfield representation of the `STATUS2` register (address `0x05`).
#[allow(unused_parens)]
#[bitfield]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Status2 {
    #[skip]
    __: B4,
    // Inactivity event detected (bit 4).
    pub inactivity: bool,
    // Activity event detected (bit 5).
    pub activity: bool,
    // Activity2 event detected (bit 6).
    pub activity2: bool,
    #[skip]
    __: B1,
}

impl From<u8> for Status2 {
    fn from(value: u8) -> Self {
        Self::from_bytes([value])
    }
}

impl From<Status2> for u8 {
    fn from(value: Status2) -> Self {
        value.into_bytes()[0]
    }
}

/// Upper bits of the FIFO entry counter (`FIFO_ENTRIES2`, address `0x06`).
#[allow(unused_parens)]
#[bitfield]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FifoEntriesUpper {
    // Bits 9:8 of the FIFO entry counter.
    pub upper: B2,
    #[skip]
    __: B6,
}

impl FifoEntriesUpper {
    /// Returns the upper two bits of the FIFO entry count as a small integer.
    pub fn as_u16(self) -> u16 {
        self.upper() as u16
    }
}

/// Bitfield representation of the `FIFO_CTL` register (address `0x3A`).
#[allow(unused_parens)]
#[bitfield]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FifoControl {
    // Most significant bit of the FIFO sample watermark.
    pub samples_msb: bool,
    // FIFO operating mode selection.
    pub mode: FifoMode,
    // FIFO data packing format.
    pub format: FifoFormat,
    #[skip]
    __: B2,
}

impl From<u8> for FifoControl {
    fn from(value: u8) -> Self {
        Self::from_bytes([value])
    }
}

impl From<FifoControl> for u8 {
    fn from(value: FifoControl) -> Self {
        value.into_bytes()[0]
    }
}

/// Bitfield representation of the `TIMING` register (address `0x3D`).
#[allow(unused_parens)]
#[bitfield]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timing {
    // External sync enable bit (bit 0).
    pub ext_sync: ExtSync,
    // External clock enable bit (bit 1).
    pub ext_clk: ExtClk,
    // Wake-up rate selection (bits 4:2).
    pub wake_up_rate: WakeUpRate,
    // Output data rate selection (bits 7:5).
    pub odr: OutputDataRate,
}

impl From<u8> for Timing {
    fn from(value: u8) -> Self {
        Self::from_bytes([value])
    }
}

impl From<Timing> for u8 {
    fn from(value: Timing) -> Self {
        value.into_bytes()[0]
    }
}

/// Bitfield representation of the `MEASURE` register (address `0x3E`).
#[allow(unused_parens)]
#[bitfield]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Measure {
    // Low-pass bandwidth selection (bits 2:0).
    pub bandwidth: Bandwidth,
    // Low-noise operating mode selection (bit 3).
    pub low_noise: LowNoise,
    // Link/loop control selection (bits 5:4).
    pub link_loop_mode: LinkLoopMode,
    // Autosleep enable flag (bit 6).
    pub autosleep: bool,
    // USER_OR / disable output selection (bit 7).
    pub user_or_disable: bool,
}

impl From<u8> for Measure {
    fn from(value: u8) -> Self {
        Self::from_bytes([value])
    }
}

impl From<Measure> for u8 {
    fn from(value: Measure) -> Self {
        value.into_bytes()[0]
    }
}

/// Bitfield representation of the `POWER_CTL` register (address `0x3F`).
#[allow(unused_parens)]
#[bitfield]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PowerControl {
    // Power mode selection (bits 1:0).
    pub mode: PowerMode,
    // High-pass filter disable flag (bit 2).
    pub hpf_disable: bool,
    // Low-pass filter disable flag (bit 3).
    pub lpf_disable: bool,
    // Filter settling behaviour (bit 4).
    pub filter_settle: SettleFilter,
    // Instant-on threshold selection (bit 5).
    pub instant_on_threshold: InstantOnThreshold,
    #[skip]
    __: B1,
    // I²C high-speed enable flag (bit 7).
    pub i2c_high_speed_enable: bool,
}

impl From<u8> for PowerControl {
    fn from(value: u8) -> Self {
        Self::from_bytes([value])
    }
}

impl From<PowerControl> for u8 {
    fn from(value: PowerControl) -> Self {
        value.into_bytes()[0]
    }
}

/// Bitfield representation of the `SELF_TEST` register (address `0x40`).
#[allow(unused_parens)]
#[bitfield]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelfTest {
    // Self-test enable bit (bit 0).
    pub st: bool,
    // Self-test complete flag (bit 1).
    pub st_done: bool,
    // User-triggered self-test request (bit 2).
    pub user_st: bool,
    #[skip]
    __: B5,
}

impl From<u8> for SelfTest {
    fn from(value: u8) -> Self {
        Self::from_bytes([value])
    }
}

impl From<SelfTest> for u8 {
    fn from(value: SelfTest) -> Self {
        value.into_bytes()[0]
    }
}

impl Register for Status {
    type Raw = u8;
    const ADDRESS: u8 = REG_STATUS;
    const ACCESS: RegisterAccess = RegisterAccess::ReadOnly;
    const RESET_VALUE: Option<Self::Raw> = Some(0xA0);
}

impl Register for Status2 {
    type Raw = u8;
    const ADDRESS: u8 = REG_STATUS2;
    const ACCESS: RegisterAccess = RegisterAccess::ReadOnly;
    const RESET_VALUE: Option<Self::Raw> = Some(0x00);
}

impl Register for FifoControl {
    type Raw = u8;
    const ADDRESS: u8 = REG_FIFO_CTL;
    const ACCESS: RegisterAccess = RegisterAccess::ReadWrite;
    const RESET_VALUE: Option<Self::Raw> = Some(0x00);
}

impl Register for Timing {
    type Raw = u8;
    const ADDRESS: u8 = REG_TIMING;
    const ACCESS: RegisterAccess = RegisterAccess::ReadWrite;
    const RESET_VALUE: Option<Self::Raw> = Some(0x00);
}

impl Register for Measure {
    type Raw = u8;
    const ADDRESS: u8 = REG_MEASURE;
    const ACCESS: RegisterAccess = RegisterAccess::ReadWrite;
    const RESET_VALUE: Option<Self::Raw> = Some(0x00);
}

impl Register for PowerControl {
    type Raw = u8;
    const ADDRESS: u8 = REG_POWER_CTL;
    const ACCESS: RegisterAccess = RegisterAccess::ReadWrite;
    const RESET_VALUE: Option<Self::Raw> = Some(0x00);
}

impl Register for SelfTest {
    type Raw = u8;
    const ADDRESS: u8 = REG_SELF_TEST;
    const ACCESS: RegisterAccess = RegisterAccess::ReadWrite;
    const RESET_VALUE: Option<Self::Raw> = Some(0x00);
}

/// Encodes the FIFO entry count from the upper and lower registers.
pub fn fifo_entry_count(upper: FifoEntriesUpper, lower: u8) -> u16 {
    (upper.as_u16() << 8) | lower as u16
}

/// Splits the FIFO entry count into upper and lower register values.
pub fn split_fifo_entry_count(count: u16) -> (FifoEntriesUpper, u8) {
    let upper = FifoEntriesUpper::new().with_upper(((count >> 8) & 0x03) as u8);
    let lower = (count & 0xFF) as u8;
    (upper, lower)
}

/// Soft reset command value written to the `RESET` register.
pub const RESET_COMMAND: u8 = 0x52;

/// Expected value returned by the `DEVID_AD` register (`0x00`).
pub const EXPECTED_DEVID_AD: u8 = 0xAD;
/// Expected value returned by the `DEVID_MST` register (`0x01`).
pub const EXPECTED_DEVID_MST: u8 = 0x1D;
/// Expected value returned by the `PARTID` register (`0x02`).
pub const EXPECTED_PART_ID: u8 = 0xFA;

#[cfg(test)]
mod tests {
    use super::*;

    /// Validates that Status bitfields match the datasheet layout.
    #[test]
    fn status_layout_matches_datasheet() {
        let status = Status::from(0b1010_0000);
        assert!(!status.data_ready());
        assert!(!status.fifo_ready());
        assert!(!status.fifo_full());
        assert!(!status.fifo_overrun());
        assert!(status.user_nvm_busy());
        assert!(!status.awake());
        assert!(status.err_user_regs());
    }

    /// Ensures Timing encodes and decodes as expected across all fields.
    #[test]
    fn timing_roundtrip() {
        let timing = Timing::new()
            .with_ext_sync(ExtSync::Enabled)
            .with_ext_clk(ExtClk::Disabled)
            .with_wake_up_rate(WakeUpRate::Ms512)
            .with_odr(OutputDataRate::Od1600Hz);

        assert_eq!(u8::from(timing), 0b010_011_0_1);
        let decoded = Timing::from(u8::from(timing));
        assert_eq!(decoded.wake_up_rate(), WakeUpRate::Ms512);
        assert_eq!(decoded.odr(), OutputDataRate::Od1600Hz);
        assert_eq!(decoded.ext_sync(), ExtSync::Enabled);
        assert_eq!(decoded.ext_clk(), ExtClk::Disabled);
    }
}
