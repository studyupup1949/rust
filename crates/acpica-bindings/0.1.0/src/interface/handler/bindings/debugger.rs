#[export_name = "AcpiOsInitializeDebugger"]
extern "C" fn acpi_os_initialize_debugger() -> AcpiStatus {
    todo!()
}

#[export_name = "AcpiOsTerminateDebugger"]
extern "C" fn acpi_os_terminate_debugger() {
    todo!()
}

#[export_name = "AcpiOsGetLine"]
extern "C" fn acpi_os_get_line(
    _buffer: *mut i8,
    _buffer_length: u32,
    _bytes_read: *mut u32,
) -> AcpiStatus {
    todo!()
}