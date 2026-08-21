use crate::bindings::types::{FfiAcpiGenericAddress, FfiAcpiTableHeader};

///  WDAT - Watchdog Action Table
///         Version 1
///
///  Conforms to \"Hardware Watchdog Timers Design Specification\",
///  Copyright 2006 Microsoft Corporation.
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableWdat {
    pub header: FfiAcpiTableHeader,
    pub header_length: u32,
    pub pci_segment: u16,
    pub pci_bus: u8,
    pub pci_device: u8,
    pub pci_function: u8,
    pub reserved: [u8; 3usize],
    pub timer_period: u32,
    pub max_count: u32,
    pub min_count: u32,
    pub flags: u8,
    pub reserved2: [u8; 3usize],
    pub entries: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiWdatEntry {
    pub action: u8,
    pub instruction: u8,
    pub reserved: u16,
    pub register_region: FfiAcpiGenericAddress,
    pub value: u32,
    pub mask: u32,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiWdatActions {
    Reset = 1,
    GetCurrentCountdown = 4,
    GetCountdown = 5,
    SetCountdown = 6,
    GetRunningState = 8,
    SetRunningState = 9,
    GetStoppedState = 10,
    SetStoppedState = 11,
    GetReboot = 16,
    SetReboot = 17,
    GetShutdown = 18,
    SetShutdown = 19,
    GetStatus = 32,
    SetStatus = 33,
    ActionReserved = 34,
}
#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiWdatInstructions {
    ReadValue = 0,
    ReadCountdown = 1,
    WriteValue = 2,
    WriteCountdown = 3,
    InstructionReserved = 4,
    PreserveRegister = 128,
}

///  WDDT - Watchdog Descriptor Table
///         Version 1
///
///  Conforms to \"Using the Intel ICH Family Watchdog Timer (WDT)\",
///  Version 001, September 2002
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableWddt {
    pub header: FfiAcpiTableHeader,
    pub spec_version: u16,
    pub table_version: u16,
    pub pci_vendor_id: u16,
    pub address: FfiAcpiGenericAddress,
    pub max_count: u16,
    pub min_count: u16,
    pub period: u16,
    pub status: u16,
    pub capability: u16,
}

///  WDRT - Watchdog Resource Table
///         Version 1
///
///  Conforms to \"Watchdog Timer Hardware Requirements for Windows Server 2003\",
///  Version 1.01, August 28, 2006
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableWdrt {
    pub header: FfiAcpiTableHeader,
    pub control_register: FfiAcpiGenericAddress,
    pub count_register: FfiAcpiGenericAddress,
    pub pci_device_id: u16,
    pub pci_vendor_id: u16,
    pub pci_bus: u8,
    pub pci_device: u8,
    pub pci_function: u8,
    pub pci_segment: u8,
    pub max_count: u16,
    pub units: u8,
}
