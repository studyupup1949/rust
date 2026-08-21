//! High-level configuration and validation API for the ADXL372 driver.
//!
//! This module defines the high-level [`Config`] structure, a builder-style API for
//! constructing it, and validation rules that enforce datasheet constraints.
//!
//! # Examples
//!
//! ```rust,no_run
//! use adxl372::config::Config;
//! use adxl372::params::{Bandwidth, OutputDataRate, PowerMode};
//!
//! let config = Config::new()
//!     .odr(OutputDataRate::Od6400Hz)
//!     .bandwidth(Bandwidth::Bw1600Hz)
//!     .power_mode(PowerMode::Measure)
//!     .build();
//!
//! config.validate().unwrap();
//! ```

use crate::params::{
    AutoSleep, Bandwidth, ExtClk, ExtSync, HpfDisable, I2cHsmEn, InstantOnThreshold, LinkLoopMode,
    LowNoise, LpfDisable, OutputDataRate, PowerMode, SettleFilter, UserOrDisable, WakeUpRate,
};

/// User-facing configuration for the ADXL372 sensor.
///
/// Prefer using the builder [`Config::new`] to construct instances and validate
/// with [`Config::validate`] before applying configuration to the device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    /// Output data rate selection.
    pub odr: OutputDataRate,
    /// Wake-up timer period used when autosleep/link-loop is enabled.
    pub wakeup_rate: WakeUpRate,
    /// External reference clock enable.
    pub ext_clk: ExtClk,
    /// External sync/trigger enable.
    pub ext_sync: ExtSync,
    /// User overrange disable behavior.
    pub user_or_disable: UserOrDisable,
    /// Autosleep operating mode.
    pub autosleep: AutoSleep,
    /// Activity/inactivity interaction mode.
    pub linkloop: LinkLoopMode,
    /// Noise performance mode.
    pub low_noise: LowNoise,
    /// Analog bandwidth selection.
    pub bandwidth: Bandwidth,
    /// I2C high-speed enable selection.
    pub i2c_hsm_en: I2cHsmEn,
    /// Instant-on threshold selection.
    pub instant_on_threshold: InstantOnThreshold,
    /// Filter settle timing selection.
    pub filter_settle: SettleFilter,
    /// Low-pass filter disable selection.
    pub lpf_disable: LpfDisable,
    /// High-pass filter disable selection.
    pub hpf_disable: HpfDisable,
    /// Operating power mode selection.
    pub power_mode: PowerMode,
}

impl Config {
    /// Begins building a [`Config`] using the builder pattern.
    pub fn new() -> ConfigBuilder {
        ConfigBuilder::new()
    }

    /// Checks whether this configuration is valid according to datasheet rules.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::NyquistViolation`] when the selected bandwidth exceeds
    /// the $\frac{ODR}{2}$ Nyquist limit.
    pub fn validate(&self) -> core::result::Result<(), ConfigError> {
        if self.bandwidth.max_hz() * 2 > self.odr.hz() {
            return Err(ConfigError::NyquistViolation);
        }

        Ok(())
    }
}

/// Builder for [`Config`] allowing piecemeal construction.
///
/// The builder starts from [`Config::default`] and allows overriding individual fields.
#[derive(Debug, Clone, Copy)]
pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    /// Creates a new builder seeded with [`Config::default()`].
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    /// Overrides the output data rate.
    pub fn odr(mut self, odr: OutputDataRate) -> Self {
        self.config.odr = odr;
        self
    }

    /// Overrides the analog bandwidth.
    pub fn bandwidth(mut self, bandwidth: Bandwidth) -> Self {
        self.config.bandwidth = bandwidth;
        self
    }

    /// Sets the wake-up timer period.
    pub fn wakeup_rate(mut self, wakeup_rate: WakeUpRate) -> Self {
        self.config.wakeup_rate = wakeup_rate;
        self
    }

    /// Enables the external clock selection.
    pub fn ext_clk(mut self, ext_clk: ExtClk) -> Self {
        self.config.ext_clk = ext_clk;
        self
    }

    /// Enables the external sync selection.
    pub fn ext_sync(mut self, ext_sync: ExtSync) -> Self {
        self.config.ext_sync = ext_sync;
        self
    }

    /// Configures autosleep behaviour.
    pub fn autosleep(mut self, autosleep: AutoSleep) -> Self {
        self.config.autosleep = autosleep;
        self
    }

    /// Sets the instant-on threshold selection.
    pub fn instant_on_threshold(mut self, threshold: InstantOnThreshold) -> Self {
        self.config.instant_on_threshold = threshold;
        self
    }

    /// Sets the filter settle timing selection.
    pub fn filter_settle(mut self, settle: SettleFilter) -> Self {
        self.config.filter_settle = settle;
        self
    }

    /// Configures the low-pass filter disable bit.
    pub fn lpf_disable(mut self, setting: LpfDisable) -> Self {
        self.config.lpf_disable = setting;
        self
    }

    /// Configures the high-pass filter disable bit.
    pub fn hpf_disable(mut self, setting: HpfDisable) -> Self {
        self.config.hpf_disable = setting;
        self
    }

    /// Enables or disables I2C high-speed mode.
    pub fn i2c_hsm_en(mut self, setting: I2cHsmEn) -> Self {
        self.config.i2c_hsm_en = setting;
        self
    }

    /// Sets the user overrange disable behavior.
    pub fn user_or_disable(mut self, setting: UserOrDisable) -> Self {
        self.config.user_or_disable = setting;
        self
    }

    /// Sets the link/loop activity processing mode.
    pub fn linkloop(mut self, mode: LinkLoopMode) -> Self {
        self.config.linkloop = mode;
        self
    }

    /// Sets the desired noise performance mode.
    pub fn low_noise(mut self, mode: LowNoise) -> Self {
        self.config.low_noise = mode;
        self
    }

    /// Sets the desired operating power mode.
    pub fn power_mode(mut self, mode: PowerMode) -> Self {
        self.config.power_mode = mode;
        self
    }

    /// Finalizes the builder and returns the [`Config`].
    pub fn build(self) -> Config {
        self.config
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            odr: OutputDataRate::Od400Hz,
            wakeup_rate: WakeUpRate::Ms52,
            ext_clk: ExtClk::Disabled,
            ext_sync: ExtSync::Disabled,
            user_or_disable: UserOrDisable::Enabled,
            autosleep: AutoSleep::Disabled,
            linkloop: LinkLoopMode::Default,
            low_noise: LowNoise::Normal,
            bandwidth: Bandwidth::Bw200Hz,
            i2c_hsm_en: I2cHsmEn::Disabled,
            instant_on_threshold: InstantOnThreshold::Low,
            filter_settle: SettleFilter::Ms370,
            lpf_disable: LpfDisable::Enabled,
            hpf_disable: HpfDisable::Enabled,
            power_mode: PowerMode::Standby,
        }
    }
}

/// Validation errors generated while verifying a [`Config`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigError {
    /// Requested bandwidth violates Nyquist sampling limits for the chosen ODR.
    NyquistViolation,
}
