mod alloc;
mod interrupts;
mod io;
#[cfg(feature = "builtin_lock")]
mod lock;
mod memory;
mod overrides;
mod pci;
mod printf;
#[cfg(feature = "builtin_semaphore")]
mod semaphore;
mod threading;

use ::core::ffi::c_void;
use core::ffi::CStr;

use log::{error, trace};

use crate::{
    bindings::{
        consts::{ACPI_SIGNAL_BREAKPOINT, ACPI_SIGNAL_FATAL},
        types::{FfiAcpiPhysicalAddress, FfiAcpiSignalFatalInfo},
    },
    interface::{
        status::{AcpiErrorAsStatusExt, AcpiStatus},
        OS_INTERFACE,
    },
    status::AcpiError,
};

#[export_name = "AcpiOsInitialize"]
extern "C" fn acpi_os_initialize() -> AcpiStatus {
    let mut interface = OS_INTERFACE.lock();
    let interface = interface.as_mut().unwrap();
    // SAFETY: This is `AcpiOsInitialize`
    unsafe { interface.initialize().to_acpi_status() }
}

#[export_name = "AcpiOsTerminate"]
extern "C" fn acpi_os_terminate() -> AcpiStatus {
    let mut interface = OS_INTERFACE.lock();

    // SAFETY: This is `AcpiOsTerminate`.
    // After this method call the struct is dropped.
    let s = unsafe { interface.as_mut().unwrap().terminate().to_acpi_status() };

    *interface = None;

    s
}

#[export_name = "AcpiOsGetRootPointer"]
extern "C" fn acpi_os_get_root_pointer() -> FfiAcpiPhysicalAddress {
    let mut interface = OS_INTERFACE.lock();
    let interface = interface.as_mut().unwrap();

    interface.get_root_pointer().0
}

#[export_name = "AcpiOsStall"]
extern "C" fn acpi_os_stall(microseconds: u32) {
    let mut interface = OS_INTERFACE.lock();
    let interface = interface.as_mut().unwrap();

    // SAFETY: This is `AcpiOsStall`
    unsafe { interface.stall(microseconds.try_into().unwrap()) }
}

#[export_name = "AcpiOsGetTimer"]
extern "C" fn acpi_os_get_timer() -> u64 {
    let mut interface = OS_INTERFACE.lock();
    let interface = interface.as_mut().unwrap();

    // SAFETY: This is `AcpiOsGetTimer`
    unsafe { interface.get_timer() }
}

#[export_name = "AcpiOsSignal"]
extern "C" fn acpi_os_signal(function: u32, info: *mut c_void) -> AcpiStatus {
    let mut interface = OS_INTERFACE.lock();
    let interface = interface.as_mut().unwrap();

    match function {
        _f @ ACPI_SIGNAL_FATAL => {
            // SAFETY: When function == ACPI_SIGNAL_FATAL, `info` points to a value of type AcpiFatalInfo.
            let info: FfiAcpiSignalFatalInfo = unsafe { core::ptr::read(info.cast()) };

            // SAFETY: This is `AcpiOsSignal`
            unsafe {
                interface
                    .signal_fatal(info.resource_type, info.code, info.argument)
                    .to_acpi_status()
            }
        }
        _f @ ACPI_SIGNAL_BREAKPOINT => {
            // SAFETY: When function == ACPI_SIGNAL_BREAKPOINT, `info` points to a null-terminated string.
            let info = unsafe { CStr::from_ptr(info.cast()) };
            let info = info
                .to_str()
                .expect("Info string should have been valid utf-8");

            // SAFETY: This is `AcpiOsSignal`
            unsafe { interface.signal_breakpoint(info).to_acpi_status() }
        }
        _ => {
            error!(target: "acpi_os_signal", "Invalid or unknown value of `function`");
            AcpiError::BadParameter.to_acpi_status()
        }
    }
}

#[export_name = "AcpiOsEnterSleep"]
extern "C" fn acpi_os_enter_sleep(sleep_state: u8, reg_a: u32, reg_b: u32) -> AcpiStatus {
    trace!(target: "acpi_os_enter_sleep", "Entering sleep state {sleep_state}. RegA: {reg_a:#x}, RegB: {reg_b:#x}");

    let mut interface = OS_INTERFACE.lock();
    let interface = interface.as_mut().unwrap();

    // SAFETY: This is `AcpiOsEnterSleep`
    unsafe {
        interface
            .enter_sleep(sleep_state, reg_a, reg_b)
            .to_acpi_status()
    }
}

/*


    Trait handlers for optional methods
    These handlers are only used if certain features are disabled.
    If the features are enabled, implementations provided by this crate are used instead of OS-specific implementations.


*/

#[cfg(not(feature = "builtin_cache"))]
#[export_name = "AcpiOsCreateCache"]
extern "C" fn acpi_os_create_cache(
    cache_name: *mut i8,
    size: u16,
    max_depth: u16,
    return_cache: *mut *mut c_void,
) -> AcpiStatus {
    todo!()
}

#[cfg(not(feature = "builtin_cache"))]
#[export_name = "AcpiOsDeleteCache"]
extern "C" fn acpi_os_delete_cache(cache: *mut c_void) -> AcpiStatus {
    todo!()
}

#[cfg(not(feature = "builtin_cache"))]
#[export_name = "AcpiOsPurgeCache"]
extern "C" fn acpi_os_purge_cache(mut cache: *mut c_void) -> AcpiStatus {
    todo!()
}

#[cfg(not(feature = "builtin_cache"))]
#[export_name = "AcpiOsAcquireObject"]
extern "C" fn acpi_os_acquire_object(mut cache: *mut c_void) -> *mut c_void {
    todo!()
}

#[cfg(not(feature = "builtin_cache"))]
#[export_name = "AcpiOsReleaseObject"]
extern "C" fn acpi_os_release_object(mut cache: *mut c_void, object: *mut c_void) -> AcpiStatus {
    todo!()
}

#[cfg(not(feature = "builtin_lock"))]
#[export_name = "AcpiOsCreateLock"]
extern "C" fn acpi_os_create_lock(out_handle: *mut *mut c_void) -> AcpiStatus {
    let mut interface = OS_INTERFACE.lock();
    let interface = interface.as_mut().unwrap();

    let created_lock = unsafe { interface.create_lock() };
    match created_lock {
        Ok(l) => unsafe {
            *out_handle = l;
            AcpiStatus::OK
        },
        Err(e) => e.to_acpi_status(),
    }
}

#[cfg(not(feature = "builtin_lock"))]
#[export_name = "AcpiOsDeleteLock"]
extern "C" fn acpi_os_delete_lock(handle: *mut c_void) {
    let mut interface = OS_INTERFACE.lock();
    let interface = interface.as_mut().unwrap();

    unsafe { interface.delete_lock(handle) }
}

#[cfg(not(feature = "builtin_lock"))]
use crate::{bindings::types::FfiAcpiCpuFlags, types::AcpiCpuFlags};

#[cfg(not(feature = "builtin_lock"))]
#[export_name = "AcpiOsAcquireLock"]
extern "C" fn acpi_os_acquire_lock(handle: *mut c_void) -> FfiAcpiCpuFlags {
    let mut interface = OS_INTERFACE.lock();
    let interface = interface.as_mut().unwrap();

    unsafe { interface.acquire_lock(handle).0 }
}

#[cfg(not(feature = "builtin_lock"))]
#[export_name = "AcpiOsReleaseLock"]
extern "C" fn acpi_os_release_lock(handle: *mut c_void, flags: FfiAcpiCpuFlags) {
    let mut interface = OS_INTERFACE.lock();
    let interface = interface.as_mut().unwrap();

    unsafe { interface.release_lock(handle, AcpiCpuFlags(flags)) }
}

#[cfg(not(feature = "builtin_semaphore"))]
#[export_name = "AcpiOsCreateSemaphore"]
extern "C" fn acpi_os_create_semaphore(
    _max_units: u32,
    _initial_units: u32,
    _out_handle: *mut *mut c_void,
) -> AcpiStatus {
    todo!()
}

#[cfg(not(feature = "builtin_semaphore"))]
#[export_name = "AcpiOsDeleteSemaphore"]
extern "C" fn acpi_os_delete_semaphore(_handle: *mut c_void) -> AcpiStatus {
    todo!()
}

#[cfg(not(feature = "builtin_semaphore"))]
#[export_name = "AcpiOsWaitSemaphore"]
extern "C" fn acpi_os_wait_semaphore(
    _handle: *mut c_void,
    _units: u32,
    _timeout: u16,
) -> AcpiStatus {
    todo!()
}

#[cfg(not(feature = "builtin_semaphore"))]
#[export_name = "AcpiOsSignalSemaphore"]
extern "C" fn acpi_os_signal_semaphore(_handle: *mut c_void, _units: u32) -> AcpiStatus {
    todo!()
}
