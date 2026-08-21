#![no_std]
#![feature(allocator_api)]
#![forbid(stable_features, unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::debug_assert_with_mut_call,
    clippy::float_arithmetic,
    clippy::as_conversions
)]
#![warn(
    clippy::cargo,
    clippy::pedantic,
    clippy::undocumented_unsafe_blocks,
    clippy::semicolon_inside_block,
    clippy::semicolon_if_nothing_returned
)]
#![allow(
    dead_code,
    clippy::missing_const_for_fn,
    clippy::needless_for_each,
    clippy::if_not_else
)]

mod sys;

use core::ptr::NonNull;

pub type PhysicalAddress = u64;
pub type LogicalAddress = NonNull<u8>;

/// Represents implementation of the OS-specific features that are required for use of the ACPICA
/// subsystem. Each function definition is an internal implementation of a function that is then
/// exported and used by the statically-linked ACPICA library.
///
/// Use [`crate::install`] to install your custom [`crate::AcpiHandler`] for use with the ACPICA
/// subsystem.
pub trait AcpiHandler: Send + Sync {
    /// Allows the OS to initialize itself. It is called during initialization of the ACPICA subsystem.
    fn initialize(&self);

    /// Allows the OS to cleanup and terminate. It is called during termination of the ACPICA subsystem.
    fn terminate(&self);

    /// Returns the physical address of the ACPI RSDP (Root System Description Pointer) table.
    /// The mechanism used to obtain this pointer is platform and/or OS dependent.
    fn get_root_address(&self) -> PhysicalAddress;

    /// Maps a physical address into the caller’s address space. The logical address is returned.
    fn map_memory(&self, physical_address: PhysicalAddress, size: usize) -> Option<LogicalAddress>;

    /// Deletes a mapping that was created by [`AcpiHandler::map_memory`].
    fn unmap_memory(&self, logical_address: LogicalAddress, size: usize);

    /// Translates a logical address to its physical address location.
    fn get_physical_address(&self, logical_address: LogicalAddress) -> PhysicalAddress;

    /// Dynamically allocates memory that is not assumed to be initialized.
    fn allocate(&self, size: usize) -> Option<LogicalAddress>;

    /// Frees memory that was previously allocated via [`AcpiHandler::allocate`].
    fn deallocate(&self, logical_address: LogicalAddress);

    /// Validates that a pointer to a memory region is valid and the entire region is readable.
    /// Used to validate input parameters to the ACPICA subsystem.
    fn is_memory_readable(&self, logical_address: LogicalAddress, length: usize) -> bool;

    /// This function validates that a pointer to a memory region is valid and the entire region
    /// is both writable and readable. Used to validate input parameters to the ACPICA subsystem.
    fn is_memory_writable(&self, logical_address: LogicalAddress, length: usize) -> bool;
}

pub static OS_LAYER: spin::Once<&'static dyn AcpiHandler> = spin::Once::new();

pub fn install(handler: &'static impl AcpiHandler) {
    debug_assert!(
        !OS_LAYER.is_completed(),
        "`acpica_sys::install` has been called more than once; this is likely an error"
    );

    OS_LAYER.call_once(|| handler);
}

fn get_os_layer() -> &'static dyn AcpiHandler {
    *OS_LAYER
        .get()
        .expect("ACPICA OS layer has not been installed")
}

/// This function locates and returns the ACPI Root System Description Pointer by scanning
/// within the first megabyte of physical memory for the RSDP signature. This mechanism is
/// only applicable to IA-32 systems.
///
/// This interface should only be called from [`AcpiHandler::get_root_address`] if this
/// memory scanning mechanism is appropropriate for the current platform.
#[cfg(target_arch = "x86")]
pub fn find_root_pointer() -> PhysicalAddress {
    let mut address: usize;
    let exception_code = unsafe { sys::AcpiFindRootPointer(&mut address) };

    match exception_code {
        sys::ExceptionCode::OK => address,
        sys::ExceptionCode::NO_MEMORY => panic!("insufficient dynamic memory"),
        sys::ExceptionCode::NOT_FOUND => panic!("valid RSDP could not be located"),
    }

    // TODO use `Result<PhysicalAddress, Error>` as return type
}
