#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StageConfiguration(pub(crate) InternalStageConfiguration);

impl StageConfiguration {
    pub fn builder() -> StageConfigurationBuilder {
        StageConfigurationBuilder {
            configuration: InternalStageConfiguration::default(),
            input_connections: heapless::Vec::new(),
        }
    }
}

pub struct StageConfigurationBuilder {
    configuration: InternalStageConfiguration,
    input_connections: heapless::Vec<InputConnection, 12>,
}

impl StageConfigurationBuilder {
    pub fn calibration_enabled(mut self, enabled: bool) -> StageConfigurationBuilder {
        self.configuration.global.calibration_enabled = enabled;
        self
    }

    pub fn low_interrupt_enabled(mut self, enabled: bool) -> StageConfigurationBuilder {
        self.configuration.global.low_int_enabled = enabled;
        self
    }

    pub fn high_interrupt_enabled(mut self, enabled: bool) -> StageConfigurationBuilder {
        self.configuration.global.high_int_enabled = enabled;
        self
    }

    pub fn conversion_complete_interrupt_enabled(
        mut self,
        enabled: bool,
    ) -> StageConfigurationBuilder {
        self.configuration.global.complete_int_enabled = enabled;
        self
    }

    pub fn add_input_connection(
        mut self,
        input_connection: InputConnection,
    ) -> StageConfigurationBuilder {
        let _ = self.input_connections.push(input_connection);
        self
    }

    pub fn sensitivity(mut self, sensitivity: StageSensitivity) -> StageConfigurationBuilder {
        self.configuration.sensitivity = sensitivity;
        self
    }

    pub fn initial_offset_low(mut self, offset: u16) -> StageConfigurationBuilder {
        self.configuration.initial_offset_low = offset;
        self
    }

    pub fn initial_offset_high(mut self, offset: u16) -> StageConfigurationBuilder {
        self.configuration.initial_offset_high = offset;
        self
    }

    pub fn offset_high_clamp(mut self, clamp: u16) -> StageConfigurationBuilder {
        self.configuration.offset_high_clamp = clamp;
        self
    }

    pub fn offset_low_clamp(mut self, clamp: u16) -> StageConfigurationBuilder {
        self.configuration.offset_low_clamp = clamp;
        self
    }

    pub fn pos_afe_offset(mut self, offset: u8) -> StageConfigurationBuilder {
        self.configuration.afe_offset_configuration.pos_afe_offset = offset;
        self
    }

    pub fn neg_afe_offset(mut self, offset: u8) -> StageConfigurationBuilder {
        self.configuration.afe_offset_configuration.neg_afe_offset = offset;
        self
    }

    pub fn build(mut self) -> StageConfiguration {
        if self.input_connections.len() > 1 {
            self.configuration.input_configuration =
                InputMode::Differential([self.input_connections[0], self.input_connections[1]]);
        } else {
            self.configuration.input_configuration =
                InputMode::SingleEnded(self.input_connections[0]);
        }
        StageConfiguration(self.configuration)
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct InternalStageConfiguration {
    input_configuration: InputMode,
    neg_afe_offset_disable: bool,
    pos_afe_offset_disable: bool,
    afe_offset_configuration: AfeOffsetConfiguration,
    sensitivity: StageSensitivity,
    initial_offset_low: u16,
    initial_offset_high: u16,
    offset_high_clamp: u16,
    offset_low_clamp: u16,
    pub(crate) global: StageGlobalConfig,
}

impl InternalStageConfiguration {
    pub(crate) fn to_reg_value(self) -> [u16; 8] {
        let mut res = [0u16; 8];
        res[0..2].copy_from_slice(&self.input_configuration.to_reg_value());
        res[1] |= (self.neg_afe_offset_disable as u16) << 14;
        res[1] |= (self.pos_afe_offset_disable as u16) << 15;
        res[2] = self.afe_offset_configuration.to_reg_value();
        res[3] = self.sensitivity.to_reg_value();
        res[4] = self.initial_offset_low;
        res[5] = self.initial_offset_high;
        res[6] = self.offset_high_clamp;
        res[7] = self.offset_low_clamp;
        res
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct StageGlobalConfig {
    pub(crate) calibration_enabled: bool,
    pub(crate) low_int_enabled: bool,
    pub(crate) high_int_enabled: bool,
    pub(crate) complete_int_enabled: bool,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum InputMode {
    #[default]
    NotConfigured,
    SingleEnded(InputConnection),
    Differential([InputConnection; 2]),
}

impl InputMode {
    pub(crate) fn to_reg_value(self) -> [u16; 2] {
        let mut res = 0u32;
        match self {
            Self::NotConfigured => {
                panic!("Input mode cannot be left unconfigured")
            }
            InputMode::SingleEnded(conn) => {
                res |= 0xFFFFFFFF >> 4;
                res &= ((conn.cdc as u32) << (conn.cin as u32)) | !(0b11 << (conn.cin as u32));
                res |= if conn.cdc == CdcInput::Positive {
                    0b01
                } else {
                    0b10
                } << 28;
            }
            InputMode::Differential(conns) => {
                res |= 0xFFFFFFFF >> 4;
                for conn in conns {
                    res &= ((conn.cdc as u32) << (conn.cin as u32)) | !(0b11 << (conn.cin as u32));
                }
                res |= 0b11 << 28;
            }
        }
        [res as u16, (res >> 16) as u16]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InputConnection {
    pub cin: CapInput,
    pub cdc: CdcInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CapInput {
    CIN0 = 0,
    CIN1 = 2,
    CIN2 = 4,
    CIN3 = 6,
    CIN4 = 8,
    CIN5 = 10,
    CIN6 = 12,
    CIN7 = 16,
    CIN8 = 18,
    CIN9 = 20,
    CIN10 = 22,
    CIN11 = 24,
    CIN12 = 26,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CdcInput {
    Positive = 0b10,
    Negative = 0b01,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct AfeOffsetConfiguration {
    neg_afe_offset: u8,
    neg_afe_offset_swap: bool,
    pos_afe_offset: u8,
    pos_afe_offset_swap: bool,
}

impl AfeOffsetConfiguration {
    pub(crate) fn to_reg_value(self) -> u16 {
        let mut res = 0u16;
        res |= (self.neg_afe_offset & 0b00011111) as u16;
        res |= (self.neg_afe_offset_swap as u16) << 7;
        res |= ((self.pos_afe_offset & 0b00011111) as u16) << 8;
        res |= (self.pos_afe_offset_swap as u16) << 15;
        res
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct StageSensitivity {
    pub neg_threshold_sensitivity: ThresholdSensitivity,
    pub neg_peak_detect: PeakDetect,
    pub pos_threshold_sensitivity: ThresholdSensitivity,
    pub pos_peak_detect: PeakDetect,
}

impl StageSensitivity {
    pub(crate) fn to_reg_value(self) -> u16 {
        let mut res = 0u16;
        res |= self.neg_threshold_sensitivity as u16;
        res |= (self.neg_peak_detect as u16) << 4;
        res |= (self.pos_threshold_sensitivity as u16) << 8;
        res |= (self.pos_peak_detect as u16) << 12;
        res
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThresholdSensitivity {
    #[default]
    Percent25 = 0,
    Percent29 = 1,
    Percent34 = 2,
    Percent39 = 3,
    Percent43 = 4,
    Percent48 = 5,
    Percent53 = 6,
    Percent58 = 7,
    Percent62 = 8,
    Percent67 = 9,
    Percent71 = 10,
    Percent76 = 11,
    Percent81 = 12,
    Percent85 = 13,
    Percent90 = 14,
    Percent95 = 15,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PeakDetect {
    #[default]
    Percent40 = 0,
    Percent50 = 1,
    Percent60 = 2,
    Percent70 = 3,
    Percent80 = 4,
    Percent90 = 5,
}
