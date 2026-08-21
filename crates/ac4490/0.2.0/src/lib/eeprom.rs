use defmt::Format;

#[repr(u8)]
#[derive(Debug, Format, Clone, Copy)]
pub enum Parameter {
    ProductIdString = 0x00,
    PageRefresh = 0x3D,
    StopBitDelay = 0x3F,
    ChannelNumber = 0x40,
    ServerClientMode = 0x41,
    // BaudRateLow = 0x42,
    // BaudRateHigh = 0x43,
    Control0 = 0x45,
    FrequencyOffset = 0x46,
    CmdDataRxDisable = 0x4B,
    TransmitRetries = 0x4C,
    BroadcastAttempts = 0x4D,
    ApiControl = 0x56,
    InterfaceTimeout = 0x58,
    SyncChannel = 0x5A,
    RfPacketSize = 0x5B,
    CtsOn = 0x5C,
    CtsOnHysterisis = 0x5D,
    MaxPower = 0x63,
    ModemMode = 0x6E,
    Parity = 0x6F,
    DestinationId = 0x70,
    SystemId = 0x76,
    MacId = 0x80,
    OriginalMaxPower = 0x8E,
    ProductId = 0x90,
    ProtocolStatusReceiveAck = 0xC0,
    ReceiveApi = 0xC1,
    EnhancedApiControl = 0xC6,
    AutoCalibrate = 0xCC,
    DesKey = 0xD0,

}

impl Parameter {

    #[must_use]
    pub const fn length(self) -> u8 {
        match self {
            Self::ProductIdString => 40,
            Self::DestinationId | Self::MacId => 6,
            Self::ProductId => 15,
            Self::DesKey => 7,
            _ => 1
        }
    }

}