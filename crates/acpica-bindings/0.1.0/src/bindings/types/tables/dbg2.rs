use crate::bindings::types::FfiAcpiTableHeader;

///  DBG2 - Debug Port Table 2
///         Version 0 (Both main table and subtables)
///
///  Conforms to \"Microsoft Debug Port Table 2 (DBG2)\", September 21, 2020
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableDbg2 {
    pub header: FfiAcpiTableHeader,
    pub info_offset: u32,
    pub info_count: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiDbg2Header {
    pub info_offset: u32,
    pub info_count: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiDbg2Device {
    pub revision: u8,
    pub length: u16,
    pub register_count: u8,
    pub namepath_length: u16,
    pub namepath_offset: u16,
    pub oem_data_length: u16,
    pub oem_data_offset: u16,
    pub port_type: u16,
    pub port_subtype: u16,
    pub reserved: u16,
    pub base_address_offset: u16,
    pub address_size_offset: u16,
}
