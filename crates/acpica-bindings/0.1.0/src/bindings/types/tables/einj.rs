use crate::{bindings::types::FfiAcpiTableHeader, bindings::types::FfiAcpiWheaHeader};

///  EINJ - Error Injection Table (ACPI 4.0)
///         Version 1
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableEinj {
    pub header: FfiAcpiTableHeader,
    pub header_length: u32,
    pub flags: u8,
    pub reserved: [u8; 3usize],
    pub entries: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiEinjEntry {
    whea_header: FfiAcpiWheaHeader,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiEinjActions {
    BeginOperation = 0,
    GetTriggerTable = 1,
    SetErrorType = 2,
    GetErrorType = 3,
    EndOperation = 4,
    ExecuteOperation = 5,
    CheckBusyStatus = 6,
    GetCommandStatus = 7,
    SetErrorTypeWithAddress = 8,
    GetExecuteTimings = 9,
    ActionReserved = 10,
    TriggerError = 255,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiEinjInstructions {
    ReadRegister = 0,
    ReadRegisterValue = 1,
    WriteRegister = 2,
    WriteRegisterValue = 3,
    Noop = 4,
    FlushCacheline = 5,
    InstructionReserved = 6,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiEinjErrorTypeWithAddr {
    pub error_type: u32,
    pub vendor_struct_offset: u32,
    pub flags: u32,
    pub apic_id: u32,
    pub address: u64,
    pub range: u64,
    pub pcie_id: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiEinjVendor {
    pub length: u32,
    pub pcie_id: u32,
    pub vendor_id: u16,
    pub device_id: u16,
    pub revision_id: u8,
    pub reserved: [u8; 3usize],
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiEinjTrigger {
    pub header_size: u32,
    pub revision: u32,
    pub table_size: u32,
    pub entry_count: u32,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiEinjCommandStatus {
    Success = 0,
    Failure = 1,
    InvalidAccess = 2,
    StatusReserved = 3,
}
