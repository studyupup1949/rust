use crate::bindings::types::FfiAcpiTableHeader;

///  DMAR - DMA Remapping table
///         Version 1
///
///  Conforms to \"Intel Virtualization Technology for Directed I/O\",
///  Version 2.3, October 2014
///
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableDmar {
    pub header: FfiAcpiTableHeader,
    pub width: u8,
    pub flags: u8,
    pub reserved: [u8; 10usize],
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiDmarHeader {
    pub header_type: u16,
    pub length: u16,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiDmarType {
    HardwareUnit = 0,
    ReservedMemory = 1,
    RootAts = 2,
    HardwareAffinity = 3,
    Namespace = 4,
    Reserved = 5,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiDmarDeviceScope {
    pub entry_type: u8,
    pub length: u8,
    pub reserved: u16,
    pub enumeration_id: u8,
    pub bus: u8,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiDmarScopeType {
    NotUsed = 0,
    Endpoint = 1,
    Bridge = 2,
    Ioapic = 3,
    Hpet = 4,
    Namespace = 5,
    Reserved = 6,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiDmarPciPath {
    pub device: u8,
    pub function: u8,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiDmarHardwareUnit {
    pub header: FfiAcpiDmarHeader,
    pub flags: u8,
    pub reserved: u8,
    pub segment: u16,
    pub address: u64,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiDmarReservedMemory {
    pub header: FfiAcpiDmarHeader,
    pub reserved: u16,
    pub segment: u16,
    pub base_address: u64,
    pub end_address: u64,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiDmarAtsr {
    pub header: FfiAcpiDmarHeader,
    pub flags: u8,
    pub reserved: u8,
    pub segment: u16,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiDmarRhsa {
    pub header: FfiAcpiDmarHeader,
    pub reserved: u32,
    pub base_address: u64,
    pub proximity_domain: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiDmarAndd {
    pub header: FfiAcpiDmarHeader,
    pub reserved: [u8; 3usize],
    pub device_number: u8,
    pub device_name: [i8; 1usize],
}
