use crate::bindings::types::FfiAcpiTableHeader;

///  RSDT/XSDT - Root System Description Tables
///              Version 1 (both)
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableRsdt {
    pub header: FfiAcpiTableHeader,
    pub table_offset_entry: [u32; 1usize],
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableXsdt {
    pub header: FfiAcpiTableHeader,
    pub table_offset_entry: [u64; 1usize],
}
