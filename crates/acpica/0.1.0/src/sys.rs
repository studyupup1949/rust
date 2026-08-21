#![allow(non_snake_case, dead_code)]

use crate::get_os_layer;
use core::{ffi::c_void, ptr::NonNull};

#[unsafe(no_mangle)]
extern "C" fn AcpiOsInitialize() {
    get_os_layer().initialize();
}

#[unsafe(no_mangle)]
extern "C" fn AcpiOsTerminate() {
    get_os_layer().terminate();
}

#[unsafe(no_mangle)]
extern "C" fn AcpiOsGetRootPointer() -> u64 {
    get_os_layer().get_root_address()
}

#[unsafe(no_mangle)]
extern "C" fn AcpiOsMapMemory(PhysicalAddress: u64, Length: usize) -> *mut c_void {
    get_os_layer()
        .map_memory(PhysicalAddress, Length)
        .map(NonNull::cast)
        .map(NonNull::as_ptr)
        .unwrap_or_default()
}

#[unsafe(no_mangle)]
extern "C" fn AcpiOsUnmapMemory(LogicalAddress: *mut c_void, Length: usize) {
    get_os_layer().unmap_memory(
        NonNull::new(LogicalAddress)
            .expect("ACPICA provided a null pointer for `AcpiOsUnmapMemory`")
            .cast(),
        Length,
    );
}

#[unsafe(no_mangle)]
extern "C" fn AcpiOsAllocate(Size: usize) -> *mut c_void {
    get_os_layer()
        .allocate(Size)
        .map(NonNull::cast)
        .map(NonNull::as_ptr)
        .unwrap_or_default()
}

#[unsafe(no_mangle)]
extern "C" fn AcpiOsFree(Memory: *mut c_void) {
    get_os_layer().deallocate(
        NonNull::new(Memory)
            .expect("ACPICA provided a null pointer for `AcpiOsFree`")
            .cast(),
    );
}

#[unsafe(no_mangle)]
extern "C" fn AcpiOsReadable(Memory: *mut c_void, Length: usize) -> bool {
    get_os_layer().is_memory_readable(
        NonNull::new(Memory)
            .expect("ACPICA provided a null pointer for `AcpiOsReadable`")
            .cast(),
        Length,
    )
}

#[unsafe(no_mangle)]
extern "C" fn AcpiOsWritable(Memory: *mut c_void, Length: usize) -> bool {
    get_os_layer().is_memory_writable(
        NonNull::new(Memory)
            .expect("ACPICA provided a null pointer for `AcpiOsWritable`")
            .cast(),
        Length,
    )
}

#[repr(u32)]
#[allow(clippy::upper_case_acronyms, non_camel_case_types)]
pub enum ExceptionCode {
    OK = 0x0000,
    ERROR = 0x0001,
    NO_ACPI_TABLES = 0x0002,
    NO_NAMESPACE = 0x0003,
    NO_MEMORY = 0x0004,
    NOT_FOUND = 0x0005,
    NOT_EXIST = 0x0006,
}

#[link(name = "acpica_sys")]
unsafe extern "C" {
    pub fn AcpiInitializeSubsystem();

    #[cfg(target_arch = "x86")]
    pub fn AcpiFindRootPointer(table_address: &mut usize) -> ExceptionCode;
}

#[test]
fn test_init() {
    // Safety: `acpica_sys` exports symbol.
    unsafe {
        AcpiInitializeSubsystem();
    }
}
