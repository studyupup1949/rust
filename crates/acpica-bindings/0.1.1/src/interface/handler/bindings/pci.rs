use log::debug;

use crate::{
    bindings::types::FfiAcpiPciId,
    interface::OS_INTERFACE,
    status::{AcpiError, AcpiErrorAsStatusExt, AcpiStatus},
    types::AcpiPciId,
};

#[export_name = "AcpiOsReadPciConfiguration"]
extern "C" fn acpi_os_read_pci_configuration(
    pci_id: *const FfiAcpiPciId,
    reg: u32,
    value: *mut u64,
    width: u32,
) -> AcpiStatus {
    if value.is_null() || pci_id.is_null() {
        return AcpiError::BadParameter.to_acpi_status();
    }

    debug!(target: "acpi_os_read_pci_configuration", "Reading {width} bytes from {pci_id:?}");

    // SAFETY: `pci_id` is non-null so it is valid
    let pci_id = unsafe { AcpiPciId::from_ffi(pci_id.read()) };

    let mut interface = OS_INTERFACE.lock();
    let interface = interface.as_mut().unwrap();

    let reg = reg.try_into().unwrap();

    // SAFETY: This is `AcpiOsReadPciConfiguration`,
    // and the read was instructed by ACPI so it should be sound
    let r = unsafe {
        match width {
            8 => interface.read_pci_config_u8(pci_id, reg).map(u8::into),
            16 => interface.read_pci_config_u16(pci_id, reg).map(u16::into),
            32 => interface.read_pci_config_u32(pci_id, reg).map(u32::into),
            64 => interface.read_pci_config_u64(pci_id, reg),
            _ => panic!("Invalid valid of `width`"),
        }
    };

    let v = match r {
        Ok(v) => v,
        Err(e) => return e.to_acpi_status(),
    };

    // SAFETY: `value` is a valid pointer as it is non-null
    unsafe {
        *value = v;
    }

    AcpiStatus::OK
}

#[export_name = "AcpiOsWritePciConfiguration"]
extern "C" fn acpi_os_write_pci_configuration(
    pci_id: FfiAcpiPciId,
    reg: u32,
    value: u64,
    width: u32,
) -> AcpiStatus {
    let pci_id = AcpiPciId::from_ffi(pci_id);

    let mut interface = OS_INTERFACE.lock();
    let interface = interface.as_mut().unwrap();

    let reg = reg.try_into().unwrap();

    #[allow(clippy::cast_possible_truncation)] // The ACPICA reference says to ignore the high bits
    // SAFETY: This is `AcpiOsWritePciConfiguration`,
    // and the write was instructed by ACPI so it should be sound
    let r = unsafe {
        match width {
            8 => interface.write_pci_config_u8(pci_id, reg, value as _),
            16 => interface.write_pci_config_u16(pci_id, reg, value as _),
            32 => interface.write_pci_config_u32(pci_id, reg, value as _),
            64 => interface.write_pci_config_u64(pci_id, reg, value),
            _ => panic!("Invalid value of `width`"),
        }
    };

    r.to_acpi_status()
}
