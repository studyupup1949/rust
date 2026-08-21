use core::ffi::c_void;

use crate::{
    bindings::types::{FfiAcpiPhysicalAddress, FfiAcpiSize},
    interface::OS_INTERFACE,
    status::{AcpiError, AcpiErrorAsStatusExt, AcpiStatus},
    types::AcpiPhysicalAddress,
};

#[export_name = "AcpiOsMapMemory"]
extern "C" fn acpi_os_map_memory(
    address: FfiAcpiPhysicalAddress,
    length: FfiAcpiSize,
) -> *mut c_void {
    let mut interface = OS_INTERFACE.lock();
    let interface = interface.as_mut().unwrap();

    // SAFETY: This is `AcpiOsMapMemory`
    unsafe {
        interface
            .map_memory(AcpiPhysicalAddress(address), length)
            .map(<*mut u8>::cast)
            .unwrap_or(core::ptr::null_mut())
    }
}

#[export_name = "AcpiOsUnmapMemory"]
extern "C" fn acpi_os_unmap_memory(logical_address: *mut c_void, size: FfiAcpiSize) {
    let mut interface = OS_INTERFACE.lock();
    let interface = interface.as_mut().unwrap();

    // SAFETY: This is `AcpiOsUnmapMemory`, `logical_address` was provided by ACPICA and produced by acpi_os_map_memory
    unsafe { interface.unmap_memory(logical_address.cast(), size) }
}

#[export_name = "AcpiOsGetPhysicalAddress"]
extern "C" fn acpi_os_get_physical_address(
    logical_address: *mut c_void,
    physical_address: *mut FfiAcpiPhysicalAddress,
) -> AcpiStatus {
    if logical_address.is_null() || physical_address.is_null() {
        return AcpiError::BadParameter.to_acpi_status();
    }

    let mut interface = OS_INTERFACE.lock();
    let interface = interface.as_mut().unwrap();

    let result = match interface.get_physical_address(logical_address.cast()) {
        Ok(Some(r)) => r,
        Ok(None) => return AcpiError::Error.to_acpi_status(),
        Err(e) => return e.to_acpi_status(),
    };

    // SAFETY: `physical_address` is a valid pointer
    unsafe { *physical_address = result.0 }

    AcpiStatus::OK
}

#[export_name = "AcpiOsReadMemory"]
extern "C" fn acpi_os_read_memory(
    address: FfiAcpiPhysicalAddress,
    value: *mut u64,
    width: u32,
) -> AcpiStatus {
    if address == 0 || value.is_null() {
        return AcpiError::BadParameter.to_acpi_status();
    }

    let mut interface = OS_INTERFACE.lock();
    let interface = interface.as_mut().unwrap();

    let address = AcpiPhysicalAddress(address);

    // SAFETY: The given parameter is non-null so it is valid for reads,
    // and this is `AcpiOsReadMemory`
    let r = unsafe {
        match width {
            8 => interface.read_physical_u8(address).map(u64::from),
            16 => interface.read_physical_u16(address).map(u64::from),
            32 => interface.read_physical_u32(address).map(u64::from),
            64 => interface.read_physical_u64(address),
            _ => panic!("Invalid value of `width`"),
        }
    };

    let result = match r {
        Ok(v) => v,
        Err(e) => return e.to_acpi_status(),
    };

    // SAFETY: `value` is a valid pointer
    unsafe {
        *value = result;
    }

    AcpiStatus::OK
}

#[export_name = "AcpiOsWriteMemory"]
extern "C" fn acpi_os_write_memory(
    address: FfiAcpiPhysicalAddress,
    value: u64,
    width: u32,
) -> AcpiStatus {
    if address == 0 {
        return AcpiError::BadParameter.to_acpi_status();
    }

    let mut interface = OS_INTERFACE.lock();
    let interface = interface.as_mut().unwrap();

    let address = AcpiPhysicalAddress(address);

    #[allow(clippy::cast_possible_truncation)] // The ACPICA reference says to ignore the high bits
    // SAFETY: The given parameter is non-null so it is valid for reads,
    // and this is `AcpiOsWriteMemory`
    let r = unsafe {
        match width {
            8 => interface.write_physical_u8(address, value as _),
            16 => interface.write_physical_u16(address, value as _),
            32 => interface.write_physical_u32(address, value as _),
            64 => interface.write_physical_u64(address, value),
            _ => panic!("Invalid value of `width`"),
        }
    };

    r.to_acpi_status()
}

#[export_name = "AcpiOsReadable"]
extern "C" fn acpi_os_readable(pointer: *mut c_void, length: FfiAcpiSize) -> bool {
    let mut interface = OS_INTERFACE.lock();
    let interface = interface.as_mut().unwrap();

    // SAFETY: This is `AcpiOsReadable`
    unsafe { interface.readable(pointer, length) }
}

#[export_name = "AcpiOsWritable"]
extern "C" fn acpi_os_writable(pointer: *mut c_void, length: FfiAcpiSize) -> bool {
    let mut interface = OS_INTERFACE.lock();
    let interface = interface.as_mut().unwrap();

    // SAFETY: This is `AcpiOsReadable`
    unsafe { interface.writable(pointer, length) }
}
