use crate::bindings::types::FfiAcpiTableHeader;

///  PRMT - Platform Runtime Mechanism Table
///         Version 1
///
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTablePrmt {
    pub header: FfiAcpiTableHeader,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTablePrmtHeader {
    pub platform_guid: [u8; 16usize],
    pub module_info_offset: u32,
    pub module_info_count: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiPrmtModuleHeader {
    pub revision: u16,
    pub length: u16,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiPrmtModuleInfo {
    pub revision: u16,
    pub length: u16,
    pub module_guid: [u8; 16usize],
    pub major_rev: u16,
    pub minor_rev: u16,
    pub handler_info_count: u16,
    pub handler_info_offset: u32,
    pub mmio_list_pointer: u64,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiPrmtHandlerInfo {
    pub revision: u16,
    pub length: u16,
    pub handler_guid: [u8; 16usize],
    pub handler_address: u64,
    pub static_data_buffer_address: u64,
    pub acpi_param_buffer_address: u64,
}
