use crate::bindings::types::FfiAcpiTableHeader;

///  HMAT - Heterogeneous Memory Attributes Table (ACPI 6.3)
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableHmat {
    pub header: FfiAcpiTableHeader,
    pub reserved: u32,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiHmatType {
    AddressRange = 0,
    Locality = 1,
    Cache = 2,
    Reserved = 3,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiHmatStructure {
    pub(crate) structure_type: u16,
    pub reserved: u16,
    pub length: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiHmatProximityDomain {
    pub header: FfiAcpiHmatStructure,
    pub flags: u16,
    pub reserved1: u16,
    pub initiator_pd: u32,
    pub memory_pd: u32,
    pub reserved2: u32,
    pub reserved3: u64,
    pub reserved4: u64,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiHmatLocality {
    pub header: FfiAcpiHmatStructure,
    pub flags: u8,
    pub data_type: u8,
    pub min_transfer_size: u8,
    pub reserved1: u8,
    pub number_of_initiator_p_ds: u32,
    pub number_of_target_p_ds: u32,
    pub reserved2: u32,
    pub entry_base_unit: u64,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiHmatCache {
    pub header: FfiAcpiHmatStructure,
    pub memory_pd: u32,
    pub reserved1: u32,
    pub cache_size: u64,
    pub cache_attributes: u32,
    pub reserved2: u16,
    pub number_of_smbios_handles: u16,
}
