use defmt::Format;
use num_enum::TryFromPrimitive;

use crate::Error;

/// RF channel (00-37)
/// 
/// Set 0 = 0x00 - 0x0F (US/Canada) (1x1/200)
/// Set 1 = 0x10 - 0x2F (US/Canada) (1x1/1000)
/// Set 2 = 0x30 - 0x37 Australia (1x1/200/1000)
///
#[repr(u8)]
#[derive(Format, Debug, Clone, Copy, TryFromPrimitive)]
pub enum Channel {
    Ch00 = 0x00,
    Ch01 = 0x01,
    Ch02 = 0x02,
    Ch03 = 0x03,
    Ch04 = 0x04,
    Ch05 = 0x05,
    Ch06 = 0x06,
    Ch07 = 0x07,
    Ch08 = 0x08,
    Ch09 = 0x09,
    Ch0A = 0x0A,
    Ch0B = 0x0B,
    Ch0C = 0x0C,
    Ch0D = 0x0D,
    Ch0E = 0x0E,
    Ch0F = 0x0F,
    Ch10 = 0x10,
    Ch11 = 0x11,
    Ch12 = 0x12,
    Ch13 = 0x13,
    Ch14 = 0x14,
    Ch15 = 0x15,
    Ch16 = 0x16,
    Ch17 = 0x17,
    Ch18 = 0x18,
    Ch19 = 0x19,
    Ch1A = 0x1A,
    Ch1B = 0x1B,
    Ch1C = 0x1C,
    Ch1D = 0x1D,
    Ch1E = 0x1E,
    Ch1F = 0x1F,
    Ch20 = 0x20,
    Ch21 = 0x21,
    Ch22 = 0x22,
    Ch23 = 0x23,
    Ch24 = 0x24,
    Ch25 = 0x25,
    Ch26 = 0x26,
    Ch27 = 0x27,
    Ch28 = 0x28,
    Ch29 = 0x29,
    Ch2A = 0x2A,
    Ch2B = 0x2B,
    Ch2C = 0x2C,
    Ch2D = 0x2D,
    Ch2E = 0x2E,
    Ch2F = 0x2F,
    Ch30 = 0x30,
    Ch31 = 0x31,
    Ch32 = 0x32,
    Ch33 = 0x33,
    Ch34 = 0x34,
    Ch35 = 0x35,
    Ch36 = 0x36,
    Ch37 = 0x37,
}

/// Server / Client mode selection
#[derive(Format, Debug, Clone, Copy)]
pub enum ServerClientMode {
    Server,
    Client,
}

impl ServerClientMode {

    pub(crate) const fn u8_for_cmd(self) -> u8 {
        match self {
            Self::Server => 0x00,
            Self::Client => 0x03,
        }
    }

    pub(crate) const fn u8_for_eeprom(self) -> u8 {
        match self {
            Self::Server => 0x01,
            Self::Client => 0x02,
        }
    }

    #[allow(dead_code)]
    pub(crate) const fn from_cmd(value: u8) -> Result<Self, Error> {
        match value {
            0x00 => Ok(Self::Server),
            0x03 => Ok(Self::Client),
            _ => Err(Error::InvalidResponseServerClient),
        }
    }

    pub(crate) const fn from_eeprom(value: u8) -> Result<Self, Error> {
        match value {
            0x01 => Ok(Self::Server),
            0x02 => Ok(Self::Client),
            _ => Err(Error::InvalidResponseServerClient),
        }
    }

}

/// Device status response (mode and in/out of range)
#[repr(u8)]
#[derive(Format, Debug, Clone, Copy, TryFromPrimitive)]
#[allow(clippy::enum_variant_names)]
pub enum Status {
    ServerInRange = 0x00,
    ClientInRange = 0x01,
    ServerOutOfRange = 0x02,
    ClientOutOfRange = 0x03,
}

/// ADC port selection
/// 
/// `AdIn` = External ADC pin
/// `Temp` = Temperature sensor
/// `Rssi` = Received signal strength indicator
/// 
#[repr(u8)]
#[derive(Format, Debug, Clone, Copy, TryFromPrimitive)]
pub enum AdcPort {
    AdIn = 0x00,
    Temp = 0x01,
    Rssi = 0x03,
}

/// Output power selection
/// 
/// In units of dBm (e.g. `Minus1dBm` = -1dBm)
/// 
#[repr(u8)]
#[derive(Format, Debug, Clone, Copy, TryFromPrimitive)]
#[allow(clippy::enum_variant_names)]
pub enum OutputPower {
    Minus1dBm = 0x00,
    Plus9dBm = 0x01,
    Plus14dBm = 0x02,
    Plus16_5dBm = 0x03,
    Plus19dBm = 0x04,
    Plus20_5dBm = 0x05,
    Plus22dBm = 0x06,
    Plus23dBm = 0x07,
    Plus24_5dBm = 0x08,
    Plus25dBm = 0x09,
    Plus26dBm = 0x0A,
    Plus26_5dBm = 0x0B,
    Plus27dBm = 0x0C,
    Plus27_5dBm = 0x0D,
    Plus28dBm = 0x0E,
    Plus28_7dBm = 0x0F,
}

/// EEPROM Control0 Flags
/// 
#[derive(Format, Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct Control0 {
    pub one_beacon_mode: bool,
    pub des_encryption: bool,
    pub sync_to_channel: bool,
    pub broadcast_packets: bool,
}

impl From<u8> for Control0 {
    fn from(value: u8) -> Self {
        Self {
            one_beacon_mode: value & 0b1000_0000 != 0,
            des_encryption: value & 0b0100_0000 != 0,
            sync_to_channel: value & 0b0010_0000 != 0,
            broadcast_packets: value & 0b0000_0010 != 0,
        }
    }
}

impl From<Control0> for u8 {
    fn from(value: Control0) -> Self {
        let mut result = 0;
        if value.one_beacon_mode {
            result |= 0b1000_0000;
        }
        if value.des_encryption {
            result |= 0b0100_0000;
        }
        if value.sync_to_channel {
            result |= 0b0010_0000;
        }
        if value.broadcast_packets {
            result |= 0b0000_0010;
        }
        result
    }
}

/// EEPROM API Control Flags
#[derive(Format, Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct ApiControl {
    pub unicast_only: bool,
    pub auto_destination: bool,
    pub client_auto_channel: bool,
    pub rts_enable: bool,
    pub duplex: bool,
    pub auto_config: bool,
}

impl From<u8> for ApiControl {
    fn from(value: u8) -> Self {
        Self {
            unicast_only: value & 0b0010_0000 != 0,
            auto_destination: value & 0b0001_0000 != 0,
            client_auto_channel: value & 0b0000_1000 != 0,
            rts_enable: value & 0b0000_0100 != 0,
            duplex: value & 0b0000_0010 != 0,
            auto_config: value & 0b0000_0001 != 0,
        }
    }
}

impl From<ApiControl> for u8 {
    fn from(value: ApiControl) -> Self {
        let mut result = 0;
        if value.unicast_only {
            result |= 0b0010_0000;
        }
        if value.auto_destination {
            result |= 0b0001_0000;
        }
        if value.client_auto_channel {
            result |= 0b0000_1000;
        }
        if value.rts_enable {
            result |= 0b0000_0100;
        }
        if value.duplex {
            result |= 0b0000_0010;
        }
        if value.auto_config {
            result |= 0b0000_0001;
        }
        result
    }
}

/// EEPROM Enhanced API Control Flags
#[derive(Format, Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct EnhancedApiControl {
    #[allow(clippy::struct_field_names)]
    pub enable_enhanced_api_control: bool,
    pub send_data_complete_enable: bool,
    pub api_transmit_packet_enable: bool,
    pub enhanced_api_receive_packet_enable: bool,
}

impl From<u8> for EnhancedApiControl {
    fn from(value: u8) -> Self {
        Self {
            enable_enhanced_api_control: (!value) & 0b1000_0000 != 0,
            send_data_complete_enable: value & 0b0000_0100 != 0,
            api_transmit_packet_enable: value & 0b0000_0010 != 0,
            enhanced_api_receive_packet_enable: value & 0b0000_0001 != 0,
        }
    }
}

impl From<EnhancedApiControl> for u8 {
    fn from(value: EnhancedApiControl) -> Self {
        let mut result = 0;
        if !value.enable_enhanced_api_control {
            result |= 0b1000_0000;
        }
        if value.send_data_complete_enable {
            result |= 0b0000_0100;
        }
        if value.api_transmit_packet_enable {
            result |= 0b0000_0010;
        }
        if value.enhanced_api_receive_packet_enable {
            result |= 0b0000_0001;
        }
        result
    }
}