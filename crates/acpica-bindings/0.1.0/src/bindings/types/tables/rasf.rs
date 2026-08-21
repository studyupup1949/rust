use crate::bindings::types::FfiAcpiTableHeader;

///  RASF - RAS Feature Table (ACPI 5.0)
///         Version 1
///
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableRasf {
    pub header: FfiAcpiTableHeader,
    pub channel_id: [u8; 12usize],
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiRasfSharedMemory {
    pub signature: u32,
    pub command: u16,
    pub status: u16,
    pub version: u16,
    pub capabilities: [u8; 16usize],
    pub set_capabilities: [u8; 16usize],
    pub num_parameter_blocks: u16,
    pub set_capabilities_status: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiRasfParameterBlock {
    pub block_type: u16,
    pub version: u16,
    pub length: u16,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiRasfPatrolScrubParameter {
    pub header: FfiAcpiRasfParameterBlock,
    pub patrol_scrub_command: u16,
    pub requested_address_range: [u64; 2usize],
    pub actual_address_range: [u64; 2usize],
    pub flags: u16,
    pub requested_speed: u8,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiRasfCommands {
    ExecuteRasfCommand = 1,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiRasfCapabiliities {
    HwPatrolScrubSupported = 0,
    SwPatrolScrubExposed = 1,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiRasfPatrolScrubCommands {
    GetPatrolParameters = 1,
    StartPatrolScrubber = 2,
    StopPatrolScrubber = 3,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiRasfStatus {
    Success = 0,
    NotValid = 1,
    NotSupported = 2,
    Busy = 3,
    Failed = 4,
    Aborted = 5,
    InvalidData = 6,
}
