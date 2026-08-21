use crate::bindings::types::{FfiAcpiTableHeader, IncompleteArrayField};

///  AEST - Arm Error Source Table
///
///  Conforms to: ACPI for the Armv8 RAS Extensions 1.1 Platform Design Document
///  September 2020.
///
#[repr(C, packed)]
pub(crate) struct FfiAcpiTableAest {
    pub header: FfiAcpiTableHeader,
    node_array: IncompleteArrayField<*mut ::core::ffi::c_void>,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiAestHdr {
    pub node_type: u8,
    pub length: u16,
    pub reserved: u8,
    pub node_specific_offset: u32,
    pub node_interface_offset: u32,
    pub node_interrupt_offset: u32,
    pub node_interrupt_count: u32,
    pub timestamp_rate: u64,
    pub reserved1: u64,
    pub error_injection_rate: u64,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiAestProcessor {
    pub processor_id: u32,
    pub resource_type: u8,
    pub reserved: u8,
    pub flags: u8,
    pub revision: u8,
    pub processor_affinity: u64,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiAestProcessorCache {
    pub cache_reference: u32,
    pub reserved: u32,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiAestProcessorTlb {
    pub tlb_level: u32,
    pub reserved: u32,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiAestProcessorGeneric {
    pub resource: *mut u8,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiAestMemory {
    pub srat_proximity_domain: u32,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiAestSmmu {
    pub iort_node_reference: u32,
    pub subcomponent_reference: u32,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiAestVendor {
    pub acpi_hid: u32,
    pub acpi_uid: u32,
    pub vendor_specific_data: [u8; 16usize],
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiAestGic {
    pub interface_type: u32,
    pub instance_id: u32,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiAestNodeInterface {
    pub node_type: u8,
    pub reserved: [u8; 3usize],
    pub flags: u32,
    pub address: u64,
    pub error_record_index: u32,
    pub error_record_count: u32,
    pub error_record_implemented: u64,
    pub error_status_reporting: u64,
    pub addressing_mode: u64,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiAestNodeInterrupt {
    pub node_type: u8,
    pub reserved: [u8; 2usize],
    pub flags: u8,
    pub gsiv: u32,
    pub iort_id: u8,
    pub reserved1: [u8; 3usize],
}
