use alloc::ffi::CString;

use crate::{
    bindings::types::{
        tables::FfiAcpiTableHeader, FfiAcpiPhysicalAddress, FfiAcpiPredefinedNames, FfiAcpiString,
    },
    interface::{DropOnTerminate, OS_INTERFACE},
    status::{AcpiError, AcpiErrorAsStatusExt, AcpiStatus},
    types::{tables::AcpiTableHeader, AcpiPredefinedNames},
};

#[export_name = "AcpiOsPredefinedOverride"]
extern "C" fn acpi_os_predefined_override(
    init_val: *const FfiAcpiPredefinedNames,
    new_val_ptr: *mut FfiAcpiString,
) -> AcpiStatus {
    if init_val.is_null() || new_val_ptr.is_null() {
        return AcpiError::BadParameter.to_acpi_status();
    }

    // SAFETY: `init_val` is valid for reads
    let init_val = unsafe { &*init_val };
    let mut lock = OS_INTERFACE.lock();
    let lock = lock.as_mut().unwrap();

    // SAFETY: This is `AcpiOsPredefinedOverride`
    let result = unsafe { lock.predefined_override(&AcpiPredefinedNames::from_ffi(init_val)) };
    let new_val = match result {
        Ok(new_val) => new_val,
        Err(e) => return e.to_acpi_status(),
    };

    // SAFETY: `new_val_ptr` is valid for writes
    unsafe {
        core::ptr::write_unaligned(
            new_val_ptr,
            new_val.map_or(core::ptr::null_mut(), |s| {
                let s = CString::new(s).unwrap();
                let ptr = s.as_ptr().cast_mut();
                lock.objects_to_drop.push(DropOnTerminate::CString(s));
                ptr
            }),
        );
    }

    AcpiStatus::OK
}

#[export_name = "AcpiOsTableOverride"]
extern "C" fn acpi_os_table_override(
    existing_table: *const FfiAcpiTableHeader,
    new_table_ptr: *mut *const FfiAcpiTableHeader,
) -> AcpiStatus {
    if existing_table.is_null() || new_table_ptr.is_null() {
        return AcpiError::BadParameter.to_acpi_status();
    }

    // SAFETY: `existing_table` is valid for reads
    let existing_table = unsafe { &*existing_table };
    let mut lock = OS_INTERFACE.lock();
    let lock = lock.as_mut().unwrap();

    // SAFETY: This is `AcpiOsTableOverride`
    let result = unsafe { lock.table_override(&AcpiTableHeader::from_ffi(existing_table)) };
    let new_table = match result {
        Ok(new_table) => new_table,
        Err(e) => return e.to_acpi_status(),
    };

    // SAFETY: `new_table_ptr` is valid for writes
    unsafe {
        core::ptr::write_unaligned(
            new_table_ptr,
            new_table.map_or(core::ptr::null(), |new_table| {
                new_table.as_ffi() as *const _
            }),
        );
    }

    AcpiStatus::OK
}

#[export_name = "AcpiOsPhysicalTableOverride"]
extern "C" fn acpi_os_physical_table_override(
    existing_table: *mut FfiAcpiTableHeader,
    new_table_address_ptr: *mut FfiAcpiPhysicalAddress,
    new_table_length_ptr: *mut u32,
) -> AcpiStatus {
    if existing_table.is_null() || new_table_address_ptr.is_null() || new_table_length_ptr.is_null()
    {
        return AcpiError::BadParameter.to_acpi_status();
    }

    // SAFETY: `existing_table` is valid for reads
    let existing_table = unsafe { &mut *existing_table };
    let mut lock = OS_INTERFACE.lock();
    let lock = lock.as_mut().unwrap();

    let result =
    // SAFETY: This is `AcpiOsPhysicalTableOverride`
        unsafe { lock.physical_table_override(&AcpiTableHeader::from_ffi(existing_table)) };

    let new_table = match result {
        Ok(new_table) => new_table,
        Err(e) => return e.to_acpi_status(),
    };

    match new_table {
        // SAFETY: `new_table_address_ptr` and `new_table_length_ptr` are valid for writes
        Some((address, length)) => unsafe {
            core::ptr::write(new_table_address_ptr, address.0);
            core::ptr::write(new_table_length_ptr, length);
        },
        // Write null to `new_table_address_ptr` to indicate the table should not be updated
        // SAFETY: `new_table_address_ptr` is valid for writes
        None => unsafe {
            core::ptr::write(new_table_address_ptr, 0);
        },
    }

    AcpiStatus::OK
}
