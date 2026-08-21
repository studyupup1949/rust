#![no_std]
#![deny(missing_docs)]

//! A driver for the TI ADS112C04 16-bit I2C Delta-Sigma ADC
//!
//! This crate provides a blocking, embedded-hal 1.0 compatible driver for the
//! ADS112C04 analog-to-digital converter.
//!
//! # Example
//!
//! ```no_run
//! use ads112c04::{Ads112c04, Config0, Gain, InputMux};
//! # use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
//! # let expectations = [];
//! # let i2c = I2cMock::new(&expectations);
//!
//! let mut adc = Ads112c04::new(i2c, 0x40);
//! adc.reset().unwrap();
//!
//! // Configure for single-ended measurement on AIN0
//! let config0 = Config0::default()
//!     .with_input_mux(InputMux::Ain0ToAvss)
//!     .with_gain(Gain::X1);
//! adc.write_config0(config0).unwrap();
//!
//! // Start conversion and read result
//! adc.start_sync().unwrap();
//! let reading = adc.read_data().unwrap();
//! ```

use bitflags::bitflags;
use embedded_hal::i2c::I2c;

/// ADS112C04 I2C commands
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Command {
    /// Reset device to default state
    Reset = 0x06,
    /// Start or restart conversions
    StartSync = 0x08,
    /// Enter power-down mode
    PowerDown = 0x02,
    /// Read conversion data
    ReadData = 0x10,
}

/// Register addresses
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Register {
    /// Configuration register 0 (Input & Gain)
    Config0 = 0x00,
    /// Configuration register 1 (Rate, Mode, Ref)
    Config1 = 0x01,
    /// Configuration register 2 (Status & IDAC Current)
    Config2 = 0x02,
    /// Configuration register 3 (IDAC Routing)
    Config3 = 0x03,
}

impl Register {
    /// Get the read register command for this register
    pub const fn read_cmd(self) -> u8 {
        0x20 | ((self as u8) << 2)
    }

    /// Get the write register command for this register
    pub const fn write_cmd(self) -> u8 {
        0x40 | ((self as u8) << 2)
    }
}

/// Input multiplexer configuration
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMux {
    /// AINp=AIN0, AINn=AIN1 (Default differential)
    Ain0ToAin1 = 0b0000,
    /// AINp=AIN0, AINn=AIN2
    Ain0ToAin2 = 0b0001,
    /// AINp=AIN0, AINn=AIN3
    Ain0ToAin3 = 0b0010,
    /// AINp=AIN1, AINn=AIN0
    Ain1ToAin0 = 0b0011,
    /// AINp=AIN1, AINn=AIN2
    Ain1ToAin2 = 0b0100,
    /// AINp=AIN1, AINn=AIN3
    Ain1ToAin3 = 0b0101,
    /// AINp=AIN2, AINn=AIN3
    Ain2ToAin3 = 0b0110,
    /// AINp=AIN3, AINn=AIN2
    Ain3ToAin2 = 0b0111,
    /// AINp=AIN0, AINn=AVSS (Single-ended)
    Ain0ToAvss = 0b1000,
    /// AINp=AIN1, AINn=AVSS (Single-ended)
    Ain1ToAvss = 0b1001,
    /// AINp=AIN2, AINn=AVSS (Single-ended)
    Ain2ToAvss = 0b1010,
    /// AINp=AIN3, AINn=AVSS (Single-ended)
    Ain3ToAvss = 0b1011,
    /// AINp=REFPx, AINn=REFNx
    RefpToRefn = 0b1100,
    /// AINp=AVDDx, AINn=AVSSx
    AvddToAvss = 0b1101,
    /// Shorted to mid-supply
    Shorted = 0b1110,
    /// Reserved
    Reserved = 0b1111,
}

/// Programmable Gain Amplifier settings
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gain {
    /// Gain = 1
    X1 = 0b000,
    /// Gain = 2
    X2 = 0b001,
    /// Gain = 4
    X4 = 0b010,
    /// Gain = 8
    X8 = 0b011,
    /// Gain = 16
    X16 = 0b100,
    /// Gain = 32
    X32 = 0b101,
    /// Gain = 64
    X64 = 0b110,
    /// Gain = 128
    X128 = 0b111,
}

impl Gain {
    /// Get the numeric gain value
    pub const fn value(self) -> u8 {
        1 << (self as u8)
    }
}

/// Data rate settings for normal mode
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataRate {
    /// 20 samples per second
    Sps20 = 0b000,
    /// 45 samples per second
    Sps45 = 0b001,
    /// 90 samples per second
    Sps90 = 0b010,
    /// 175 samples per second
    Sps175 = 0b011,
    /// 330 samples per second
    Sps330 = 0b100,
    /// 600 samples per second
    Sps600 = 0b101,
    /// 1000 samples per second
    Sps1000 = 0b110,
    /// Reserved
    Reserved = 0b111,
}

/// Operating mode
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    /// Normal mode
    Normal = 0,
    /// Turbo mode
    Turbo = 1,
}

/// Conversion mode
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConversionMode {
    /// Single-shot conversion
    SingleShot = 0,
    /// Continuous conversion
    Continuous = 1,
}

/// Voltage reference selection
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VoltageReference {
    /// Internal 2.048V reference
    Internal2048mV = 0b00,
    /// External reference (REFP/REFN)
    External = 0b01,
    /// Analog supply (AVDD/AVSS)
    AnalogSupply = 0b10,
    /// Reserved
    Reserved = 0b11,
}

/// IDAC current magnitude
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IdacCurrent {
    /// IDAC off
    Off = 0b000,
    /// 10 µA
    MicroA10 = 0b001,
    /// 50 µA
    MicroA50 = 0b010,
    /// 100 µA
    MicroA100 = 0b011,
    /// 250 µA
    MicroA250 = 0b100,
    /// 500 µA
    MicroA500 = 0b101,
    /// 1000 µA
    MicroA1000 = 0b110,
    /// 1500 µA
    MicroA1500 = 0b111,
}

/// IDAC routing options
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IdacRouting {
    /// Route to AIN0
    Ain0 = 0b000,
    /// Route to AIN1
    Ain1 = 0b001,
    /// Route to AIN2
    Ain2 = 0b010,
    /// Route to AIN3
    Ain3 = 0b011,
    /// Route to REFP
    Refp = 0b100,
    /// Route to REFN
    Refn = 0b101,
    /// No connection
    NoConnection = 0b110,
    /// Reserved
    Reserved = 0b111,
}

/// Configuration Register 0 (Input & Gain)
#[derive(Debug, Clone, Copy, Default)]
pub struct Config0 {
    value: u8,
}

impl Config0 {
    /// Create a new Config0 from raw value
    pub const fn from_raw(value: u8) -> Self {
        Self { value }
    }

    /// Get the raw register value
    pub const fn raw(self) -> u8 {
        self.value
    }

    /// Set the input multiplexer
    pub const fn with_input_mux(mut self, mux: InputMux) -> Self {
        self.value = (self.value & 0x0F) | ((mux as u8) << 4);
        self
    }

    /// Get the input multiplexer setting
    pub const fn input_mux(self) -> InputMux {
        match (self.value >> 4) & 0x0F {
            0b0000 => InputMux::Ain0ToAin1,
            0b0001 => InputMux::Ain0ToAin2,
            0b0010 => InputMux::Ain0ToAin3,
            0b0011 => InputMux::Ain1ToAin0,
            0b0100 => InputMux::Ain1ToAin2,
            0b0101 => InputMux::Ain1ToAin3,
            0b0110 => InputMux::Ain2ToAin3,
            0b0111 => InputMux::Ain3ToAin2,
            0b1000 => InputMux::Ain0ToAvss,
            0b1001 => InputMux::Ain1ToAvss,
            0b1010 => InputMux::Ain2ToAvss,
            0b1011 => InputMux::Ain3ToAvss,
            0b1100 => InputMux::RefpToRefn,
            0b1101 => InputMux::AvddToAvss,
            0b1110 => InputMux::Shorted,
            _ => InputMux::Reserved,
        }
    }

    /// Set the gain
    pub const fn with_gain(mut self, gain: Gain) -> Self {
        self.value = (self.value & 0xF1) | ((gain as u8) << 1);
        self
    }

    /// Get the gain setting
    pub const fn gain(self) -> Gain {
        match (self.value >> 1) & 0x07 {
            0b000 => Gain::X1,
            0b001 => Gain::X2,
            0b010 => Gain::X4,
            0b011 => Gain::X8,
            0b100 => Gain::X16,
            0b101 => Gain::X32,
            0b110 => Gain::X64,
            _ => Gain::X128,
        }
    }

    /// Set PGA bypass
    pub const fn with_pga_bypass(mut self, bypass: bool) -> Self {
        if bypass {
            self.value |= 0x01;
        } else {
            self.value &= 0xFE;
        }
        self
    }

    /// Get PGA bypass setting
    pub const fn pga_bypass(self) -> bool {
        (self.value & 0x01) != 0
    }
}

/// Configuration Register 1 (Rate, Mode, Ref)
#[derive(Debug, Clone, Copy, Default)]
pub struct Config1 {
    value: u8,
}

impl Config1 {
    /// Create a new Config1 from raw value
    pub const fn from_raw(value: u8) -> Self {
        Self { value }
    }

    /// Get the raw register value
    pub const fn raw(self) -> u8 {
        self.value
    }

    /// Set the data rate
    pub const fn with_data_rate(mut self, rate: DataRate) -> Self {
        self.value = (self.value & 0x1F) | ((rate as u8) << 5);
        self
    }

    /// Get the data rate setting
    pub const fn data_rate(self) -> DataRate {
        match (self.value >> 5) & 0x07 {
            0b000 => DataRate::Sps20,
            0b001 => DataRate::Sps45,
            0b010 => DataRate::Sps90,
            0b011 => DataRate::Sps175,
            0b100 => DataRate::Sps330,
            0b101 => DataRate::Sps600,
            0b110 => DataRate::Sps1000,
            _ => DataRate::Reserved,
        }
    }

    /// Set the operating mode
    pub const fn with_mode(mut self, mode: Mode) -> Self {
        if mode as u8 != 0 {
            self.value |= 0x10;
        } else {
            self.value &= 0xEF;
        }
        self
    }

    /// Get the operating mode
    pub const fn mode(self) -> Mode {
        if (self.value & 0x10) != 0 {
            Mode::Turbo
        } else {
            Mode::Normal
        }
    }

    /// Set the conversion mode
    pub const fn with_conversion_mode(mut self, mode: ConversionMode) -> Self {
        if mode as u8 != 0 {
            self.value |= 0x08;
        } else {
            self.value &= 0xF7;
        }
        self
    }

    /// Get the conversion mode
    pub const fn conversion_mode(self) -> ConversionMode {
        if (self.value & 0x08) != 0 {
            ConversionMode::Continuous
        } else {
            ConversionMode::SingleShot
        }
    }

    /// Set the voltage reference
    pub const fn with_voltage_reference(mut self, vref: VoltageReference) -> Self {
        self.value = (self.value & 0xF9) | ((vref as u8) << 1);
        self
    }

    /// Get the voltage reference setting
    pub const fn voltage_reference(self) -> VoltageReference {
        match (self.value >> 1) & 0x03 {
            0b00 => VoltageReference::Internal2048mV,
            0b01 => VoltageReference::External,
            0b10 => VoltageReference::AnalogSupply,
            _ => VoltageReference::Reserved,
        }
    }

    /// Enable/disable temperature sensor
    pub const fn with_temperature_sensor(mut self, enable: bool) -> Self {
        if enable {
            self.value |= 0x01;
        } else {
            self.value &= 0xFE;
        }
        self
    }

    /// Get temperature sensor enable status
    pub const fn temperature_sensor_enabled(self) -> bool {
        (self.value & 0x01) != 0
    }
}

bitflags! {
    /// Configuration Register 2 (Status & IDAC Current)
    #[derive(Debug, Clone, Copy)]
    pub struct Config2: u8 {
        /// Data Ready flag (read-only, active low)
        const DRDY = 0x80;
        /// Data Counter Enable
        const DCNT = 0x40;
        /// CRC bit 1
        const CRC1 = 0x20;
        /// CRC bit 0
        const CRC0 = 0x10;
        /// Burn-out Current Sources
        const BCS = 0x08;
        /// IDAC Current bit 2
        const IDAC2 = 0x04;
        /// IDAC Current bit 1
        const IDAC1 = 0x02;
        /// IDAC Current bit 0
        const IDAC0 = 0x01;
    }
}

impl Default for Config2 {
    fn default() -> Self {
        Config2::empty()
    }
}

impl Config2 {
    /// Set the IDAC current
    pub fn with_idac_current(self, current: IdacCurrent) -> Self {
        // Clear IDAC bits and set new value
        let bits = (self.bits() & 0xF8) | (current as u8);
        Config2::from_bits_truncate(bits)
    }

    /// Get the IDAC current setting
    pub fn idac_current(self) -> IdacCurrent {
        match self.bits() & 0x07 {
            0b000 => IdacCurrent::Off,
            0b001 => IdacCurrent::MicroA10,
            0b010 => IdacCurrent::MicroA50,
            0b011 => IdacCurrent::MicroA100,
            0b100 => IdacCurrent::MicroA250,
            0b101 => IdacCurrent::MicroA500,
            0b110 => IdacCurrent::MicroA1000,
            _ => IdacCurrent::MicroA1500,
        }
    }

    /// Check if data is ready (DRDY bit is active low)
    pub fn data_ready(self) -> bool {
        !self.contains(Config2::DRDY)
    }
}

/// Configuration Register 3 (IDAC Routing)
#[derive(Debug, Clone, Copy, Default)]
pub struct Config3 {
    value: u8,
}

impl Config3 {
    /// Create a new Config3 from raw value
    pub const fn from_raw(value: u8) -> Self {
        Self { value }
    }

    /// Get the raw register value
    pub const fn raw(self) -> u8 {
        self.value
    }

    /// Set IDAC1 routing
    pub const fn with_idac1_routing(mut self, routing: IdacRouting) -> Self {
        self.value = (self.value & 0x1F) | ((routing as u8) << 5);
        self
    }

    /// Get IDAC1 routing
    pub const fn idac1_routing(self) -> IdacRouting {
        match (self.value >> 5) & 0x07 {
            0b000 => IdacRouting::Ain0,
            0b001 => IdacRouting::Ain1,
            0b010 => IdacRouting::Ain2,
            0b011 => IdacRouting::Ain3,
            0b100 => IdacRouting::Refp,
            0b101 => IdacRouting::Refn,
            0b110 => IdacRouting::NoConnection,
            _ => IdacRouting::Reserved,
        }
    }

    /// Set IDAC2 routing
    pub const fn with_idac2_routing(mut self, routing: IdacRouting) -> Self {
        self.value = (self.value & 0xE3) | ((routing as u8) << 2);
        self
    }

    /// Get IDAC2 routing
    pub const fn idac2_routing(self) -> IdacRouting {
        match (self.value >> 2) & 0x07 {
            0b000 => IdacRouting::Ain0,
            0b001 => IdacRouting::Ain1,
            0b010 => IdacRouting::Ain2,
            0b011 => IdacRouting::Ain3,
            0b100 => IdacRouting::Refp,
            0b101 => IdacRouting::Refn,
            0b110 => IdacRouting::NoConnection,
            _ => IdacRouting::Reserved,
        }
    }
}

/// ADS112C04 driver
pub struct Ads112c04<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C, E> Ads112c04<I2C>
where
    I2C: I2c<Error = E>,
{
    /// Create a new ADS112C04 driver instance
    ///
    /// # Arguments
    /// * `i2c` - I2C peripheral
    /// * `address` - 7-bit I2C address (typically 0x40-0x43)
    pub fn new(i2c: I2C, address: u8) -> Self {
        Self { i2c, address }
    }

    /// Release the I2C peripheral
    pub fn release(self) -> I2C {
        self.i2c
    }

    /// Send a command to the device
    pub fn send_command(&mut self, command: Command) -> Result<(), E> {
        self.i2c.write(self.address, &[command as u8])?;
        Ok(())
    }

    /// Reset the device to default state
    pub fn reset(&mut self) -> Result<(), E> {
        self.send_command(Command::Reset)
    }

    /// Start or restart conversions
    pub fn start_sync(&mut self) -> Result<(), E> {
        self.send_command(Command::StartSync)
    }

    /// Enter power-down mode
    pub fn power_down(&mut self) -> Result<(), E> {
        self.send_command(Command::PowerDown)
    }

    /// Read conversion data
    ///
    /// Returns a signed 16-bit value in two's complement format
    pub fn read_data(&mut self) -> Result<i16, E> {
        // Send RDATA command
        self.send_command(Command::ReadData)?;

        // Read 2 bytes (MSB first)
        let mut buffer = [0u8; 2];
        self.i2c.read(self.address, &mut buffer)?;

        // Convert to signed 16-bit value
        Ok(i16::from_be_bytes(buffer))
    }

    /// Read a register
    pub fn read_register(&mut self, register: Register) -> Result<u8, E> {
        let cmd = register.read_cmd();
        self.i2c.write(self.address, &[cmd])?;

        let mut buffer = [0u8; 1];
        self.i2c.read(self.address, &mut buffer)?;

        Ok(buffer[0])
    }

    /// Write to a register
    pub fn write_register(&mut self, register: Register, value: u8) -> Result<(), E> {
        let cmd = register.write_cmd();
        self.i2c.write(self.address, &[cmd, value])?;
        Ok(())
    }

    /// Read Configuration Register 0
    pub fn read_config0(&mut self) -> Result<Config0, E> {
        let value = self.read_register(Register::Config0)?;
        Ok(Config0::from_raw(value))
    }

    /// Write Configuration Register 0
    pub fn write_config0(&mut self, config: Config0) -> Result<(), E> {
        self.write_register(Register::Config0, config.raw())
    }

    /// Read Configuration Register 1
    pub fn read_config1(&mut self) -> Result<Config1, E> {
        let value = self.read_register(Register::Config1)?;
        Ok(Config1::from_raw(value))
    }

    /// Write Configuration Register 1
    pub fn write_config1(&mut self, config: Config1) -> Result<(), E> {
        self.write_register(Register::Config1, config.raw())
    }

    /// Read Configuration Register 2
    pub fn read_config2(&mut self) -> Result<Config2, E> {
        let value = self.read_register(Register::Config2)?;
        Ok(Config2::from_bits_truncate(value))
    }

    /// Write Configuration Register 2
    pub fn write_config2(&mut self, config: Config2) -> Result<(), E> {
        self.write_register(Register::Config2, config.bits())
    }

    /// Read Configuration Register 3
    pub fn read_config3(&mut self) -> Result<Config3, E> {
        let value = self.read_register(Register::Config3)?;
        Ok(Config3::from_raw(value))
    }

    /// Write Configuration Register 3
    pub fn write_config3(&mut self, config: Config3) -> Result<(), E> {
        self.write_register(Register::Config3, config.raw())
    }

    /// Check if data is ready by reading the DRDY bit in Config2
    pub fn data_ready(&mut self) -> Result<bool, E> {
        let config2 = self.read_config2()?;
        Ok(config2.data_ready())
    }

    /// Perform a single-shot conversion
    ///
    /// This method configures the device for single-shot mode, starts a conversion,
    /// waits for it to complete, and returns the result.
    pub fn read_single_shot(&mut self) -> Result<i16, E> {
        // Configure for single-shot mode
        let mut config1 = self.read_config1()?;
        config1 = config1.with_conversion_mode(ConversionMode::SingleShot);
        self.write_config1(config1)?;

        // Start conversion
        self.start_sync()?;

        // Wait for conversion to complete
        // Note: In a real application, you might want to add a timeout here
        while !self.data_ready()? {
            // Could add a small delay here if needed
        }

        // Read the result
        self.read_data()
    }
}

/// Convert raw ADC reading to voltage (standalone function)
///
/// # Arguments
/// * `raw_value` - Raw ADC reading from read_data()
/// * `vref_mv` - Reference voltage in millivolts
/// * `gain` - PGA gain setting
/// * `pga_bypassed` - Whether PGA is bypassed
///
/// # Returns
/// Voltage in millivolts
pub fn raw_to_voltage_mv(raw_value: i16, vref_mv: u16, gain: Gain, pga_bypassed: bool) -> f32 {
    let effective_gain = if pga_bypassed {
        1.0
    } else {
        gain.value() as f32
    };
    let lsb_mv = (vref_mv as f32) / (32768.0 * effective_gain);
    (raw_value as f32) * lsb_mv
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config0_default() {
        let config = Config0::default();
        assert_eq!(config.input_mux(), InputMux::Ain0ToAin1);
        assert_eq!(config.gain(), Gain::X1);
        assert_eq!(config.pga_bypass(), false);
    }

    #[test]
    fn test_config0_builders() {
        let config = Config0::default()
            .with_input_mux(InputMux::Ain0ToAvss)
            .with_gain(Gain::X4)
            .with_pga_bypass(true);

        assert_eq!(config.input_mux(), InputMux::Ain0ToAvss);
        assert_eq!(config.gain(), Gain::X4);
        assert_eq!(config.pga_bypass(), true);
    }

    #[test]
    fn test_config1_default() {
        let config = Config1::default();
        assert_eq!(config.data_rate(), DataRate::Sps20);
        assert_eq!(config.mode(), Mode::Normal);
        assert_eq!(config.conversion_mode(), ConversionMode::SingleShot);
        assert_eq!(config.voltage_reference(), VoltageReference::Internal2048mV);
        assert_eq!(config.temperature_sensor_enabled(), false);
    }

    #[test]
    fn test_gain_values() {
        assert_eq!(Gain::X1.value(), 1);
        assert_eq!(Gain::X2.value(), 2);
        assert_eq!(Gain::X4.value(), 4);
        assert_eq!(Gain::X8.value(), 8);
        assert_eq!(Gain::X16.value(), 16);
        assert_eq!(Gain::X32.value(), 32);
        assert_eq!(Gain::X64.value(), 64);
        assert_eq!(Gain::X128.value(), 128);
    }

    #[test]
    fn test_register_commands() {
        assert_eq!(Register::Config0.read_cmd(), 0x20);
        assert_eq!(Register::Config1.read_cmd(), 0x24);
        assert_eq!(Register::Config0.write_cmd(), 0x40);
        assert_eq!(Register::Config1.write_cmd(), 0x44);
    }

    #[test]
    fn test_voltage_conversion() {
        // Test with internal 2.048V reference, gain = 1, PGA enabled
        let voltage = raw_to_voltage_mv(16384, 2048, Gain::X1, false);
        assert!((voltage - 1024.0).abs() < 0.1); // Should be ~1024mV (half of full scale)

        // Test with gain = 2
        let voltage = raw_to_voltage_mv(16384, 2048, Gain::X2, false);
        assert!((voltage - 512.0).abs() < 0.1); // Should be ~512mV
    }
}
