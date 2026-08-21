use crate::bindings::types::FfiAcpiTableHeader;

///  DRTM - Dynamic Root of Trust for Measurement table
///  Conforms to \"TCG D-RTM Architecture\" June 17 2013, Version 1.0.0
///  Table version 1
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableDrtm {
    pub header: FfiAcpiTableHeader,
    pub entry_base_address: u64,
    pub entry_length: u64,
    pub entry_address32: u32,
    pub entry_address64: u64,
    pub exit_address: u64,
    pub log_area_address: u64,
    pub log_area_length: u32,
    pub arch_dependent_address: u64,
    pub flags: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiDrtmVtableList {
    pub validated_table_count: u32,
    pub validated_tables: [u64; 1usize],
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiDrtmResource {
    pub size: [u8; 7usize],
    pub resource_type: u8,
    pub address: u64,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiDrtmResourceList {
    pub resource_count: u32,
    pub resources: [FfiAcpiDrtmResource; 1usize],
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiDrtmDpsId {
    pub dps_id_length: u32,
    pub dps_id: [u8; 16usize],
}
