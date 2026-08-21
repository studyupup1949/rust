use crate::{
    bindings::types::FfiAcpiIoAddress,
    interface::OS_INTERFACE,
    status::{AcpiError, AcpiErrorAsStatusExt, AcpiStatus},
    types::AcpiIoAddress,
};

#[export_name = "AcpiOsReadPort"]
extern "C" fn acpi_os_read_port(
    address: FfiAcpiIoAddress,
    value: *mut u32,
    width: u32,
) -> AcpiStatus {
    if value.is_null() {
        return AcpiError::BadParameter.to_acpi_status();
    }

    let mut interface = OS_INTERFACE.lock();
    let interface = interface.as_mut().unwrap();

    let address = AcpiIoAddress(address.try_into().unwrap());

    let data = match width {
        // SAFETY: This is `AcpiOsReadPort`
        8 => unsafe { interface.read_port_u8(address).map(u8::into) },
        // SAFETY: This is `AcpiOsReadPort`
        16 => unsafe { interface.read_port_u16(address).map(u16::into) },
        // SAFETY: This is `AcpiOsReadPort`
        32 => unsafe { interface.read_port_u32(address) },

        _ => panic!("Invalid value of `width`"),
    };

    let data = match data {
        Ok(data) => data,
        Err(e) => return e.to_acpi_status(),
    };

    // SAFETY: `value` is a valid pointer
    unsafe { *value = data }

    AcpiStatus::OK
}

#[export_name = "AcpiOsWritePort"]
extern "C" fn acpi_os_write_port(address: FfiAcpiIoAddress, value: u32, width: u32) -> AcpiStatus {
    let mut interface = OS_INTERFACE.lock();
    let interface = interface.as_mut().unwrap();

    let address = AcpiIoAddress(address.try_into().unwrap());

    #[allow(clippy::cast_possible_truncation)] // The ACPICA reference says to ignore the high bits
    let result = match width {
        // SAFETY: This is `AcpiOsReadPort`
        8 => unsafe { interface.write_port_u8(address, value as u8) },
        // SAFETY: This is `AcpiOsReadPort`
        16 => unsafe { interface.write_port_u16(address, value as u16) },
        // SAFETY: This is `AcpiOsReadPort`
        32 => unsafe { interface.write_port_u32(address, value) },

        _ => panic!("Invalid value of `width`"),
    };

    result.to_acpi_status()
}
