use crate::bindings::types::FfiAcpiGenericAddress;

use super::FfiAcpiTableHeader;

/// FACS - Firmware ACPI Control Structure (FACS)
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableFacs {
    pub signature: [i8; 4usize],
    pub length: u32,
    pub hardware_signature: u32,
    pub firmware_waking_vector: u32,
    pub global_lock: u32,
    pub flags: u32,
    pub x_firmware_waking_vector: u64,
    pub version: u8,
    pub reserved: [u8; 3usize],
    pub ospm_flags: u32,
    pub reserved1: [u8; 24usize],
}

///  FADT - Fixed ACPI Description Table (Signature \"FACP\")
///         Version 6
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableFadt {
    pub header: FfiAcpiTableHeader,
    pub facs: u32,
    pub dsdt: u32,
    pub model: u8,
    pub preferred_profile: u8,
    pub sci_interrupt: u16,
    pub smi_command: u32,
    pub ffi_acpi_enable: u8,
    pub ffi_acpi_disable: u8,
    pub s4_bios_request: u8,
    pub pstate_control: u8,
    pub pm1a_event_block: u32,
    pub pm1b_event_block: u32,
    pub pm1a_control_block: u32,
    pub pm1b_control_block: u32,
    pub pm2_control_block: u32,
    pub pm_timer_block: u32,
    pub gpe0_block: u32,
    pub gpe1_block: u32,
    pub pm1_event_length: u8,
    pub pm1_control_length: u8,
    pub pm2_control_length: u8,
    pub pm_timer_length: u8,
    pub gpe0_block_length: u8,
    pub gpe1_block_length: u8,
    pub gpe1_base: u8,
    pub cst_control: u8,
    pub c2_latency: u16,
    pub c3_latency: u16,
    pub flush_size: u16,
    pub flush_stride: u16,
    pub duty_offset: u8,
    pub duty_width: u8,
    pub day_alarm: u8,
    pub month_alarm: u8,
    pub century: u8,
    pub boot_flags: u16,
    pub reserved: u8,
    pub flags: u32,
    pub reset_register: FfiAcpiGenericAddress,
    pub reset_value: u8,
    pub arm_boot_flags: u16,
    pub minor_revision: u8,
    pub x_facs: u64,
    pub x_dsdt: u64,
    pub x_pm1a_event_block: FfiAcpiGenericAddress,
    pub x_pm1b_event_block: FfiAcpiGenericAddress,
    pub x_pm1a_control_block: FfiAcpiGenericAddress,
    pub x_pm1b_control_block: FfiAcpiGenericAddress,
    pub x_pm2_control_block: FfiAcpiGenericAddress,
    pub x_pm_timer_block: FfiAcpiGenericAddress,
    pub x_gpe0_block: FfiAcpiGenericAddress,
    pub x_gpe1_block: FfiAcpiGenericAddress,
    pub sleep_control: FfiAcpiGenericAddress,
    pub sleep_status: FfiAcpiGenericAddress,
    pub hypervisor_id: u64,
}
