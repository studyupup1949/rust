use crate::bindings::types::FfiAcpiTableHeader;

///  FPDT - Firmware Performance Data Table (ACPI 5.0)
///         Version 1
///
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableFpdt {
    pub header: FfiAcpiTableHeader,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiFpdtHeader {
    pub header_type: u16,
    pub length: u8,
    pub revision: u8,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiFpdtType {
    Boot = 0,
    S3perf = 1,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiFpdtBootPointer {
    pub header: FfiAcpiFpdtHeader,
    pub reserved: [u8; 4usize],
    pub address: u64,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiFpdtS3ptPointer {
    pub header: FfiAcpiFpdtHeader,
    pub reserved: [u8; 4usize],
    pub address: u64,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableS3pt {
    pub signature: [u8; 4usize],
    pub length: u32,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiS3ptType {
    S3ptTypeResume = 0,
    S3ptTypeSuspend = 1,
    FpdtBootPerformance = 2,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiS3ptResume {
    pub header: FfiAcpiFpdtHeader,
    pub resume_count: u32,
    pub full_resume: u64,
    pub average_resume: u64,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiS3ptSuspend {
    pub header: FfiAcpiFpdtHeader,
    pub suspend_start: u64,
    pub suspend_end: u64,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiFpdtBoot {
    pub header: FfiAcpiFpdtHeader,
    pub reserved: [u8; 4usize],
    pub reset_end: u64,
    pub load_start: u64,
    pub startup_start: u64,
    pub exit_services_entry: u64,
    pub exit_services_exit: u64,
}
