use core::ops::{Deref, DerefMut};

use alloc::{boxed::Box, ffi::CString, vec::Vec};
use spin::Mutex;

use crate::bindings::{
    consts::ACPI_FULL_INITIALIZATION,
    functions::{
        AcpiEnableSubsystem, AcpiInitializeObjects, AcpiInitializeSubsystem, AcpiInitializeTables,
        AcpiLoadTables,
    },
};

use self::{handler::AcpiHandler, status::AcpiError};

pub mod handler;

pub mod devices;
pub mod status;
mod tables;
pub mod types;

static OS_INTERFACE: Mutex<Option<OsInterface>> = Mutex::new(None);

#[derive(Debug)]
enum DropOnTerminate {
    CString(CString),
}

struct OsInterface {
    handler: Box<dyn AcpiHandler + Send>,
    objects_to_drop: Vec<DropOnTerminate>,
}

impl Deref for OsInterface {
    type Target = Box<dyn AcpiHandler + Send>;

    fn deref(&self) -> &Self::Target {
        &self.handler
    }
}

impl DerefMut for OsInterface {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.handler
    }
}

/// Registers `interface` as the handler for ACPICA functions, and starts the initialization of ACPICA.
/// See the docs for [`AcpicaOperation`] for more info.
///
/// # Panics
/// If called more than once.
pub fn register_interface<T: AcpiHandler + Send + 'static>(
    interface: T,
) -> Result<AcpicaOperation<false, false, false, false>, AcpiError> {
    let mut lock = OS_INTERFACE.lock();

    assert!(!lock.is_some(), "Interface is already initialized");

    *lock = Some(OsInterface {
        handler: Box::new(interface),
        objects_to_drop: Vec::new(),
    });

    // AcpiInitializeSubsystem calls functions which need this lock
    drop(lock);

    // SAFETY: Handlers for AcpiOs functions have been set up
    unsafe { AcpiInitializeSubsystem().as_result()? };

    Ok(AcpicaOperation)
}

/// The interface to ACPICA functions. The state of ACPICA's initialization is tracked using const generics on this type.
///
/// ACPICA is not initialized all at once - it has multiple initialization functions for different systems.
/// This struct keeps track of the calling of these functions using the type parameters `T`, for whether ACPICA tables are initialized,
/// and `S` for whether the ACPICA subsystem is enabled. An `AcpiOperation<false, false>` can be obtained from [`register_interface`],
/// which also calls `AcpiInitializeSubsystem`.
///
/// Subsystem initialization code could look like the following:
///
/// ```ignore
/// # use acpica_bindings::status::AcpiError;
/// # use acpica_bindings::handler::register_interface;
/// # fn main() -> Result<(), AcpiError> {
///     let interface = todo!(); // In real code this would be an object implementing the AcpiHandler trait
///
///     let initialization = register_interface(interface)?;
///     let initialization = initialization.load_tables()?;
///     let initialization = initialization.enable_subsystem()?;
///     let initialization = initialization.initialize_objects()?;
///
/// #   Ok(())
/// # }
/// ```
#[derive(Debug)]
#[must_use]
pub struct AcpicaOperation<
    const TABLES_INITIALIZED: bool,
    const TABLES_LOADED: bool,
    const SUBSYSTEM_ENABLED: bool,
    const OBJECTS_INITIALIZED: bool,
>;

/// An alias to an [`AcpicaOperation`] which is completely initialized, and all methods are available.
pub type AcpicaOperationFullyInitialized = AcpicaOperation<true, true, true, true>;

impl AcpicaOperation<false, false, false, false> {
    /// Calls the ACPICA function `AcpiInitializeTables`.
    ///
    /// This function causes ACPICA to parse all the tables pointed to by the RSDT/XSDT
    pub fn initialize_tables(
        self,
    ) -> Result<AcpicaOperation<true, false, false, false>, AcpiError> {
        // SAFETY: `AcpiInitializeSubsystem` has been called
        unsafe { AcpiInitializeTables(core::ptr::null_mut(), 16, false).as_result()? };

        Ok(AcpicaOperation)
    }
}

impl AcpicaOperation<true, false, false, false> {
    /// Calls the ACPICA function `AcpiLoadTables`.
    ///
    /// This function causes ACPICA to parse and execute AML code in order to build the AML namespace.
    pub fn load_tables(self) -> Result<AcpicaOperation<true, true, false, false>, AcpiError> {
        // SAFETY: `AcpiInitializeTables` has been called
        unsafe { AcpiLoadTables().as_result()? };

        Ok(AcpicaOperation)
    }
}

impl AcpicaOperation<true, true, false, false> {
    /// Calls the ACPICA function `AcpiEnableSubsystem`.
    ///
    /// This function causes ACPICA to enter ACPI mode and start receiving ACPI interrupts.
    pub fn enable_subsystem(self) -> Result<AcpicaOperation<true, true, true, false>, AcpiError> {
        // SAFETY: `AcpiLoadTables` has been called
        unsafe { AcpiEnableSubsystem(ACPI_FULL_INITIALIZATION).as_result()? };

        Ok(AcpicaOperation)
    }
}

impl AcpicaOperation<true, true, true, false> {
    /// Calls the ACPICA function `AcpiEnableSubsystem`.
    ///
    /// This function causes ACPICA to enter ACPI mode and start receiving ACPI interrupts.
    pub fn initialize_objects(self) -> Result<AcpicaOperationFullyInitialized, AcpiError> {
        // SAFETY: `AcpiEnableSubsystem` has been called
        unsafe { AcpiInitializeObjects(ACPI_FULL_INITIALIZATION).as_result()? };

        Ok(AcpicaOperation)
    }
}
