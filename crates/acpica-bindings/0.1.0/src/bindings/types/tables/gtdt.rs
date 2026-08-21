use crate::bindings::types::FfiAcpiTableHeader;

///  GTDT - Generic Timer Description Table (ACPI 5.1)
///         Version 2
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableGtdt {
    pub header: FfiAcpiTableHeader,
    pub counter_block_addresss: u64,
    pub reserved: u32,
    pub secure_el1_interrupt: u32,
    pub secure_el1_flags: u32,
    pub non_secure_el1_interrupt: u32,
    pub non_secure_el1_flags: u32,
    pub virtual_timer_interrupt: u32,
    pub virtual_timer_flags: u32,
    pub non_secure_el2_interrupt: u32,
    pub non_secure_el2_flags: u32,
    pub counter_read_block_address: u64,
    pub platform_timer_count: u32,
    pub platform_timer_offset: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiGtdtEl2 {
    pub virtual_el2_timer_gsiv: u32,
    pub virtual_el2_timer_flags: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiGtdtHeader {
    pub header_type: u8,
    pub length: u16,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiGtdtType {
    TimerBlock = 0,
    Watchdog = 1,
    Reserved = 2,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiGtdtTimerBlock {
    pub header: FfiAcpiGtdtHeader,
    pub reserved: u8,
    pub block_address: u64,
    pub timer_count: u32,
    pub timer_offset: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiGtdtTimerEntry {
    pub frame_number: u8,
    pub reserved: [u8; 3usize],
    pub base_address: u64,
    pub el0_base_address: u64,
    pub timer_interrupt: u32,
    pub timer_flags: u32,
    pub virtual_timer_interrupt: u32,
    pub virtual_timer_flags: u32,
    pub common_flags: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiGtdtWatchdog {
    pub header: FfiAcpiGtdtHeader,
    pub reserved: u8,
    pub refresh_frame_address: u64,
    pub control_frame_address: u64,
    pub timer_interrupt: u32,
    pub timer_flags: u32,
}
