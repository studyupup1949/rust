use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::boxed::Box;
use log::trace;

use crate::status::{AcpiError, AcpiErrorAsStatusExt, AcpiStatus};

#[derive(Debug)]
struct AcpiSemaphore {
    max_units: usize,
    units: AtomicUsize,
}

impl AcpiSemaphore {
    fn try_sub(&self, units: usize) -> Result<usize, usize> {
        self.units
            .fetch_update(Ordering::Release, Ordering::Acquire, |v| {
                v.checked_sub(units)
            })
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct AcpiSemaphorePtr(*const AcpiSemaphore);

impl AcpiSemaphorePtr {
    /// Gets the pointer as a reference
    ///
    /// # Safety
    /// * This struct must have been initialized with a valid pointer
    unsafe fn as_ref(&self) -> &AcpiSemaphore {
        // SAFETY: The contained pointer is valid
        unsafe { &*self.0 }
    }
}

#[export_name = "AcpiOsCreateSemaphore"]
extern "C" fn acpi_os_create_semaphore(
    max_units: u32,
    initial_units: u32,
    out_handle: *mut AcpiSemaphorePtr,
) -> AcpiStatus {
    if out_handle.is_null() || initial_units > max_units {
        return AcpiError::BadParameter.to_acpi_status();
    }

    trace!(target: "acpi_os_create_semaphore", "Creating semaphore: max units {max_units:#x}, initial units {initial_units:#x}");

    // TODO: make this fallible
    let semaphore = Box::new(AcpiSemaphore {
        max_units: max_units as _,
        units: AtomicUsize::new(initial_units as _),
    });

    let leak = Box::leak(semaphore) as *const _;

    trace!(target: "acpi_os_create_semaphore", "Semaphore created at {leak:p}");

    // SAFETY: `out_handle` is a valid pointer
    unsafe { *out_handle = AcpiSemaphorePtr(leak) };

    AcpiStatus::OK
}

#[export_name = "AcpiOsDeleteSemaphore"]
extern "C" fn acpi_os_delete_semaphore(handle: AcpiSemaphorePtr) -> AcpiStatus {
    trace!(target: "acpi_os_delete_semaphore", "Deleting semaphore at {:p}", handle.0);

    // TODO: This is not OS-portable, remove
    if (handle.0 as usize) < 0x4000_0000_0000 {
        return AcpiError::BadParameter.to_acpi_status();
    }

    // SAFETY: This function is only called if there are no more references to this semaphore
    // Therefore it is safe to cast the pointer to mutable and construct a box from it.
    let b = unsafe { Box::from_raw(handle.0.cast_mut()) };

    drop(b);

    AcpiStatus::OK
}

#[export_name = "AcpiOsWaitSemaphore"]
extern "C" fn acpi_os_wait_semaphore(
    handle: AcpiSemaphorePtr,
    units: u32,
    timeout: u16,
) -> AcpiStatus {
    trace!(target: "acpi_os_wait_semaphore", "Waiting on semaphore at {:p}: units {units:#x}, timeout {timeout:#x}", handle.0);

    // SAFETY: The `handle` pointer was passed to ACPICA by `acpi_os_create_semaphore`, so it's a valid pointer
    let handle = unsafe { handle.as_ref() };

    match timeout {
        0 => match handle.try_sub(units as usize) {
            Ok(_) => AcpiStatus::OK,
            Err(_) => AcpiError::Time.to_acpi_status(),
        },
        0xFFFF => loop {
            if handle.try_sub(units as usize).is_ok() {
                break AcpiStatus::OK;
            }
        },
        _ => todo!(),
    }
}

#[export_name = "AcpiOsSignalSemaphore"]
extern "C" fn acpi_os_signal_semaphore(handle: AcpiSemaphorePtr, units: u32) -> AcpiStatus {
    trace!(target: "acpi_os_signal_semaphore", "Signalling semaphore at {:p} for {units:#x} units", handle.0);

    // SAFETY: The `handle` pointer was passed to ACPICA by `acpi_os_create_lock`, so it's a valid pointer
    let handle = unsafe { handle.as_ref() };

    let result = handle
        .units
        .fetch_update(Ordering::Release, Ordering::Acquire, |v| {
            let new_units = v + units as usize;

            if new_units > handle.max_units {
                None
            } else {
                Some(new_units)
            }
        });

    match result {
        Ok(_) => AcpiStatus::OK,
        Err(_) => AcpiError::Limit.to_acpi_status(),
    }
}
