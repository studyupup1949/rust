use crate::bindings::types::FfiAcpiTableHeader;

use super::FfiAcpiSubtableHeader;

///  PPTT - Processor Properties Topology Table (ACPI 6.2)
///         Version 1
///
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTablePptt {
    pub header: FfiAcpiTableHeader,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiPpttType {
    Processor = 0,
    Cache = 1,
    Id = 2,
    Reserved = 3,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiPpttProcessor {
    pub header: FfiAcpiSubtableHeader,
    pub reserved: u16,
    pub flags: u32,
    pub parent: u32,
    pub acpi_processor_id: u32,
    pub number_of_priv_resources: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiPpttCache {
    pub header: FfiAcpiSubtableHeader,
    pub reserved: u16,
    pub flags: u32,
    pub next_level_of_cache: u32,
    pub size: u32,
    pub number_of_sets: u32,
    pub associativity: u8,
    pub attributes: u8,
    pub line_size: u16,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiPpttCacheV1 {
    pub cache_id: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiPpttId {
    pub header: FfiAcpiSubtableHeader,
    pub reserved: u16,
    pub vendor_id: u32,
    pub level1_id: u64,
    pub level2_id: u64,
    pub major_rev: u16,
    pub minor_rev: u16,
    pub spin_rev: u16,
}
