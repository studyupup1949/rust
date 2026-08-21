//! The [`Fadt`] type

use core::fmt::Debug;

use bitfield_struct::bitfield;

use crate::{bindings::types::tables::fadt::FfiAcpiTableFadt, types::AcpiGenericAddress};

use super::AcpiTableHeader;

/// The FADT table
pub struct Fadt<'a>(&'a FfiAcpiTableFadt);

impl<'a> Fadt<'a> {
    pub(crate) fn from_ffi(ffi_ptr: &'a FfiAcpiTableFadt) -> Self {
        Self(ffi_ptr)
    }
}

impl<'a> Debug for Fadt<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Fadt")
            .field("header", &self.header())
            .field("facs", &self.facs())
            .field("dsdt", &self.dsdt())
            .field("model", &self.model())
            .field("preferred_profile", &self.preferred_profile())
            .field("sci_interrupt", &self.sci_interrupt())
            .field("smi_command", &self.smi_command())
            .field("ffi_acpi_enable", &self.ffi_acpi_enable())
            .field("ffi_acpi_disable", &self.ffi_acpi_disable())
            .field("s4_bios_request", &self.s4_bios_request())
            .field("pstate_control", &self.pstate_control())
            .field("pm1a_event_block", &self.pm1a_event_block())
            .field("pm1b_event_block", &self.pm1b_event_block())
            .field("pm1a_control_block", &self.pm1a_control_block())
            .field("pm1b_control_block", &self.pm1b_control_block())
            .field("pm2_control_block", &self.pm2_control_block())
            .field("pm_timer_block", &self.pm_timer_block())
            .field("gpe0_block", &self.gpe0_block())
            .field("gpe1_block", &self.gpe1_block())
            .field("pm1_event_length", &self.pm1_event_length())
            .field("pm1_control_length", &self.pm1_control_length())
            .field("pm2_control_length", &self.pm2_control_length())
            .field("pm_timer_length", &self.pm_timer_length())
            .field("gpe0_block_length", &self.gpe0_block_length())
            .field("gpe1_block_length", &self.gpe1_block_length())
            .field("gpe1_base", &self.gpe1_base())
            .field("cst_control", &self.cst_control())
            .field("c2_latency", &self.c2_latency())
            .field("c3_latency", &self.c3_latency())
            .field("flush_size", &self.flush_size())
            .field("flush_stride", &self.flush_stride())
            .field("duty_offset", &self.duty_offset())
            .field("duty_width", &self.duty_width())
            .field("day_alarm", &self.day_alarm())
            .field("month_alarm", &self.month_alarm())
            .field("century", &self.century())
            .field("boot_flags", &self.boot_architecture_flags())
            .field("flags", &self.flags())
            .field("reset_register", &self.reset_register())
            .field("reset_value", &self.reset_value())
            .field("arm_boot_flags", &self.arm_boot_flags())
            .field("minor_revision", &self.minor_revision())
            .field("x_facs", &self.x_facs())
            .field("x_dsdt", &self.x_dsdt())
            .field("x_pm1a_event_block", &self.x_pm1a_event_block())
            .field("x_pm1b_event_block", &self.x_pm1b_event_block())
            .field("x_pm1a_control_block", &self.x_pm1a_control_block())
            .field("x_pm1b_control_block", &self.x_pm1b_control_block())
            .field("x_pm2_control_block", &self.x_pm2_control_block())
            .field("x_pm_timer_block", &self.x_pm_timer_block())
            .field("x_gpe0_block", &self.x_gpe0_block())
            .field("x_gpe1_block", &self.x_gpe1_block())
            .field("sleep_control", &self.sleep_control())
            .field("sleep_status", &self.sleep_status())
            .field("hypervisor_id", &self.hypervisor_id())
            .finish()
    }
}

#[allow(missing_docs)] // TODO: DOCS
#[rustfmt::skip]
impl<'a> Fadt<'a> {
    #[must_use] pub fn header(&self) -> AcpiTableHeader { AcpiTableHeader::from_ffi(&self.0.header) }
    #[must_use] pub fn facs(&self) -> u32 { self.0.facs }
    #[must_use] pub fn dsdt(&self) -> u32 { self.0.dsdt }
    #[must_use] pub fn model(&self) -> u8 { self.0.model }
    #[must_use] pub fn preferred_profile(&self) -> u8 { self.0.preferred_profile }
    #[must_use] pub fn sci_interrupt(&self) -> u16 { self.0.sci_interrupt }
    #[must_use] pub fn smi_command(&self) -> u32 { self.0.smi_command }
    #[must_use] pub fn ffi_acpi_enable(&self) -> u8 { self.0.ffi_acpi_enable }
    #[must_use] pub fn ffi_acpi_disable(&self) -> u8 { self.0.ffi_acpi_disable }
    #[must_use] pub fn s4_bios_request(&self) -> u8 { self.0.s4_bios_request }
    #[must_use] pub fn pstate_control(&self) -> u8 { self.0.pstate_control }
    #[must_use] pub fn pm1a_event_block(&self) -> u32 { self.0.pm1a_event_block }
    #[must_use] pub fn pm1b_event_block(&self) -> u32 { self.0.pm1b_event_block }
    #[must_use] pub fn pm1a_control_block(&self) -> u32 { self.0.pm1a_control_block }
    #[must_use] pub fn pm1b_control_block(&self) -> u32 { self.0.pm1b_control_block }
    #[must_use] pub fn pm2_control_block(&self) -> u32 { self.0.pm2_control_block }
    #[must_use] pub fn pm_timer_block(&self) -> u32 { self.0.pm_timer_block }
    #[must_use] pub fn gpe0_block(&self) -> u32 { self.0.gpe0_block }
    #[must_use] pub fn gpe1_block(&self) -> u32 { self.0.gpe1_block }
    #[must_use] pub fn pm1_event_length(&self) -> u8 { self.0.pm1_event_length }
    #[must_use] pub fn pm1_control_length(&self) -> u8 { self.0.pm1_control_length }
    #[must_use] pub fn pm2_control_length(&self) -> u8 { self.0.pm2_control_length }
    #[must_use] pub fn pm_timer_length(&self) -> u8 { self.0.pm_timer_length }
    #[must_use] pub fn gpe0_block_length(&self) -> u8 { self.0.gpe0_block_length }
    #[must_use] pub fn gpe1_block_length(&self) -> u8 { self.0.gpe1_block_length }
    #[must_use] pub fn gpe1_base(&self) -> u8 { self.0.gpe1_base }
    #[must_use] pub fn cst_control(&self) -> u8 { self.0.cst_control }
    #[must_use] pub fn c2_latency(&self) -> u16 { self.0.c2_latency }
    #[must_use] pub fn c3_latency(&self) -> u16 { self.0.c3_latency }
    #[must_use] pub fn flush_size(&self) -> u16 { self.0.flush_size }
    #[must_use] pub fn flush_stride(&self) -> u16 { self.0.flush_stride }
    #[must_use] pub fn duty_offset(&self) -> u8 { self.0.duty_offset }
    #[must_use] pub fn duty_width(&self) -> u8 { self.0.duty_width }
    #[must_use] pub fn day_alarm(&self) -> u8 { self.0.day_alarm }
    #[must_use] pub fn month_alarm(&self) -> u8 { self.0.month_alarm }
    #[must_use] pub fn century(&self) -> u8 { self.0.century }
    #[must_use] pub fn flags(&self) -> u32 { self.0.flags }
}

#[allow(missing_docs)] // TODO: DOCS
#[rustfmt::skip]
impl<'a> Fadt<'a> {
    /// Gets the `boot_architecture_flags` field of the FADT, if the ACPI version in use supports it.
    /// This field is available in versions 2.0+ of ACPI.
    #[must_use] pub fn boot_architecture_flags(&self) -> Option<BootArchitectureFlags> { 
        if self.header().revision() < 2 { None } else { Some(BootArchitectureFlags::from(self.0.boot_flags)) }
    }
    /// Gets the `reset_register` field of the FADT, if the ACPI version in use supports it.
    /// This field is available in versions 2.0+ of ACPI.
    #[must_use] pub fn reset_register(&self) -> Option<AcpiGenericAddress> {
        if self.header().revision() < 2 {None} else { Some(AcpiGenericAddress::from_ffi(self.0.reset_register)) }
    }
    /// Gets the `reset_value` field of the FADT, if the ACPI version in use supports it.
    /// This field is available in versions 2.0+ of ACPI.
    #[must_use] pub fn reset_value(&self) -> Option<u8> {
        if self.header().revision() < 2 {None} else { Some(self.0.reset_value) }
    }
    /// Gets the `arm_boot_flags` field of the FADT, if the ACPI version in use supports it.
    /// This field is available in versions 2.0+ of ACPI.
    #[must_use] pub fn arm_boot_flags(&self) -> Option<u16> {
        if self.header().revision() < 2 {None} else { Some(self.0.arm_boot_flags) }
    }
    /// Gets the `minor_revision` field of the FADT, if the ACPI version in use supports it.
    /// This field is available in versions 2.0+ of ACPI.
    #[must_use] pub fn minor_revision(&self) -> Option<u8> {
        if self.header().revision() < 2 {None} else { Some(self.0.minor_revision) }
    }
    /// Gets the `x_facs` field of the FADT, if the ACPI version in use supports it.
    /// This field is available in versions 2.0+ of ACPI.
    #[must_use] pub fn x_facs(&self) -> Option<u64> {
        if self.header().revision() < 2 {None} else { Some(self.0.x_facs) }
    }
    /// Gets the `x_dsdt` field of the FADT, if the ACPI version in use supports it.
    /// This field is available in versions 2.0+ of ACPI.
    #[must_use] pub fn x_dsdt(&self) -> Option<u64> {
        if self.header().revision() < 2 {None} else { Some(self.0.x_dsdt) }
    }
    /// Gets the `x_pm1a_event_block` field of the FADT, if the ACPI version in use supports it.
    /// This field is available in versions 2.0+ of ACPI.
    #[must_use] pub fn x_pm1a_event_block(&self) -> Option<AcpiGenericAddress> {
        if self.header().revision() < 2 {None} else { Some(AcpiGenericAddress::from_ffi(self.0.x_pm1a_event_block)) }
    }
    /// Gets the `x_pm1b_event_block` field of the FADT, if the ACPI version in use supports it.
    /// This field is available in versions 2.0+ of ACPI.
    #[must_use] pub fn x_pm1b_event_block(&self) -> Option<AcpiGenericAddress> {
        if self.header().revision() < 2 {None} else { Some(AcpiGenericAddress::from_ffi(self.0.x_pm1b_event_block)) }
    }
    /// Gets the `x_pm1a_control_block` field of the FADT, if the ACPI version in use supports it.
    /// This field is available in versions 2.0+ of ACPI.
    #[must_use] pub fn x_pm1a_control_block(&self) -> Option<AcpiGenericAddress> {
        if self.header().revision() < 2 {None} else { Some(AcpiGenericAddress::from_ffi(self.0.x_pm1a_control_block)) }
    }
    /// Gets the `x_pm1b_control_block` field of the FADT, if the ACPI version in use supports it.
    /// This field is available in versions 2.0+ of ACPI.
    #[must_use] pub fn x_pm1b_control_block(&self) -> Option<AcpiGenericAddress> {
        if self.header().revision() < 2 {None} else { Some(AcpiGenericAddress::from_ffi(self.0.x_pm1b_control_block)) }
    }
    /// Gets the `x_pm2_control_block` field of the FADT, if the ACPI version in use supports it.
    /// This field is available in versions 2.0+ of ACPI.
    #[must_use] pub fn x_pm2_control_block(&self) -> Option<AcpiGenericAddress> {
        if self.header().revision() < 2 {None} else { Some(AcpiGenericAddress::from_ffi(self.0.x_pm2_control_block)) }
    }
    /// Gets the `x_pm_timer_block` field of the FADT, if the ACPI version in use supports it.
    /// This field is available in versions 2.0+ of ACPI.
    #[must_use] pub fn x_pm_timer_block(&self) -> Option<AcpiGenericAddress> {
        if self.header().revision() < 2 {None} else { Some(AcpiGenericAddress::from_ffi(self.0.x_pm_timer_block)) }
    }
    /// Gets the `x_gpe0_block` field of the FADT, if the ACPI version in use supports it.
    /// This field is available in versions 2.0+ of ACPI.
    #[must_use] pub fn x_gpe0_block(&self) -> Option<AcpiGenericAddress> {
        if self.header().revision() < 2 {None} else { Some(AcpiGenericAddress::from_ffi(self.0.x_gpe0_block)) }
    }
    /// Gets the `x_gpe1_block` field of the FADT, if the ACPI version in use supports it.
    /// This field is available in versions 2.0+ of ACPI.
    #[must_use] pub fn x_gpe1_block(&self) -> Option<AcpiGenericAddress> {
        if self.header().revision() < 2 {None} else { Some(AcpiGenericAddress::from_ffi(self.0.x_gpe1_block)) }
    }


    /// Gets the `sleep_control` field of the FADT, if the ACPI version in use supports it.
    /// This field is available in versions 5.0+ of ACPI.
    #[must_use] pub fn sleep_control(&self) -> Option<AcpiGenericAddress> {
        if self.header().revision() < 5 {None} else { Some(AcpiGenericAddress::from_ffi(self.0.sleep_control)) }
    }
    /// Gets the `sleep_status` field of the FADT, if the ACPI version in use supports it.
    /// This field is available in versions 5.0+ of ACPI.
    #[must_use] pub fn sleep_status(&self) -> Option<AcpiGenericAddress> {
        if self.header().revision() < 5 {None} else { Some(AcpiGenericAddress::from_ffi(self.0.sleep_status)) }
    }

    /// Gets the `hypervisor_id` field of the FADT, if the ACPI version in use supports it.
    /// This field is available in versions 6.0+ of ACPI.
    #[must_use] pub fn hypervisor_id(&self) -> Option<u64> {
        if self.header().revision() < 6 {None} else { Some(self.0.hypervisor_id) }
    }
}

/// A power management profile given to the OS by firmware
#[derive(Debug)]
pub enum PowerManagementProfile {
    /// The profile is not specified - the OS should make a guess at the type of hardware or should use a generic power management profile
    Unspecified,
    /// The OS should use a desktop power management profile
    Desktop,
    /// The OS should use a mobile power management profile
    Mobile,
    /// The OS should use a workstation power management profile
    Workstation,
    /// The OS should use an enterprise server power management profile
    EnterpriseServer,
    /// The OS should use a Soho server power management profile
    SohoServer,
    /// The OS should use an appliance PC power management profile
    AppliancePC,
    /// The OS should use a performance server power management profile
    PerformanceServer,
    /// The OS should use a tablet power management profile
    Tablet,

    /// The value is unspecified. The specific value is stored in case the OS knows what the value means even if this library doesn't.
    Reserved(u8),
}

impl PowerManagementProfile {
    fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Unspecified,
            1 => Self::Desktop,
            2 => Self::Mobile,
            3 => Self::Workstation,
            4 => Self::EnterpriseServer,
            5 => Self::SohoServer,
            6 => Self::AppliancePC,
            7 => Self::PerformanceServer,
            8 => Self::Tablet,

            _ => Self::Reserved(value),
        }
    }
}

/// Flags which can be used by the OS to guide the assumptions it can make in initializing hardware on IA-PC platforms.
#[bitfield(u16, debug = false)]
pub struct BootArchitectureFlags {
    f_legacy_devices: bool,
    f_has_8042_controller: bool,
    f_vga_not_present: bool,
    f_msi_not_supported: bool,
    f_pcie_aspm_controls: bool,
    f_cmos_rtc_not_present: bool,

    #[bits(10)]
    _reserved: (),
}

#[rustfmt::skip]
impl BootArchitectureFlags {
    /// Whether the motherboard supports user-visible devices on the LPC or ISA bus. 
    /// User-visible devices are devices that have end-user accessible connectors (for example, LPT port), 
    /// or devices for which the OS must load a device driver so that an end-user application can use a device. 
    /// If `false`, the OS may assume there are no such devices and that all devices in the system can be detected 
    /// exclusively via industry standard device enumeration mechanisms (including the ACPI namespace).
    #[must_use] pub fn legacy_devices(&self) -> bool { self.f_legacy_devices() }
    /// Whether the motherboard contains support for a port 60 and 64 based keyboard controller,
    ///  usually implemented as an 8042 or equivalent micro-controller.
    #[must_use] pub fn has_8042_controller(&self) -> bool { self.f_has_8042_controller() }
    /// Whether OSPM must not blindly probe the VGA hardware (that responds to MMIO addresses A0000h-BFFFFh and IO ports 3B0h-3BBh and 3C0h-3DFh)
    /// that may cause machine check on this system. If `false`, it is safe to probe the VGA hardware.
    #[must_use] pub fn vga_not_present(&self) -> bool { self.f_vga_not_present() }
    /// Whether OSPM must not enable Message Signalled Interrupts (MSI) on this platform.
    #[must_use] pub fn msi_not_supported(&self) -> bool { self.f_msi_not_supported() }
    /// Whether OSPM must not enable OSPM ASPM control on this platform.
    #[must_use] pub fn pcie_aspm_controls(&self) -> bool { self.f_pcie_aspm_controls() }
    /// Whether the CMOS RTC is either not implemented, or does not exist at the legacy addresses. 
    /// OSPM uses the Control Method Time and Alarm Namespace device instead.
    #[must_use] pub fn cmos_rtc_not_present(&self) -> bool { self.f_cmos_rtc_not_present() }
}

impl Debug for BootArchitectureFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BootArchitectureFlags")
            .field("legacy_devices", &self.legacy_devices())
            .field("has_8042_controller", &self.has_8042_controller())
            .field("vga_not_present", &self.vga_not_present())
            .field("msi_not_supported", &self.msi_not_supported())
            .field("pcie_aspm_controls", &self.pcie_aspm_controls())
            .field("cmos_rtc_not_present", &self.cmos_rtc_not_present())
            .finish()
    }
}
