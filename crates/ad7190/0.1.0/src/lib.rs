// SPDX-License-Identifier: MPL-2.0
//
// Copyright (c) 2026 Damian Peckett <damian@pecke.tt>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![no_std]

//! Async embedded-hal driver for the Analog Devices AD7190 ADC.
//!
//! This crate provides a `no_std`, async driver for the AD7190 24-bit
//! sigma-delta ADC using [`embedded-hal-async`] traits.

#[cfg(not(feature = "defmt"))]
use bitflags::bitflags;
use core::marker::PhantomData;
#[cfg(feature = "defmt")]
use defmt::bitflags;
use embedded_hal_async::{
    delay::DelayNs,
    digital::Wait,
    spi::{Operation, SpiDevice},
};

/// AD7190 register addresses (RS2..RS0) used in the Communications Register.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Reg {
    Status = 0b000,    // read-only, 8-bit
    Mode = 0b001,      // 24-bit
    Config = 0b010,    // 24-bit
    Data = 0b011,      // 24-bit or 32-bit if DAT_STA=1
    Id = 0b100,        // read-only, 8-bit
    Gpocon = 0b101,    // 8-bit
    Offset = 0b110,    // 24-bit (channel-specific)
    FullScale = 0b111, // 24-bit (channel-specific)
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ClockSel {
    ExtCrystal = 0b00,
    ExtClock = 0b01,
    IntNoMclk2 = 0b10,
    IntOnMclk2 = 0b11,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FilterSel {
    Sinc4,
    Sinc3,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OperatingMode {
    ContinuousConversion = 0b000,
    SingleConversion = 0b001,
    Idle = 0b010,
    PowerDown = 0b011,
    InternalZeroScaleCal = 0b100,
    InternalFullScaleCal = 0b101,
    SystemZeroScaleCal = 0b110,
    SystemFullScaleCal = 0b111,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Gain {
    G1 = 0b000,
    G8 = 0b011,
    G16 = 0b100,
    G32 = 0b101,
    G64 = 0b110,
    G128 = 0b111,
}

impl Gain {
    pub fn as_u16(self) -> u16 {
        match self {
            Gain::G1 => 1,
            Gain::G8 => 8,
            Gain::G16 => 16,
            Gain::G32 => 32,
            Gain::G64 => 64,
            Gain::G128 => 128,
        }
    }
}

bitflags! {
    /// STATUS register bits (8-bit).
    ///
    /// Note: Some bits are fields, not flags (e.g. CHD[2:0]).
    #[cfg_attr(not(feature = "defmt"), derive(Copy, Clone, Debug, Eq, PartialEq))]
    pub struct StatusFlags: u8 {
        /// RDY bit (bit7). 1 = not ready, 0 = ready
        const RDY = 1 << 7;
        /// ERR bit (bit6) (datasheet-defined error indicator)
        const ERR = 1 << 6;
        /// NOREF bit (bit5) (reference detect indicator)
        const NOREF = 1 << 5;
        // Bits 4..3 are (typically) parity and reserved depending on configuration; leave as raw.
    }
}

bitflags! {
    /// CONFIG register flag-type bits (24-bit).
    ///
    /// Fields like GAIN and CHANNEL_MASK are handled separately in `Config`.
    #[cfg_attr(not(feature = "defmt"), derive(Copy, Clone, Debug, Eq, PartialEq))]
    pub struct ConfigFlags: u32 {
        /// CHOP (bit23)
        const CHOP = 1 << 23;
        /// REFSEL (bit20) 0=REFIN1, 1=REFIN2 (P1/P0)
        const REFSEL = 1 << 20;
        /// BURN (bit7)
        const BURNOUT = 1 << 7;
        /// REFDET (bit6)
        const REFDET = 1 << 6;
        /// BUF (bit4)
        const BUF = 1 << 4;
        /// U/B (bit3) 1=unipolar, 0=bipolar (offset binary)
        const UNIPOLAR = 1 << 3;
    }
}

bitflags! {
    /// MODE register flag-type bits (24-bit).
    ///
    /// Fields like MODE, CLOCKSEL, FS are handled separately in `Mode`.
    #[cfg_attr(not(feature = "defmt"), derive(Copy, Clone, Debug, Eq, PartialEq))]
    pub struct ModeFlags: u32 {
        /// DAT_STA (bit20): append status byte after data read
        const DAT_STA = 1 << 20;
        /// SINC3 (bit15): 1 = SINC3, 0 = SINC4
        const SINC3 = 1 << 15;
        /// ENPAR (bit13): parity enable
        const ENPAR = 1 << 13;
        /// SINGLE (bit11): single cycle conversion
        const SINGLE = 1 << 11;
        /// REJ60 (bit10): 50/60 Hz rejection
        const REJ60 = 1 << 10;
    }
}

bitflags! {
    /// Channel enable mask (CON15..CON8).
    /// Use as `Channels::CH0 | Channels::CH2`, etc.
    #[cfg_attr(not(feature = "defmt"), derive(Copy, Clone, Debug, Eq, PartialEq))]
    pub struct Channels: u8 {
        const CH0 = 1 << 0;
        const CH1 = 1 << 1;
        const CH2 = 1 << 2;
        const CH3 = 1 << 3;
        const CH4 = 1 << 4;
        const CH5 = 1 << 5;
        const CH6 = 1 << 6;
        const CH7 = 1 << 7;
    }
}

/// Configuration register builder (24-bit).
///
/// Uses `bitflags` for the boolean-like fields, plus a channel mask and gain field.
#[derive(Copy, Clone, Debug)]
pub struct Config {
    pub flags: ConfigFlags,
    pub channels: Channels,
    pub gain: Gain,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            flags: ConfigFlags::BUF, // buffered enabled
            channels: Channels::CH0,
            gain: Gain::G128,
        }
    }
}

impl Config {
    pub fn with_unipolar(mut self, unipolar: bool) -> Self {
        if unipolar {
            self.flags.insert(ConfigFlags::UNIPOLAR);
        } else {
            self.flags.remove(ConfigFlags::UNIPOLAR);
        }
        self
    }

    pub fn to_u24(self) -> u32 {
        let mut v = self.flags.bits() & 0x00FF_FFFF;

        // CON15..CON8 channel enables
        v |= (self.channels.bits() as u32) << 8;

        // CON2..CON0 gain
        v |= (self.gain as u32) & 0x7;

        v & 0x00FF_FFFF
    }
}

/// Mode register builder (24-bit).
///
/// Uses `bitflags` for boolean-like fields, plus explicit mode/clock/fs fields.
#[derive(Copy, Clone, Debug)]
pub struct Mode {
    pub op_mode: OperatingMode, // MR23..MR21
    pub clock: ClockSel,        // MR19..MR18
    pub flags: ModeFlags,       // e.g. DAT_STA, SINC3, ...
    pub fs: u16,                // MR9..MR0 (1..=1023)
}

impl Default for Mode {
    fn default() -> Self {
        Self {
            op_mode: OperatingMode::ContinuousConversion,
            clock: ClockSel::IntNoMclk2,
            flags: ModeFlags::empty(),
            fs: 0x060,
        }
    }
}

impl Mode {
    pub fn with_dat_sta(mut self, dat_sta: bool) -> Self {
        if dat_sta {
            self.flags.insert(ModeFlags::DAT_STA);
        } else {
            self.flags.remove(ModeFlags::DAT_STA);
        }
        self
    }

    pub fn with_filter(mut self, filter: FilterSel) -> Self {
        match filter {
            FilterSel::Sinc4 => self.flags.remove(ModeFlags::SINC3),
            FilterSel::Sinc3 => self.flags.insert(ModeFlags::SINC3),
        }
        self
    }

    pub fn to_u24(self) -> u32 {
        let fs = self.fs.clamp(1, 1023) as u32;

        let mut v = 0u32;
        v |= (self.op_mode as u32) << 21;
        v |= (self.clock as u32) << 18;
        v |= self.flags.bits() & 0x00FF_FFFF;
        v |= fs & 0x3FF;

        v & 0x00FF_FFFF
    }

    pub fn dat_sta_enabled(&self) -> bool {
        self.flags.contains(ModeFlags::DAT_STA)
    }

    /// AD7190 output data rate (ODR) in chop-disabled mode:
    /// f_adc = f_clk / (1024 * FS)
    ///
    /// This sets FS to achieve (approximately) `hz` given `f_clk_hz`.
    /// - `hz` is clamped to >= 1e-3 to avoid division-by-zero.
    /// - FS is rounded to nearest integer and clamped to 1..=1023.
    ///
    /// Note: if you enable CHOP elsewhere, this formula is different.
    pub fn with_sample_rate_hz_from_clk(mut self, hz: f32, f_clk_hz: f32) -> Self {
        let hz = hz.max(1e-3);
        let f_clk_hz = f_clk_hz.max(1e-3);

        // FS ≈ f_clk / (1024 * f_adc)
        let fs_f = f_clk_hz / (1024.0 * hz);

        // Round to nearest, then clamp to AD7190 limits.
        let fs = (fs_f + 0.5) as i32;
        let fs = fs.clamp(1, 1023) as u16;

        self.fs = fs;
        self
    }

    /// Convenience helper when using the AD7190 internal clock.
    ///
    /// Uses the typical internal clock frequency (datasheet-typical). If you use an
    /// external clock/crystal, prefer `with_sample_rate_hz_from_clk`.
    pub fn with_sample_rate_hz(mut self, hz: f32) -> Self {
        // Typical internal clock ~4.92 MHz (datasheet "typ").
        // If you care about exact ODR, pass your real f_clk to `_from_clk`.
        const INT_CLK_TYP_HZ: f32 = 4_920_000.0;

        match self.clock {
            ClockSel::IntNoMclk2 | ClockSel::IntOnMclk2 => {
                self = self.with_sample_rate_hz_from_clk(hz, INT_CLK_TYP_HZ);
            }
            ClockSel::ExtCrystal | ClockSel::ExtClock => {
                // We don't know your external clock frequency here.
                // Leave FS unchanged; caller should use `_from_clk`.
            }
        }
        self
    }
}

/// Result of reading the data register.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Sample {
    /// 24-bit conversion code in bits [23:0]
    pub code: u32,
    /// Optional status byte (present when DAT_STA=1)
    pub status: Option<u8>,
}

/// Driver errors.
#[derive(Debug)]
pub enum Error<SpiErr, PinErr> {
    Spi(SpiErr),
    Drdy(PinErr),
    Timeout,
    /// Optional policy hook for callers (e.g., status shows ERR/NOREF).
    AdcFault {
        status: u8,
    },
}

/// Continuous conversion behavior configuration.
#[derive(Copy, Clone, Debug)]
pub struct ContinuousCfg {
    /// If true, read returns 24-bit data + status byte (recommended for multi-channel scanning).
    pub dat_sta: bool,
    /// If true, and DOUT/RDY is provided, await it. Otherwise fall back to polling.
    pub use_pin_if_available: bool,
    /// If true, enable CREAD (streaming data reads) after entering continuous conversion.
    pub use_cread: bool,
    /// Poll loop max tries (only used when DRDY pin not used).
    pub poll_max_tries: u32,
    /// Poll loop delay in microseconds (only used when DRDY pin not used).
    pub poll_delay_us: u32,
}

impl Default for ContinuousCfg {
    fn default() -> Self {
        Self {
            dat_sta: true,
            use_pin_if_available: true,
            use_cread: false,
            poll_max_tries: 50_000,
            poll_delay_us: 10,
        }
    }
}

/// AD7190 GPIO pin identifiers (general-purpose outputs).
///
/// Notes:
/// - P0/P1 share enable bit GP10EN.
/// - P2/P3 share enable bit GP32EN.
/// - P0/P1 are multiplexed with REFIN2(-)/(+) when REFSEL=1 in CONFIG.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum GpioPin {
    P0,
    P1,
    P2,
    P3,
}

/// Parsed representation of the GPOCON register (8-bit).
///
/// Bit layout (MSB..LSB):
/// - GP7: must be 0
/// - GP6: BPDSW
/// - GP5: GP32EN (enable P3,P2 outputs)
/// - GP4: GP10EN (enable P1,P0 outputs)
/// - GP3: P3DAT
/// - GP2: P2DAT
/// - GP1: P1DAT
/// - GP0: P0DAT
#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
pub struct Gpocon {
    /// Bridge power-down switch control (BPDSW).
    pub bpdsw: bool,
    /// Enable outputs P3 and P2 (GP32EN).
    pub enable_p3_p2: bool,
    /// Enable outputs P1 and P0 (GP10EN).
    pub enable_p1_p0: bool,
    /// Output data bit for P3 (P3DAT).
    pub p3: bool,
    /// Output data bit for P2 (P2DAT).
    pub p2: bool,
    /// Output data bit for P1 (P1DAT).
    pub p1: bool,
    /// Output data bit for P0 (P0DAT).
    pub p0: bool,
}

impl Gpocon {
    #[inline(always)]
    pub fn from_u8(v: u8) -> Self {
        Self {
            bpdsw: (v & (1 << 6)) != 0,
            enable_p3_p2: (v & (1 << 5)) != 0,
            enable_p1_p0: (v & (1 << 4)) != 0,
            p3: (v & (1 << 3)) != 0,
            p2: (v & (1 << 2)) != 0,
            p1: (v & (1 << 1)) != 0,
            p0: (v & (1 << 0)) != 0,
        }
    }

    #[inline(always)]
    pub fn to_u8(self) -> u8 {
        let mut v = 0u8;

        // GP7 must be 0.
        if self.bpdsw {
            v |= 1 << 6;
        }
        if self.enable_p3_p2 {
            v |= 1 << 5;
        }
        if self.enable_p1_p0 {
            v |= 1 << 4;
        }
        if self.p3 {
            v |= 1 << 3;
        }
        if self.p2 {
            v |= 1 << 2;
        }
        if self.p1 {
            v |= 1 << 1;
        }
        if self.p0 {
            v |= 1 << 0;
        }

        v
    }

    #[inline(always)]
    pub fn is_enabled(self, pin: GpioPin) -> bool {
        match pin {
            GpioPin::P0 | GpioPin::P1 => self.enable_p1_p0,
            GpioPin::P2 | GpioPin::P3 => self.enable_p3_p2,
        }
    }

    #[inline(always)]
    pub fn level(self, pin: GpioPin) -> bool {
        match pin {
            GpioPin::P0 => self.p0,
            GpioPin::P1 => self.p1,
            GpioPin::P2 => self.p2,
            GpioPin::P3 => self.p3,
        }
    }

    #[inline(always)]
    pub fn set_level(&mut self, pin: GpioPin, high: bool) {
        match pin {
            GpioPin::P0 => self.p0 = high,
            GpioPin::P1 => self.p1 = high,
            GpioPin::P2 => self.p2 = high,
            GpioPin::P3 => self.p3 = high,
        }
    }

    /// Enable/disable the enable group for the selected pin (GP10EN or GP32EN).
    #[inline(always)]
    pub fn set_enabled_group(&mut self, pin: GpioPin, enabled: bool) {
        match pin {
            GpioPin::P0 | GpioPin::P1 => self.enable_p1_p0 = enabled,
            GpioPin::P2 | GpioPin::P3 => self.enable_p3_p2 = enabled,
        }
    }
}

/// AD7190 async driver.
///
/// - Uses `SpiDevice` so CS is managed by the SPI device implementation.
/// - `drdy` is optional at runtime: `Option<DRDY>`.
/// - If `drdy` is None, the driver can poll STATUS.RDY.
pub struct Ad7190<SPI, DRDY> {
    spi: SPI,
    drdy: Option<DRDY>,
    _phantom: PhantomData<DRDY>,
}

impl<SPI, DRDY> Ad7190<SPI, DRDY> {
    pub fn new(spi: SPI, drdy: Option<DRDY>) -> Self {
        Self {
            spi,
            drdy,
            _phantom: PhantomData,
        }
    }

    pub fn free(self) -> (SPI, Option<DRDY>) {
        (self.spi, self.drdy)
    }
}

impl<SPI, DRDY, SpiErr, PinErr> Ad7190<SPI, DRDY>
where
    SPI: SpiDevice<Error = SpiErr>,
    DRDY: Wait<Error = PinErr>,
{
    /// Communications register byte:
    ///
    /// - bit7 WEN (must be 0 to enable writes)
    /// - bit6 R/W (1=read, 0=write)
    /// - bit5..3 RS2..RS0 (register select)
    /// - bit2 CREAD (continuous read enable)
    /// - bit1..0 0
    #[inline(always)]
    fn comm_byte(reg: Reg, read: bool, continuous_read: bool) -> u8 {
        let rw = if read { 1u8 } else { 0u8 };
        let cread = if continuous_read { 1u8 } else { 0u8 };
        (rw << 6) | ((reg as u8) << 3) | (cread << 2)
    }

    /// Reset the interface by clocking 40 ones (5 bytes of 0xFF), then wait ~500 µs.
    pub async fn reset<D: DelayNs>(&mut self, delay: &mut D) -> Result<(), Error<SpiErr, PinErr>> {
        let ones = [0xFFu8; 5];
        self.spi.write(&ones).await.map_err(Error::Spi)?;
        delay.delay_us(500).await;
        Ok(())
    }

    /// Read an 8-bit register (STATUS, ID, GPOCON).
    ///
    /// IMPORTANT: must keep CS asserted across "cmd" then "read".
    pub async fn read_u8(&mut self, reg: Reg) -> Result<u8, Error<SpiErr, PinErr>> {
        let cmd = [Self::comm_byte(reg, true, false)];
        let mut b = [0u8; 1];

        self.spi
            .transaction(&mut [Operation::Write(&cmd), Operation::Read(&mut b)])
            .await
            .map_err(Error::Spi)?;

        Ok(b[0])
    }

    /// Read STATUS as flags (does not include CHD field; get that separately via `status_chd()`).
    pub async fn read_status_flags(&mut self) -> Result<(StatusFlags, u8), Error<SpiErr, PinErr>> {
        let raw = self.read_u8(Reg::Status).await?;
        Ok((StatusFlags::from_bits_truncate(raw), raw))
    }

    /// Extract channel ID from a raw STATUS byte (CHD[2:0] are bits [2:0]).
    #[inline(always)]
    pub fn status_chd(status_raw: u8) -> u8 {
        status_raw & 0x07
    }

    /// Write an 8-bit register (GPOCON only; STATUS/ID are read-only).
    pub async fn write_u8(&mut self, reg: Reg, val: u8) -> Result<(), Error<SpiErr, PinErr>> {
        let cmd = [Self::comm_byte(reg, false, false), val];
        self.spi.write(&cmd).await.map_err(Error::Spi)?;
        Ok(())
    }

    /// Read a 24-bit register (MODE, CONFIG, OFFSET, FULLSCALE).
    ///
    /// IMPORTANT: must keep CS asserted across "cmd" then "read".
    pub async fn read_u24(&mut self, reg: Reg) -> Result<u32, Error<SpiErr, PinErr>> {
        let cmd = [Self::comm_byte(reg, true, false)];
        let mut b = [0u8; 3];

        self.spi
            .transaction(&mut [Operation::Write(&cmd), Operation::Read(&mut b)])
            .await
            .map_err(Error::Spi)?;

        Ok(((b[0] as u32) << 16) | ((b[1] as u32) << 8) | (b[2] as u32))
    }

    /// Write a 24-bit register (MODE, CONFIG, OFFSET, FULLSCALE).
    pub async fn write_u24(&mut self, reg: Reg, val: u32) -> Result<(), Error<SpiErr, PinErr>> {
        let v = val & 0x00FF_FFFF;
        let cmd = [
            Self::comm_byte(reg, false, false),
            ((v >> 16) & 0xFF) as u8,
            ((v >> 8) & 0xFF) as u8,
            (v & 0xFF) as u8,
        ];
        self.spi.write(&cmd).await.map_err(Error::Spi)?;
        Ok(())
    }

    /// Set Mode register from a builder.
    pub async fn set_mode(&mut self, mode: Mode) -> Result<(), Error<SpiErr, PinErr>> {
        self.write_u24(Reg::Mode, mode.to_u24()).await
    }

    /// Set Config register from a builder.
    pub async fn set_config(&mut self, cfg: Config) -> Result<(), Error<SpiErr, PinErr>> {
        self.write_u24(Reg::Config, cfg.to_u24()).await
    }

    /// Read a conversion sample from the Data register.
    ///
    /// If `dat_sta=true`, read 24-bit data plus appended status byte.
    ///
    /// IMPORTANT: must keep CS asserted across "cmd" then "read".
    pub async fn read_data(&mut self, dat_sta: bool) -> Result<Sample, Error<SpiErr, PinErr>> {
        let cmd = [Self::comm_byte(Reg::Data, true, false)];

        if dat_sta {
            let mut b = [0u8; 4];
            self.spi
                .transaction(&mut [Operation::Write(&cmd), Operation::Read(&mut b)])
                .await
                .map_err(Error::Spi)?;

            let code = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | (b[2] as u32);
            Ok(Sample {
                code,
                status: Some(b[3]),
            })
        } else {
            let mut b = [0u8; 3];
            self.spi
                .transaction(&mut [Operation::Write(&cmd), Operation::Read(&mut b)])
                .await
                .map_err(Error::Spi)?;

            let code = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | (b[2] as u32);
            Ok(Sample { code, status: None })
        }
    }

    /// Wait for DRDY pin (DOUT/RDY) to go low.
    pub async fn wait_ready_pin(&mut self) -> Result<(), Error<SpiErr, PinErr>> {
        if let Some(drdy) = self.drdy.as_mut() {
            drdy.wait_for_low().await.map_err(Error::Drdy)?;
            Ok(())
        } else {
            Err(Error::Timeout)
        }
    }

    /// Poll STATUS.RDY until ready, or timeout.
    pub async fn wait_ready_poll<D: DelayNs>(
        &mut self,
        delay: &mut D,
        max_tries: u32,
        delay_us: u32,
    ) -> Result<(), Error<SpiErr, PinErr>> {
        for _ in 0..max_tries {
            let status = self.read_u8(Reg::Status).await?;
            let flags = StatusFlags::from_bits_truncate(status);
            // RDY bit = 1 means not ready; ready when RDY cleared.
            if !flags.contains(StatusFlags::RDY) {
                return Ok(());
            }
            delay.delay_us(delay_us).await;
        }
        Err(Error::Timeout)
    }

    /// Start a single conversion and read the result.
    pub async fn single_conversion<D: DelayNs>(
        &mut self,
        delay: &mut D,
        mut mode: Mode,
        dat_sta: bool,
        use_pin_if_available: bool,
    ) -> Result<Sample, Error<SpiErr, PinErr>> {
        mode.op_mode = OperatingMode::SingleConversion;
        mode = mode.with_dat_sta(dat_sta);
        self.set_mode(mode).await?;

        if use_pin_if_available && self.drdy.is_some() {
            self.wait_ready_pin().await?;
        } else {
            self.wait_ready_poll(delay, 50_000, 10).await?;
        }

        self.read_data(dat_sta).await
    }

    /// Run a calibration mode and wait for completion.
    pub async fn calibrate<D: DelayNs>(
        &mut self,
        delay: &mut D,
        mut mode: Mode,
        cal: OperatingMode,
        use_pin_if_available: bool,
    ) -> Result<(), Error<SpiErr, PinErr>> {
        mode.op_mode = cal;
        self.set_mode(mode).await?;

        if use_pin_if_available && self.drdy.is_some() {
            self.wait_ready_pin().await?;
        } else {
            self.wait_ready_poll(delay, 200_000, 10).await?;
        }

        Ok(())
    }

    // -------------------------
    // Continuous conversion API
    // -------------------------

    /// Put ADC into continuous conversion mode and (optionally) enable CREAD streaming.
    ///
    /// Typical setup:
    /// - reset()
    /// - set_config()
    /// - start_continuous(mode, cfg)
    pub async fn start_continuous(
        &mut self,
        mut mode: Mode,
        cfg: ContinuousCfg,
    ) -> Result<ContinuousCfg, Error<SpiErr, PinErr>> {
        mode.op_mode = OperatingMode::ContinuousConversion;
        mode = mode.with_dat_sta(cfg.dat_sta);
        self.set_mode(mode).await?;

        if cfg.use_cread {
            // Enable continuous-read mode targeting the DATA register.
            let cmd = [Self::comm_byte(Reg::Data, true, true)];
            self.spi.write(&cmd).await.map_err(Error::Spi)?;
        }

        Ok(cfg)
    }

    /// Stop continuous read mode (CREAD) if you enabled it.
    ///
    /// Any communications byte with CREAD=0 will clear the mode; we use STATUS read as a harmless one.
    pub async fn stop_cread(&mut self) -> Result<(), Error<SpiErr, PinErr>> {
        // NOTE: we only need to send a comm byte with CREAD=0; a single write is fine.
        let cmd = [Self::comm_byte(Reg::Status, true, false)];
        self.spi.write(&cmd).await.map_err(Error::Spi)?;
        Ok(())
    }

    async fn wait_ready_continuous<D: DelayNs>(
        &mut self,
        delay: &mut D,
        cfg: &ContinuousCfg,
    ) -> Result<(), Error<SpiErr, PinErr>> {
        if cfg.use_pin_if_available && self.drdy.is_some() {
            self.wait_ready_pin().await
        } else {
            self.wait_ready_poll(delay, cfg.poll_max_tries, cfg.poll_delay_us)
                .await
        }
    }

    /// Await the next sample in continuous conversion mode.
    ///
    /// - If `cfg.use_cread=true`, this clocks out only the data bytes (and optional status).
    /// - Otherwise, it performs an explicit DATA register read each time (cmd+read under one CS).
    pub async fn next_sample<D: DelayNs>(
        &mut self,
        delay: &mut D,
        cfg: &ContinuousCfg,
    ) -> Result<Sample, Error<SpiErr, PinErr>> {
        self.wait_ready_continuous(delay, cfg).await?;

        if cfg.use_cread {
            // In CREAD mode, you do NOT send a comm byte each time; you just clock data out.
            if cfg.dat_sta {
                let mut b = [0u8; 4];
                self.spi.read(&mut b).await.map_err(Error::Spi)?;
                let code = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | (b[2] as u32);
                Ok(Sample {
                    code,
                    status: Some(b[3]),
                })
            } else {
                let mut b = [0u8; 3];
                self.spi.read(&mut b).await.map_err(Error::Spi)?;
                let code = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | (b[2] as u32);
                Ok(Sample { code, status: None })
            }
        } else {
            self.read_data(cfg.dat_sta).await
        }
    }

    // -------------------------
    // GPIO (GPOCON) API
    // -------------------------

    /// Read raw GPOCON register byte.
    pub async fn read_gpocon_raw(&mut self) -> Result<u8, Error<SpiErr, PinErr>> {
        self.read_u8(Reg::Gpocon).await
    }

    /// Write raw GPOCON register byte.
    ///
    /// Note: GP7 must be 0 for correct operation.
    pub async fn write_gpocon_raw(&mut self, mut v: u8) -> Result<(), Error<SpiErr, PinErr>> {
        v &= 0x7F; // force GP7=0
        self.write_u8(Reg::Gpocon, v).await
    }

    /// Read parsed GPOCON state.
    pub async fn read_gpocon(&mut self) -> Result<Gpocon, Error<SpiErr, PinErr>> {
        let raw = self.read_gpocon_raw().await?;
        Ok(Gpocon::from_u8(raw))
    }

    /// Write parsed GPOCON state.
    pub async fn write_gpocon(&mut self, gpocon: Gpocon) -> Result<(), Error<SpiErr, PinErr>> {
        self.write_gpocon_raw(gpocon.to_u8()).await
    }

    /// Enable/disable the P0/P1 output drivers (GP10EN).
    pub async fn set_gpio_enable_p1_p0(
        &mut self,
        enable: bool,
    ) -> Result<(), Error<SpiErr, PinErr>> {
        let mut g = self.read_gpocon().await?;
        g.enable_p1_p0 = enable;
        self.write_gpocon(g).await
    }

    /// Enable/disable the P2/P3 output drivers (GP32EN).
    pub async fn set_gpio_enable_p3_p2(
        &mut self,
        enable: bool,
    ) -> Result<(), Error<SpiErr, PinErr>> {
        let mut g = self.read_gpocon().await?;
        g.enable_p3_p2 = enable;
        self.write_gpocon(g).await
    }

    /// Enable/disable the output driver group that contains `pin`.
    ///
    /// - P0/P1 share GP10EN
    /// - P2/P3 share GP32EN
    pub async fn set_gpio_enabled_for_pin(
        &mut self,
        pin: GpioPin,
        enable: bool,
    ) -> Result<(), Error<SpiErr, PinErr>> {
        let mut g = self.read_gpocon().await?;
        g.set_enabled_group(pin, enable);
        self.write_gpocon(g).await
    }

    /// Set a GPIO output level bit (P0DAT..P3DAT) and write GPOCON back.
    ///
    /// Note:
    /// - If the corresponding enable bit is cleared, the pin is tri-stated and the DAT bit is ignored by the pin.
    pub async fn gpio_write(
        &mut self,
        pin: GpioPin,
        high: bool,
    ) -> Result<(), Error<SpiErr, PinErr>> {
        let mut g = self.read_gpocon().await?;
        g.set_level(pin, high);
        self.write_gpocon(g).await
    }

    /// Read a GPIO "level" from GPOCON.
    ///
    /// Datasheet behavior:
    /// - When enabled, PnDAT reflects the *actual* pin level (useful for short-circuit detection).
    /// - When disabled (tri-stated), behavior is not guaranteed; this returns the register bit as read.
    pub async fn gpio_read(&mut self, pin: GpioPin) -> Result<bool, Error<SpiErr, PinErr>> {
        let g = self.read_gpocon().await?;
        Ok(g.level(pin))
    }

    /// Convenience: set pin high.
    pub async fn gpio_set_high(&mut self, pin: GpioPin) -> Result<(), Error<SpiErr, PinErr>> {
        self.gpio_write(pin, true).await
    }

    /// Convenience: set pin low.
    pub async fn gpio_set_low(&mut self, pin: GpioPin) -> Result<(), Error<SpiErr, PinErr>> {
        self.gpio_write(pin, false).await
    }

    /// Control the bridge power-down switch (BPDSW bit in GPOCON).
    pub async fn set_bpdsw(&mut self, enable: bool) -> Result<(), Error<SpiErr, PinErr>> {
        let mut g = self.read_gpocon().await?;
        g.bpdsw = enable;
        self.write_gpocon(g).await
    }

    // -------------------------
    // Utility helpers
    // -------------------------

    /// If DAT_STA is enabled, return the channel for this sample.
    #[inline(always)]
    pub fn sample_channel(sample: &Sample) -> Option<u8> {
        sample.status.map(Self::status_chd)
    }

    /// Optional policy helper: turn a status byte into a fault if ERR/NOREF are set.
    pub fn status_to_fault(status_raw: u8) -> Option<Error<SpiErr, PinErr>> {
        let flags = StatusFlags::from_bits_truncate(status_raw);
        if flags.contains(StatusFlags::ERR) || flags.contains(StatusFlags::NOREF) {
            Some(Error::AdcFault { status: status_raw })
        } else {
            None
        }
    }

    /// Convert unipolar raw 24-bit code to volts.
    ///
    /// AIN ≈ code / 2^24 * Vref / gain
    pub fn code_to_volts_unipolar(code: u32, vref: f32, gain: Gain) -> f32 {
        let full = 16_777_216.0_f32; // 2^24
        (code as f32) * vref / (full * gain.as_u16() as f32)
    }

    /// Convert bipolar (offset binary) 24-bit code to volts (AD7190).
    ///
    /// 0x000000 => -Vref/gain
    /// 0x800000 => 0
    /// 0xFFFFFF => +Vref/gain * (1 - 1/2^23)
    pub fn code_to_volts_bipolar(code: u32, vref: f32, gain: Gain) -> f32 {
        let mid = 8_388_608.0_f32; // 2^23
        let full = 8_388_608.0_f32; // 2^23
        ((code as f32 - mid) / full) * (vref / gain.as_u16() as f32)
    }

    /// Convert internal temperature sensor code to Celsius.
    pub fn code_to_celsius_temp_sensor(code: u32) -> f32 {
        (code as f32 - 8_388_608.0) / 2815.0 - 273.0
    }
}
