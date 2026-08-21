use crate::{bindings::types::FfiAcpiTableHeader, bindings::types::FfiAcpiWheaHeader};

///  ERST - Error Record Serialization Table (ACPI 4.0)
///         Version 1
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableErst {
    pub header: FfiAcpiTableHeader,
    pub header_length: u32,
    pub reserved: u32,
    pub entries: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiErstEntry {
    whea_header: FfiAcpiWheaHeader,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiErstActions {
    BeginWrite = 0,
    BeginRead = 1,
    BeginClear = 2,
    End = 3,
    SetRecordOffset = 4,
    ExecuteOperation = 5,
    CheckBusyStatus = 6,
    GetCommandStatus = 7,
    GetRecordId = 8,
    SetRecordId = 9,
    GetRecordCount = 10,
    BeginDummyWriite = 11,
    NotUsed = 12,
    GetErrorRange = 13,
    GetErrorLength = 14,
    GetErrorAttributes = 15,
    ExecuteTimings = 16,
    ActionReserved = 17,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiErstInstructions {
    ReadRegister = 0,
    ReadRegisterValue = 1,
    WriteRegister = 2,
    WriteRegisterValue = 3,
    Noop = 4,
    LoadVar1 = 5,
    LoadVar2 = 6,
    StoreVar1 = 7,
    Add = 8,
    Subtract = 9,
    AddValue = 10,
    SubtractValue = 11,
    Stall = 12,
    StallWhileTrue = 13,
    SkipNextIfTrue = 14,
    Goto = 15,
    SetSrcAddressBase = 16,
    SetDstAddressBase = 17,
    MoveData = 18,
    InstructionReserved = 19,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiErstCommandStatus {
    Success = 0,
    NoSpace = 1,
    NotAvailable = 2,
    Failure = 3,
    RecordEmpty = 4,
    NotFound = 5,
    StatusReserved = 6,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub(crate) struct FfiAcpiErstInfo {
    pub signature: u16,
    pub data: [u8; 48usize],
}
