use crate::bindings::types::FfiAcpiTableHeader;

use super::FfiAcpiSubtableHeader;

///  SRAT - System Resource Affinity Table
///         Version 3
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableSrat {
    pub header: FfiAcpiTableHeader,
    pub table_revision: u32,
    pub reserved: u64,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiSratType {
    CpuAffinity = 0,
    MemoryAffinity = 1,
    X2apicCpuAffinity = 2,
    GiccAffinity = 3,
    GicItsAffinity = 4,
    GenericAffinity = 5,
    Reserved = 6,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiSratCpuAffinity {
    pub header: FfiAcpiSubtableHeader,
    pub proximity_domain_lo: u8,
    pub apic_id: u8,
    pub flags: u32,
    pub local_sapic_eid: u8,
    pub proximity_domain_hi: [u8; 3usize],
    pub clock_domain: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiSratMemAffinity {
    pub header: FfiAcpiSubtableHeader,
    pub proximity_domain: u32,
    pub reserved: u16,
    pub base_address: u64,
    pub length: u64,
    pub reserved1: u32,
    pub flags: u32,
    pub reserved2: u64,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiSratX2apicCpuAffinity {
    pub header: FfiAcpiSubtableHeader,
    pub reserved: u16,
    pub proximity_domain: u32,
    pub apic_id: u32,
    pub flags: u32,
    pub clock_domain: u32,
    pub reserved2: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiSratGiccAffinity {
    pub header: FfiAcpiSubtableHeader,
    pub proximity_domain: u32,
    pub acpi_processor_uid: u32,
    pub flags: u32,
    pub clock_domain: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiSratGicItsAffinity {
    pub header: FfiAcpiSubtableHeader,
    pub proximity_domain: u32,
    pub reserved: u16,
    pub its_id: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiSratGenericAffinity {
    pub header: FfiAcpiSubtableHeader,
    pub reserved: u8,
    pub device_handle_type: u8,
    pub proximity_domain: u32,
    pub device_handle: [u8; 16usize],
    pub flags: u32,
    pub reserved1: u32,
}
