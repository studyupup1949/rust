use crate::bindings::types::FfiAcpiGenericAddress;

/// A type of address space that a [generic address structure][AcpiGenericAddress] can point into
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GasAddressSpace {
    /// A physical address in main memory
    SystemMemory,
    /// An IO port number
    SystemIO,
    /// A register in the PCI configuration space. PCI addresses in a GAS are confined to devices on PCI Segment Group 0, bus 0.
    ///
    /// Using this address space, [`address`] is interpreted as follows:
    ///
    /// * Most significant word: Ignored, must be 0
    /// * Next word: Device number on bus 0
    /// * Next word: Function number
    /// * Least significant word: Offset into the configuration registers (TODO: is this in bytes or registers?)
    ///
    /// For example: Offset 0x23 of Function 2 on device 7 on bus 0 segment 0 would be represented as 0x0000 0007 0002 0023.
    ///
    /// [`address`]: AcpiGenericAddress::address
    PciConfigurationSpace,
    /// An embedded controller
    EmbeddedController,
    /// The system management bus
    SMBus,
    /// The system's CMOS chip
    SystemCMOS,
    /// A register inside an MMIO region, located by a BAR located in the PCI configuration space.
    ///
    /// Using this address space, [`address`] is interpreted as follows:
    ///
    /// Bits 63:56 – PCI Segment\
    /// Bits 55:48 – PCI Bus\
    /// Bits 47:43 – PCI Device\
    /// Bits 42:40 – PCI Function\
    /// Bits 39:37 – BAR Number\
    /// Bits 36:0 – Offset from BAR in DWORDs\
    ///
    /// [`address`]: AcpiGenericAddress::address
    PciBarTarget,
    /// The IPMI bus
    IPMI,
    /// An address relating to GPIO
    GeneralPurposeIO,
    /// A serial bus
    GenericSerialBus,
    /// The platform communications channel
    PlatformCommunicationsChannel,
    /// The platform runtime mechanism
    PlatformRuntimeMechanism,
    /// Functional fixed hardware
    FunctionalFixedHardware,

    /// An unknown or reserved value
    Other(u8),
}

impl GasAddressSpace {
    /// Gets the [`GasAddressSpace`] from its [`u8`] representation
    fn from_u8(ffi_address_space: u8) -> Self {
        match ffi_address_space {
            0x00 => Self::SystemMemory,
            0x01 => Self::SystemIO,
            0x02 => Self::PciConfigurationSpace,
            0x03 => Self::EmbeddedController,
            0x04 => Self::SMBus,
            0x05 => Self::SystemCMOS,
            0x06 => Self::PciBarTarget,
            0x07 => Self::IPMI,
            0x08 => Self::GeneralPurposeIO,
            0x09 => Self::GenericSerialBus,
            0x0A => Self::PlatformCommunicationsChannel,
            0x0B => Self::PlatformRuntimeMechanism,
            0x0C => Self::FunctionalFixedHardware,
            s => Self::Other(s),
        }
    }

    /// Gets the [`GasAddressSpace`] from its [`u8`] representation
    fn to_u8(self) -> u8 {
        match self {
            Self::SystemMemory => 0x00,
            Self::SystemIO => 0x01,
            Self::PciConfigurationSpace => 0x02,
            Self::EmbeddedController => 0x03,
            Self::SMBus => 0x04,
            Self::SystemCMOS => 0x05,
            Self::PciBarTarget => 0x06,
            Self::IPMI => 0x07,
            Self::GeneralPurposeIO => 0x08,
            Self::GenericSerialBus => 0x09,
            Self::PlatformCommunicationsChannel => 0x0A,
            Self::PlatformRuntimeMechanism => 0x0B,
            Self::FunctionalFixedHardware => 0x0C,
            Self::Other(s) => s,
        }
    }
}

/// GAS - Generic Address Structure
///
/// This struct represents an address in some address space - that could be main memory, port I/O, PCI configuration space, etc.
#[derive(Debug, Copy, Clone)]
pub struct AcpiGenericAddress {
    /// What address space to access for this value
    pub space_id: GasAddressSpace,
    /// The size in bits of the data pointed to by the address
    pub bit_width: u8,
    /// The offset in bits of the data from the address
    pub bit_offset: u8,
    /// A value representing the access size needed to read data.
    ///
    /// The value in this register does not match to number of bytes, rather there is a mapping as follows:
    /// * 1 = Byte access
    /// * 2 = Word access (2 bytes)
    /// * 3 = DWord access (4 bytes)
    /// * 4 = QWord access (8 bytes)
    ///
    /// The `AcpiGenericAddress::ACCESS_WIDTH_*` consts can be used to reference these values.
    ///
    /// This field is not represented by an enum because some values of [`space_id`] could use a different system.
    ///
    /// [`space_id`]: AcpiGenericAddress::space_id
    pub access_width: u8,
    /// The address where the data is stored
    pub address: u64,
}

impl AcpiGenericAddress {
    /// The value of [`access_width`][AcpiGenericAddress::access_width] corresponding to a byte access
    pub const ACCESS_WIDTH_1_BYTE: u8 = 1;
    /// The value of [`access_width`][AcpiGenericAddress::access_width] corresponding to a word access (2 bytes)
    pub const ACCESS_WIDTH_2_BYTES: u8 = 2;
    /// The value of [`access_width`][AcpiGenericAddress::access_width] corresponding to a double word access (4 bytes)
    pub const ACCESS_WIDTH_4_BYTES: u8 = 3;
    /// The value of [`access_width`][AcpiGenericAddress::access_width] corresponding to a quad word access (8 bytes)
    pub const ACCESS_WIDTH_8_BYTES: u8 = 4;

    pub(crate) fn from_ffi(value: FfiAcpiGenericAddress) -> Self {
        Self {
            space_id: GasAddressSpace::from_u8(value.space_id),
            bit_width: value.bit_width,
            bit_offset: value.bit_offset,
            access_width: value.access_width,
            address: value.address,
        }
    }
}
