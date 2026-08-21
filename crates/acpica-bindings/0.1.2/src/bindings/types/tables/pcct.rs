use crate::bindings::types::FfiAcpiGenericAddress;

use super::{FfiAcpiSubtableHeader, FfiAcpiTableHeader};

///  PCCT - Platform Communications Channel Table (ACPI 5.0)
///         Version 2 (ACPI 6.2)
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTablePcct {
    pub header: FfiAcpiTableHeader,
    pub flags: u32,
    pub reserved: u64,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiPcctType {
    GenericSubspace = 0,
    HwReducedSubspace = 1,
    HwReducedSubspaceType2 = 2,
    ExtPccMasterSubspace = 3,
    ExtPccSlaveSubspace = 4,
    HwRegCommSubspace = 5,
    Reserved = 6,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiPcctSubspace {
    pub header: FfiAcpiSubtableHeader,
    pub reserved: [u8; 6usize],
    pub base_address: u64,
    pub length: u64,
    pub doorbell_register: FfiAcpiGenericAddress,
    pub preserve_mask: u64,
    pub write_mask: u64,
    pub latency: u32,
    pub max_access_rate: u32,
    pub min_turnaround_time: u16,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiPcctHwReduced {
    pub header: FfiAcpiSubtableHeader,
    pub platform_interrupt: u32,
    pub flags: u8,
    pub reserved: u8,
    pub base_address: u64,
    pub length: u64,
    pub doorbell_register: FfiAcpiGenericAddress,
    pub preserve_mask: u64,
    pub write_mask: u64,
    pub latency: u32,
    pub max_access_rate: u32,
    pub min_turnaround_time: u16,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiPcctHwReducedType2 {
    pub header: FfiAcpiSubtableHeader,
    pub platform_interrupt: u32,
    pub flags: u8,
    pub reserved: u8,
    pub base_address: u64,
    pub length: u64,
    pub doorbell_register: FfiAcpiGenericAddress,
    pub preserve_mask: u64,
    pub write_mask: u64,
    pub latency: u32,
    pub max_access_rate: u32,
    pub min_turnaround_time: u16,
    pub platform_ack_register: FfiAcpiGenericAddress,
    pub ack_preserve_mask: u64,
    pub ack_write_mask: u64,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiPcctExtPccMaster {
    pub header: FfiAcpiSubtableHeader,
    pub platform_interrupt: u32,
    pub flags: u8,
    pub reserved1: u8,
    pub base_address: u64,
    pub length: u32,
    pub doorbell_register: FfiAcpiGenericAddress,
    pub preserve_mask: u64,
    pub write_mask: u64,
    pub latency: u32,
    pub max_access_rate: u32,
    pub min_turnaround_time: u32,
    pub platform_ack_register: FfiAcpiGenericAddress,
    pub ack_preserve_mask: u64,
    pub ack_set_mask: u64,
    pub reserved2: u64,
    pub cmd_complete_register: FfiAcpiGenericAddress,
    pub cmd_complete_mask: u64,
    pub cmd_update_register: FfiAcpiGenericAddress,
    pub cmd_update_preserve_mask: u64,
    pub cmd_update_set_mask: u64,
    pub error_status_register: FfiAcpiGenericAddress,
    pub error_status_mask: u64,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiPcctExtPccSlave {
    pub header: FfiAcpiSubtableHeader,
    pub platform_interrupt: u32,
    pub flags: u8,
    pub reserved1: u8,
    pub base_address: u64,
    pub length: u32,
    pub doorbell_register: FfiAcpiGenericAddress,
    pub preserve_mask: u64,
    pub write_mask: u64,
    pub latency: u32,
    pub max_access_rate: u32,
    pub min_turnaround_time: u32,
    pub platform_ack_register: FfiAcpiGenericAddress,
    pub ack_preserve_mask: u64,
    pub ack_set_mask: u64,
    pub reserved2: u64,
    pub cmd_complete_register: FfiAcpiGenericAddress,
    pub cmd_complete_mask: u64,
    pub cmd_update_register: FfiAcpiGenericAddress,
    pub cmd_update_preserve_mask: u64,
    pub cmd_update_set_mask: u64,
    pub error_status_register: FfiAcpiGenericAddress,
    pub error_status_mask: u64,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiPcctHwReg {
    pub header: FfiAcpiSubtableHeader,
    pub version: u16,
    pub base_address: u64,
    pub length: u64,
    pub doorbell_register: FfiAcpiGenericAddress,
    pub doorbell_preserve: u64,
    pub doorbell_write: u64,
    pub cmd_complete_register: FfiAcpiGenericAddress,
    pub cmd_complete_mask: u64,
    pub error_status_register: FfiAcpiGenericAddress,
    pub error_status_mask: u64,
    pub nominal_latency: u32,
    pub min_turnaround_time: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiPcctSharedMemory {
    pub signature: u32,
    pub command: u16,
    pub status: u16,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiPcctExtPccSharedMemory {
    pub signature: u32,
    pub flags: u32,
    pub length: u32,
    pub command: u32,
}
