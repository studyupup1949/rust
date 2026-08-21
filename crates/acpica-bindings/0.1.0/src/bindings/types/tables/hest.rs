use crate::bindings::types::{FfiAcpiGenericAddress, FfiAcpiTableHeader};

///  HEST - Hardware Error Source Table (ACPI 4.0)
///         Version 1
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableHest {
    pub header: FfiAcpiTableHeader,
    pub error_source_count: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiHestHeader {
    pub header_type: u16,
    pub source_id: u16,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiHestTypes {
    TypeIa32Check = 0,
    TypeIa32CorrectedCheck = 1,
    TypeIa32Nmi = 2,
    TypeNotUsed3 = 3,
    TypeNotUsed4 = 4,
    TypeNotUsed5 = 5,
    TypeAerRootPort = 6,
    TypeAerEndpoint = 7,
    TypeAerBridge = 8,
    TypeGenericError = 9,
    TypeGenericErrorV2 = 10,
    TypeIa32DeferredCheck = 11,
    TypeReserved = 12,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiHestIaErrorBank {
    pub bank_number: u8,
    pub clear_status_on_init: u8,
    pub status_format: u8,
    pub reserved: u8,
    pub control_register: u32,
    pub control_data: u64,
    pub status_register: u32,
    pub address_register: u32,
    pub misc_register: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiHestAerCommon {
    pub reserved1: u16,
    pub flags: u8,
    pub enabled: u8,
    pub records_to_preallocate: u32,
    pub max_sections_per_record: u32,
    pub bus: u32,
    pub device: u16,
    pub function: u16,
    pub device_control: u16,
    pub reserved2: u16,
    pub uncorrectable_mask: u32,
    pub uncorrectable_severity: u32,
    pub correctable_mask: u32,
    pub advanced_capabilities: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiHestNotify {
    pub notify_type: u8,
    pub length: u8,
    pub config_write_enable: u16,
    pub poll_interval: u32,
    pub vector: u32,
    pub polling_threshold_value: u32,
    pub polling_threshold_window: u32,
    pub error_threshold_value: u32,
    pub error_threshold_window: u32,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiHestNotifyTypes {
    Polled = 0,
    External = 1,
    Local = 2,
    Sci = 3,
    Nmi = 4,
    Cmci = 5,
    Mce = 6,
    Gpio = 7,
    Sea = 8,
    Sei = 9,
    Gsiv = 10,
    SoftwareDelegated = 11,
    Reserved = 12,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiHestIaMachineCheck {
    pub header: FfiAcpiHestHeader,
    pub reserved1: u16,
    pub flags: u8,
    pub enabled: u8,
    pub records_to_preallocate: u32,
    pub max_sections_per_record: u32,
    pub global_capability_data: u64,
    pub global_control_data: u64,
    pub num_hardware_banks: u8,
    pub reserved3: [u8; 7usize],
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiHestIaCorrected {
    pub header: FfiAcpiHestHeader,
    pub reserved1: u16,
    pub flags: u8,
    pub enabled: u8,
    pub records_to_preallocate: u32,
    pub max_sections_per_record: u32,
    pub notify: FfiAcpiHestNotify,
    pub num_hardware_banks: u8,
    pub reserved2: [u8; 3usize],
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiHestIaNmi {
    pub header: FfiAcpiHestHeader,
    pub reserved: u32,
    pub records_to_preallocate: u32,
    pub max_sections_per_record: u32,
    pub max_raw_data_length: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiHestAerRoot {
    pub header: FfiAcpiHestHeader,
    pub aer: FfiAcpiHestAerCommon,
    pub root_error_command: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiHestAer {
    pub header: FfiAcpiHestHeader,
    pub aer: FfiAcpiHestAerCommon,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiHestAerBridge {
    pub header: FfiAcpiHestHeader,
    pub aer: FfiAcpiHestAerCommon,
    pub uncorrectable_mask2: u32,
    pub uncorrectable_severity2: u32,
    pub advanced_capabilities2: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiHestGeneric {
    pub header: FfiAcpiHestHeader,
    pub related_source_id: u16,
    pub reserved: u8,
    pub enabled: u8,
    pub records_to_preallocate: u32,
    pub max_sections_per_record: u32,
    pub max_raw_data_length: u32,
    pub error_status_address: FfiAcpiGenericAddress,
    pub notify: FfiAcpiHestNotify,
    pub error_block_length: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiHestGenericV2 {
    pub header: FfiAcpiHestHeader,
    pub related_source_id: u16,
    pub reserved: u8,
    pub enabled: u8,
    pub records_to_preallocate: u32,
    pub max_sections_per_record: u32,
    pub max_raw_data_length: u32,
    pub error_status_address: FfiAcpiGenericAddress,
    pub notify: FfiAcpiHestNotify,
    pub error_block_length: u32,
    pub read_ack_register: FfiAcpiGenericAddress,
    pub read_ack_preserve: u64,
    pub read_ack_write: u64,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiHestGenericStatus {
    pub block_status: u32,
    pub raw_data_offset: u32,
    pub raw_data_length: u32,
    pub data_length: u32,
    pub error_severity: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiHestGenericData {
    pub section_type: [u8; 16usize],
    pub error_severity: u32,
    pub revision: u16,
    pub validation_bits: u8,
    pub flags: u8,
    pub error_data_length: u32,
    pub fru_id: [u8; 16usize],
    pub fru_text: [u8; 20usize],
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiHestGenericDataV300 {
    pub section_type: [u8; 16usize],
    pub error_severity: u32,
    pub revision: u16,
    pub validation_bits: u8,
    pub flags: u8,
    pub error_data_length: u32,
    pub fru_id: [u8; 16usize],
    pub fru_text: [u8; 20usize],
    pub time_stamp: u64,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiHestIaDeferredCheck {
    pub header: FfiAcpiHestHeader,
    pub reserved1: u16,
    pub flags: u8,
    pub enabled: u8,
    pub records_to_preallocate: u32,
    pub max_sections_per_record: u32,
    pub notify: FfiAcpiHestNotify,
    pub num_hardware_banks: u8,
    pub reserved2: [u8; 3usize],
}
