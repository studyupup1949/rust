use core::ffi::c_void;

use crate::{
    bindings::types::functions::FfiAcpiOsdHandler,
    interface::OS_INTERFACE,
    status::{AcpiErrorAsStatusExt, AcpiStatus},
    types::{AcpiInterruptCallback, AcpiInterruptCallbackTag},
};

#[export_name = "AcpiOsInstallInterruptHandler"]
extern "C" fn acpi_os_install_interrupt_handler(
    interrupt_number: u32,
    service_routine: FfiAcpiOsdHandler,
    context: *mut c_void,
) -> AcpiStatus {
    let mut interface = OS_INTERFACE.lock();
    let interface = interface.as_mut().unwrap();

    let callback = AcpiInterruptCallback {
        function: service_routine,
        context,
    };

    // SAFETY: This is `AcpiOsInstallInterruptHandler`
    unsafe {
        interface
            .install_interrupt_handler(interrupt_number, callback)
            .to_acpi_status()
    }
}

#[export_name = "AcpiOsRemoveInterruptHandler"]
extern "C" fn acpi_os_remove_interrupt_handler(
    interrupt_number: u32,
    service_routine: FfiAcpiOsdHandler,
) -> AcpiStatus {
    let mut interface = OS_INTERFACE.lock();
    let interface = interface.as_mut().unwrap();

    // SAFETY: This is `AcpiOsRemoveInterruptHandler`
    unsafe {
        interface
            .remove_interrupt_handler(interrupt_number, AcpiInterruptCallbackTag(service_routine))
            .to_acpi_status()
    }
}
