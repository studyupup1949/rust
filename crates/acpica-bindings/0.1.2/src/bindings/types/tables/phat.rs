use crate::bindings::types::FfiAcpiTableHeader;

///  PHAT - Platform Health Assessment Table (ACPI 6.4)
///         Version 1
///
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTablePhat {
    pub header: FfiAcpiTableHeader,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiPhatHeader {
    pub header_type: u16,
    pub length: u16,
    pub revision: u8,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiPhatVersionData {
    pub header: FfiAcpiPhatHeader,
    pub reserved: [u8; 3usize],
    pub element_count: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiPhatVersionElement {
    pub guid: [u8; 16usize],
    pub version_value: u64,
    pub producer_id: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiPhatHealthData {
    pub header: FfiAcpiPhatHeader,
    pub reserved: [u8; 2usize],
    pub health: u8,
    pub device_guid: [u8; 16usize],
    pub device_specific_offset: u32,
}
