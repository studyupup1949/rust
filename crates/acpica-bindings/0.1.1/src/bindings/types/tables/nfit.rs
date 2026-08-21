use crate::bindings::types::FfiAcpiTableHeader;

///  NFIT - NVDIMM Interface Table (ACPI 6.0+)
///         Version 1
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableNfit {
    pub header: FfiAcpiTableHeader,
    pub reserved: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiNfitHeader {
    pub header_type: u16,
    pub length: u16,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiNfitType {
    SystemAddress = 0,
    MemoryMap = 1,
    Interleave = 2,
    Smbios = 3,
    ControlRegion = 4,
    DataRegion = 5,
    FlushAddress = 6,
    Capabilities = 7,
    Reserved = 8,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiNfitSystemAddress {
    pub header: FfiAcpiNfitHeader,
    pub range_index: u16,
    pub flags: u16,
    pub reserved: u32,
    pub proximity_domain: u32,
    pub range_guid: [u8; 16usize],
    pub address: u64,
    pub length: u64,
    pub memory_mapping: u64,
    pub location_cookie: u64,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiNfitMemoryMap {
    pub header: FfiAcpiNfitHeader,
    pub device_handle: u32,
    pub physical_id: u16,
    pub region_id: u16,
    pub range_index: u16,
    pub region_index: u16,
    pub region_size: u64,
    pub region_offset: u64,
    pub address: u64,
    pub interleave_index: u16,
    pub interleave_ways: u16,
    pub flags: u16,
    pub reserved: u16,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiNfitInterleave {
    pub header: FfiAcpiNfitHeader,
    pub interleave_index: u16,
    pub reserved: u16,
    pub line_count: u32,
    pub line_size: u32,
    pub line_offset: [u32; 1usize],
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiNfitSmbios {
    pub header: FfiAcpiNfitHeader,
    pub reserved: u32,
    pub data: [u8; 1usize],
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiNfitControlRegion {
    pub header: FfiAcpiNfitHeader,
    pub region_index: u16,
    pub vendor_id: u16,
    pub device_id: u16,
    pub revision_id: u16,
    pub subsystem_vendor_id: u16,
    pub subsystem_device_id: u16,
    pub subsystem_revision_id: u16,
    pub valid_fields: u8,
    pub manufacturing_location: u8,
    pub manufacturing_date: u16,
    pub reserved: [u8; 2usize],
    pub serial_number: u32,
    pub code: u16,
    pub windows: u16,
    pub window_size: u64,
    pub command_offset: u64,
    pub command_size: u64,
    pub status_offset: u64,
    pub status_size: u64,
    pub flags: u16,
    pub reserved1: [u8; 6usize],
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiNfitDataRegion {
    pub header: FfiAcpiNfitHeader,
    pub region_index: u16,
    pub windows: u16,
    pub offset: u64,
    pub size: u64,
    pub capacity: u64,
    pub start_address: u64,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiNfitFlushAddress {
    pub header: FfiAcpiNfitHeader,
    pub device_handle: u32,
    pub hint_count: u16,
    pub reserved: [u8; 6usize],
    pub hint_address: [u64; 1usize],
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiNfitCapabilities {
    pub header: FfiAcpiNfitHeader,
    pub highest_capability: u8,
    pub reserved: [u8; 3usize],
    pub capabilities: u32,
    pub reserved2: u32,
}
