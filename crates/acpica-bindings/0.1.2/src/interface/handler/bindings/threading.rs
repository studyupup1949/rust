use core::ffi::c_void;

use crate::{
    bindings::types::{functions::FfiAcpiOsdExecCallback, FfiAcpiExecuteType},
    interface::OS_INTERFACE,
    status::{AcpiErrorAsStatusExt, AcpiStatus},
    types::AcpiThreadCallback,
};

#[export_name = "AcpiOsGetThreadId"]
extern "C" fn acpi_os_get_thread_id() -> u64 {
    let mut interface = OS_INTERFACE.lock();
    let interface = interface.as_mut().unwrap();

    interface.get_thread_id()
}

#[export_name = "AcpiOsExecute"]
extern "C" fn acpi_os_execute(
    _type: FfiAcpiExecuteType,
    function: FfiAcpiOsdExecCallback,
    context: *mut c_void,
) -> AcpiStatus {
    let mut interface = OS_INTERFACE.lock();
    let interface = interface.as_mut().unwrap();

    let callback = AcpiThreadCallback { function, context };

    // SAFETY: This is `AcpiOsExecute`
    unsafe { interface.execute(callback).to_acpi_status() }
}

#[export_name = "AcpiOsWaitEventsComplete"]
extern "C" fn acpi_os_wait_events_complete() {
    let mut interface = OS_INTERFACE.lock();
    let interface = interface.as_mut().unwrap();

    // SAFETY: This is `AcpiOsWaitEventsComplete`
    unsafe { interface.wait_for_events() }
}

#[export_name = "AcpiOsSleep"]
extern "C" fn acpi_os_sleep(milliseconds: u64) {
    let mut interface = OS_INTERFACE.lock();
    let interface = interface.as_mut().unwrap();

    // SAFETY: This is `AcpiOsSleep`
    unsafe { interface.sleep(milliseconds.try_into().unwrap()) }
}
