use crate::stage::{InternalStageConfiguration, StageConfiguration};

#[derive(Debug)]
pub struct DeviceConfiguration<const STAGES: usize>(pub InternalDeviceConfiguration<STAGES>);

impl DeviceConfiguration<0> {
    pub fn builder() -> ConfigurationBuilder<WithoutStages, 0> {
        ConfigurationBuilder {
            configuration: InternalDeviceConfiguration::default(),
            _state: WithoutStages,
        }
    }
}

#[derive(Debug)]
pub struct InternalDeviceConfiguration<const STAGES: usize> {
    power_control: PowerControl,
    pub(crate) calibration_enable: CalibrationEnable,
    pub(crate) ambient_comp_control: AmbientCompensationControl,
    pub(crate) stage_low_int_enable: StageLowIntEnable,
    pub(crate) stage_high_int_enable: StageHighIntEnable,
    pub(crate) stage_complete_int_enable: StageCompleteIntEnable,
    pub(crate) stages: [InternalStageConfiguration; STAGES],
}

impl Default for InternalDeviceConfiguration<0> {
    fn default() -> Self {
        Self {
            power_control: Default::default(),
            calibration_enable: Default::default(),
            ambient_comp_control: Default::default(),
            stage_low_int_enable: Default::default(),
            stage_high_int_enable: Default::default(),
            stage_complete_int_enable: Default::default(),
            stages: Default::default(),
        }
    }
}

impl<const S: usize> InternalDeviceConfiguration<S> {
    pub fn to_reg_value_for_init(&self) -> [u16; 8] {
        let mut res = [0u16; 8];
        res[0] = self.power_control.to_reg_value();
        res[1] = 0x0000;
        res[2..5].copy_from_slice(&self.ambient_comp_control.to_reg_value());
        res[5] = self.stage_low_int_enable.to_reg_value();
        res[6] = self.stage_high_int_enable.to_reg_value();
        res[7] = self.stage_complete_int_enable.to_reg_value();
        res
    }

    pub(crate) fn _to_reg_value(&self) -> [u16; 8] {
        let mut res = [0u16; 8];
        res[0] = self.power_control.to_reg_value();
        res[1] = self.calibration_enable.to_reg_value();
        res[2..5].copy_from_slice(&self.ambient_comp_control.to_reg_value());
        res[5] = self.stage_low_int_enable.to_reg_value();
        res[6] = self.stage_high_int_enable.to_reg_value();
        res[7] = self.stage_complete_int_enable.to_reg_value();
        res
    }
}

pub struct WithoutStages;
pub struct WithStages;

pub struct ConfigurationBuilder<STATE, const S: usize> {
    configuration: InternalDeviceConfiguration<S>,
    _state: STATE,
}

impl<STATE, const S: usize> ConfigurationBuilder<STATE, S> {
    pub fn power_mode(mut self, mode: PowerMode) -> Self {
        self.configuration.power_control.mode = mode;
        self
    }

    pub fn low_power_conversion_delay(mut self, lp_conv_delay: LowPowerConversionDelay) -> Self {
        self.configuration.power_control.lp_conversion_delay = lp_conv_delay;
        self
    }

    pub fn decimation(mut self, decimation: DecimationFactor) -> Self {
        self.configuration.power_control.decimation = decimation;
        self
    }

    pub fn interrupt_polarity_active_low(mut self) -> Self {
        self.configuration.power_control.interrupt_polarity = false;
        self
    }

    pub fn interrupt_polarity_active_high(mut self) -> Self {
        self.configuration.power_control.interrupt_polarity = true;
        self
    }

    pub fn excitation_source_enabled(mut self, enabled: bool) -> Self {
        self.configuration.power_control.excitation_source = !enabled;
        self
    }

    pub fn cdc_bias_current(mut self, bias: CdcBias) -> Self {
        self.configuration.power_control.cdc_bias = bias;
        self
    }

    pub fn full_power_mode_average_sample_skip(mut self, skip: FullPowerSkip) -> Self {
        self.configuration.calibration_enable.avg_fp_skip = skip;
        self
    }

    pub fn low_power_mode_average_sample_skip(mut self, skip: LowPowerSkip) -> Self {
        self.configuration.calibration_enable.avg_lp_skip = skip;
        self
    }

    pub fn fast_filter_sample_skip(mut self, skip: u8) -> Self {
        assert!(skip <= 12, "fast filter skip cannot be greater than 12");
        self.configuration.ambient_comp_control.fast_filter_skip = skip;
        self
    }

    pub fn no_forced_calibration(mut self) -> Self {
        self.configuration.ambient_comp_control.forced_calibration = false;
        self
    }

    pub fn no_conversion_reset(mut self) -> Self {
        self.configuration.ambient_comp_control.conversion_reset = false;
        self
    }

    pub fn calibration_disable_period_full_power(mut self, fp_prox_cnt: u8) -> Self {
        assert!(
            fp_prox_cnt < 8,
            "calibration disable period cannot be greater than 8"
        );
        self.configuration
            .ambient_comp_control
            .full_power_proximity_disable = fp_prox_cnt;
        self
    }

    pub fn calibration_disable_period_low_power(mut self, lp_prox_cnt: u8) -> Self {
        assert!(
            lp_prox_cnt < 8,
            "calibration disable period cannot be greater than 8"
        );
        self.configuration
            .ambient_comp_control
            .low_power_proximity_disable = lp_prox_cnt;
        self
    }

    pub fn power_down_timeout(mut self, timeout: PowerDownTimeout) -> Self {
        self.configuration.ambient_comp_control.power_down_timeout = timeout;
        self
    }

    pub fn proximity_recalibration_level(mut self, level: u8) -> Self {
        self.configuration
            .ambient_comp_control
            .proximity_recalibration_level = level;
        self
    }

    pub fn proximity_detection_rate(mut self, rate: u8) -> Self {
        assert!(
            rate < 64,
            "proximity detection rate cannot be greater than 64"
        );
        self.configuration
            .ambient_comp_control
            .proximity_detection_rate = rate;
        self
    }

    pub fn slow_filter_update_level(mut self, level: u8) -> Self {
        assert!(
            level < 4,
            "slow filter update level cannot be grater than 3"
        );
        self.configuration
            .ambient_comp_control
            .slow_filter_update_level = level;
        self
    }

    pub fn full_power_proximity_recalibration_time(mut self, time: u8) -> Self {
        self.configuration
            .ambient_comp_control
            .full_power_proximity_recalibration = time;
        self
    }

    pub fn low_power_proximity_recalibration_time(mut self, time: u8) -> Self {
        self.configuration
            .ambient_comp_control
            .low_power_proximity_recalibration = time;
        self
    }

    pub fn gpio_setup(mut self, setup: GpioSetup) -> Self {
        self.configuration.stage_low_int_enable.gpio_setup = setup;
        self
    }

    pub fn gpio_input_config(mut self, config: GpioInputConfig) -> Self {
        self.configuration.stage_low_int_enable.gpio_input_config = config;
        self
    }

    pub fn stages<const N: usize>(
        self,
        ext_stages: [StageConfiguration; N],
    ) -> ConfigurationBuilder<WithStages, N> {
        let mut stages = [InternalStageConfiguration::default(); N];

        for (i, ext_stage) in ext_stages.into_iter().enumerate() {
            stages[i] = ext_stage.0;
        }

        let mut calibration = [false; 12];
        let mut low_int = [false; 12];
        let mut high_int = [false; 12];
        let mut conv_int = [false; 12];

        for (i, stage) in stages.iter().enumerate() {
            calibration[i] = stage.global.calibration_enabled;
            low_int[i] = stage.global.low_int_enabled;
            high_int[i] = stage.global.high_int_enabled;
            conv_int[i] = stage.global.complete_int_enabled;
        }

        let mut new_self = ConfigurationBuilder {
            configuration: InternalDeviceConfiguration {
                power_control: self.configuration.power_control,
                calibration_enable: self.configuration.calibration_enable,
                ambient_comp_control: self.configuration.ambient_comp_control,
                stage_low_int_enable: self.configuration.stage_low_int_enable,
                stage_high_int_enable: self.configuration.stage_high_int_enable,
                stage_complete_int_enable: self.configuration.stage_complete_int_enable,
                stages,
            },
            _state: WithStages,
        };

        new_self.configuration.calibration_enable.stages_enable = calibration;
        new_self.configuration.stage_low_int_enable.stages_enable = low_int;
        new_self.configuration.stage_high_int_enable.stages_enable = high_int;
        new_self
            .configuration
            .stage_complete_int_enable
            .stages_enable = conv_int;
        new_self.configuration.power_control.num_stages = N as u8;

        new_self
    }
}

impl<const S: usize> ConfigurationBuilder<WithStages, S> {
    pub fn build(self) -> DeviceConfiguration<S> {
        DeviceConfiguration(self.configuration)
    }
}

#[derive(Debug, Default)]
struct PowerControl {
    mode: PowerMode,
    lp_conversion_delay: LowPowerConversionDelay,
    num_stages: u8,
    decimation: DecimationFactor,
    soft_reset: bool,
    interrupt_polarity: bool,
    excitation_source: bool,
    cdc_bias: CdcBias,
}

impl PowerControl {
    pub(crate) const _REGISTER: u8 = 0x000;

    pub(crate) fn to_reg_value(&self) -> u16 {
        let mut res = 0u16;
        res |= self.mode as u16;
        res |= (self.lp_conversion_delay as u16) << 2;
        res |= ((self.num_stages - 1) as u16) << 4;
        res |= (self.decimation as u16) << 8;
        res |= (self.soft_reset as u16) << 10;
        res |= (self.interrupt_polarity as u16) << 11;
        res |= (self.excitation_source as u16) << 12;
        res |= (self.cdc_bias as u16) << 14;
        res
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PowerMode {
    #[default]
    Full = 0,
    Low = 2,
    Shutdown = 3,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LowPowerConversionDelay {
    #[default]
    Delay200ms = 0,
    Delay400ms = 1,
    Delay600ms = 2,
    Delay800ms = 3,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DecimationFactor {
    #[default]
    Factor256 = 0,
    Factor128 = 1,
    Factor64 = 2,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CdcBias {
    #[default]
    Normal = 0,
    Plus20Percent = 1,
    Plus35Percent = 2,
    Plus50Percent = 3,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CalibrationEnable {
    stages_enable: [bool; 12],
    avg_fp_skip: FullPowerSkip,
    avg_lp_skip: LowPowerSkip,
}

impl CalibrationEnable {
    pub(crate) const REGISTER: u16 = 0x001;

    pub(crate) fn to_reg_value(self) -> u16 {
        let mut res = 0u16;
        for (i, val) in self.stages_enable.iter().enumerate() {
            res |= (*val as u16) << i;
        }
        res |= (self.avg_fp_skip as u16) << 12;
        res |= (self.avg_lp_skip as u16) << 14;
        res
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum FullPowerSkip {
    #[default]
    Skip3 = 0,
    Skip7 = 1,
    Skip15 = 2,
    Skip31 = 3,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LowPowerSkip {
    #[default]
    SkipNone = 0,
    Skip1 = 1,
    Skip2 = 2,
    Skip3 = 3,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct AmbientCompensationControl {
    fast_filter_skip: u8,
    full_power_proximity_disable: u8,
    low_power_proximity_disable: u8,
    power_down_timeout: PowerDownTimeout,
    forced_calibration: bool,
    pub(crate) conversion_reset: bool,
    proximity_recalibration_level: u8,
    proximity_detection_rate: u8,
    slow_filter_update_level: u8,
    full_power_proximity_recalibration: u8,
    low_power_proximity_recalibration: u8,
}

impl AmbientCompensationControl {
    pub(crate) const REGISTER: u16 = 0x002;

    pub(crate) fn to_reg_value(self) -> [u16; 3] {
        let mut res = [0u16; 3];
        res[0] |= self.fast_filter_skip as u16;
        res[0] |= (self.full_power_proximity_disable as u16) << 4;
        res[0] |= (self.low_power_proximity_disable as u16) << 8;
        res[0] |= (self.power_down_timeout as u16) << 12;
        res[0] |= (self.forced_calibration as u16) << 14;
        res[0] |= (self.conversion_reset as u16) << 15;
        res[1] |= self.proximity_recalibration_level as u16;
        res[1] |= (self.proximity_detection_rate as u16) << 8;
        res[1] |= (self.slow_filter_update_level as u16) << 14;
        res[2] |= self.full_power_proximity_recalibration as u16;
        res[2] |= (self.low_power_proximity_recalibration as u16) << 10;
        res
    }
}

impl Default for AmbientCompensationControl {
    fn default() -> Self {
        Self {
            fast_filter_skip: Default::default(),
            full_power_proximity_disable: 0,
            low_power_proximity_disable: 0,
            power_down_timeout: PowerDownTimeout::Factor1_25,
            forced_calibration: false,
            conversion_reset: false,
            proximity_recalibration_level: 64,
            proximity_detection_rate: 1,
            slow_filter_update_level: 0,
            full_power_proximity_recalibration: 50,
            low_power_proximity_recalibration: 2,
        }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PowerDownTimeout {
    #[default]
    Factor1_25 = 0,
    Factor1_50 = 1,
    Factor1_75 = 2,
    Factor2 = 3,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct StageLowIntEnable {
    stages_enable: [bool; 12],
    gpio_setup: GpioSetup,
    gpio_input_config: GpioInputConfig,
}

impl StageLowIntEnable {
    pub(crate) const _REGISTER: u16 = 0x005;

    pub(crate) fn to_reg_value(self) -> u16 {
        let mut res = 0u16;
        for (i, val) in self.stages_enable.iter().enumerate() {
            res |= (*val as u16) << i;
        }
        res |= (self.gpio_setup as u16) << 12;
        res |= (self.gpio_input_config as u16) << 14;
        res
    }

    pub(crate) fn _is_any_enabled(&self) -> bool {
        self.stages_enable.iter().any(|e| *e)
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum GpioSetup {
    #[default]
    Disable = 0,
    Input = 1,
    OutputActiveLow = 2,
    OutputActiveHigh = 3,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum GpioInputConfig {
    #[default]
    NegativeLevelTrigger = 0,
    PositiveEdgeTrigger = 1,
    NegativeEdgeTrigger = 2,
    PositiveLevelTrigger = 3,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct StageHighIntEnable {
    stages_enable: [bool; 12],
}

impl StageHighIntEnable {
    pub(crate) const _REGISTER: u16 = 0x006;

    pub(crate) fn to_reg_value(self) -> u16 {
        let mut res = 0u16;
        for (i, val) in self.stages_enable.iter().enumerate() {
            res |= (*val as u16) << i;
        }
        res
    }

    pub(crate) fn _is_any_enabled(&self) -> bool {
        self.stages_enable.iter().any(|e| *e)
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct StageCompleteIntEnable {
    stages_enable: [bool; 12],
}

impl StageCompleteIntEnable {
    pub(crate) const _REGISTER: u16 = 0x007;

    pub(crate) fn to_reg_value(self) -> u16 {
        let mut res = 0u16;
        for (i, val) in self.stages_enable.iter().enumerate() {
            res |= (*val as u16) << i;
        }
        res
    }

    pub(crate) fn _is_any_enabled(&self) -> bool {
        self.stages_enable.iter().any(|e| *e)
    }
}
