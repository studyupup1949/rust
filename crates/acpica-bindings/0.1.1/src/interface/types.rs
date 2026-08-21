//! Contains rust equivalents of types used by ACPICA

mod generic_address;
pub mod object;
pub mod tables;

use core::{
    ffi::{c_void, CStr},
    fmt::{Debug, Display},
};

use crate::bindings::types::{
    functions::{FfiAcpiOsdExecCallback, FfiAcpiOsdHandler},
    FfiAcpiCpuFlags, FfiAcpiPciId, FfiAcpiPredefinedNames,
};

pub use generic_address::*;

use self::object::AcpiObject;

/// A physical address into main memory
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct AcpiPhysicalAddress(pub usize);

impl AcpiPhysicalAddress {
    /// A null pointer, i.e. a pointer of value 0
    pub const NULL: Self = Self(0);
}

impl Debug for AcpiPhysicalAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("AcpiPhysicalAddress")
            .field(&format_args!("{:#x}", self.0))
            .finish()
    }
}

#[cfg(feature = "x86_64")]
impl From<AcpiPhysicalAddress> for x86_64::PhysAddr {
    fn from(val: AcpiPhysicalAddress) -> Self {
        x86_64::PhysAddr::new(val.0.try_into().unwrap())
    }
}

/// A default value in an ACPI namespace.
///
/// This struct is passed to the [`predefined_override`] method to allow the OS to change the values of default objects.
///
/// [`predefined_override`]: crate::interface::handler::AcpiHandler::predefined_override
#[derive(Debug)]
pub struct AcpiPredefinedNames<'a>(&'a FfiAcpiPredefinedNames);

impl<'a> AcpiPredefinedNames<'a> {
    pub(crate) fn from_ffi(ffi_predefined_names: &'a FfiAcpiPredefinedNames) -> Self {
        Self(ffi_predefined_names)
    }

    pub(crate) fn as_ffi(&self) -> &'a FfiAcpiPredefinedNames {
        self.0
    }

    /// Gets the name of the object in the namespace
    #[must_use]
    #[allow(clippy::missing_panics_doc)] // ACPI names are ASCII, so this should never panic
    pub fn name(&self) -> &str {
        // SAFETY: The `name` pointer in an FfiAcpiPredefinedNames struct is a null terminated string
        unsafe {
            CStr::from_ptr(self.0.name)
                .to_str()
                .expect("Object name should have been valid utf-8")
        }
    }

    /// Gets the object which will be added to the namespace
    #[must_use]
    pub fn object(&self) -> AcpiObject {
        // SAFETY: The values were passed by ACPICA, so they are valid.
        unsafe { AcpiObject::from_type_and_val(self.0.object_type, self.0.val) }
    }
}

/// The address of an I/O port
#[derive(Debug, Clone, Copy)]
pub struct AcpiIoAddress(pub usize);

/// A tag identifying an interrupt callback so that it can be removed.
/// The OS will receive the tag from [`install_interrupt_handler`] and it must be compared against each handler using [`is_tag`]
///
/// [`install_interrupt_handler`]: super::handler::AcpiHandler::install_interrupt_handler
/// [`is_tag`]: AcpiInterruptCallback::is_tag
#[derive(Debug)]
pub struct AcpiInterruptCallbackTag(pub(crate) FfiAcpiOsdHandler);

/// A callback to notify ACPICA about an interrupt.
///
/// The [`call`] method should be used to run the callback from the associated interrupt handler.
///
/// [`call`]: AcpiInterruptCallback::call
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AcpiInterruptCallback {
    pub(crate) function: FfiAcpiOsdHandler,
    pub(crate) context: *mut c_void,
}

// SAFETY: The `context` pointer in an `AcpiInterruptCallback` points into ACPICA's memory - either
// in a static or on the kernel heap. Both of these need to be available to all kernel threads for
// ACPICA to function at all, so the OS needs to make this sound for the library to function.
unsafe impl Send for AcpiInterruptCallback {}

/// Whether an interrupt is handled or not.
///
/// TODO: What should the OS use this for?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcpiInterruptHandledStatus {
    /// The interrupt has been handled
    Handled,
    /// The interrupt has not been handled
    NotHandled,
}

impl AcpiInterruptCallback {
    /// Calls the callback
    ///
    /// # Safety
    /// This method may only be called from the interrupt handler this callback is for.
    /// An interrupt vector will have been provided along with this object, and this method should only be called from the
    /// interrupt handler for that interrupt vector.
    pub unsafe fn call(&mut self) -> AcpiInterruptHandledStatus {
        // SAFETY:
        let call_result = unsafe { (self.function)(self.context) };
        match call_result {
            0 => AcpiInterruptHandledStatus::Handled,
            1 => AcpiInterruptHandledStatus::NotHandled,
            _ => unreachable!("Acpi callback returned an invalid value"),
        }
    }

    /// Checks whether this callback matches the given `tag`
    #[must_use]
    pub fn is_tag(&self, tag: &AcpiInterruptCallbackTag) -> bool {
        self.function == tag.0
    }
}

/// A callback to run in a new thread.
///
/// The [`call`] method should be used to run the callback from the created thread.
///
/// [`call`]: AcpiThreadCallback::call
#[derive(Debug)]
pub struct AcpiThreadCallback {
    pub(crate) function: FfiAcpiOsdExecCallback,
    pub(crate) context: *mut c_void,
}

impl AcpiThreadCallback {
    /// Calls the callback
    ///
    /// # Safety
    /// This method must be called from a new kernel thread.
    pub unsafe fn call(&mut self) {
        // SAFETY: This is
        unsafe { (self.function)(self.context) };
    }
}

/// A PCI device, identified by its `segment:bus:device:function` address.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct AcpiPciId {
    /// The PCI segment which the device is accessible through
    pub segment: u16,
    /// The PCI bus number
    pub bus: u16,
    /// The device number on the PCI bus
    pub device: u16,
    /// The function number of the device
    pub function: u16,
}

impl AcpiPciId {
    pub(crate) fn from_ffi(v: FfiAcpiPciId) -> Self {
        Self {
            segment: v.segment,
            bus: v.bus,
            device: v.device,
            function: v.function,
        }
    }
}

/// An error which can occur during allocation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcpiAllocationError {
    /// The system has run out of dynamic memory
    OutOfMemory,
}

impl Display for AcpiAllocationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::OutOfMemory => f.write_str("Out of memory"),
        }
    }
}

/// An error which can occur when mapping physical memory
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcpiMappingError {
    /// The system has run out of frames of memory to map
    OutOfMemory,
}

impl Display for AcpiMappingError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::OutOfMemory => f.write_str("Out of memory"),
        }
    }
}

/// CPU flags to be preserved after releasing a lock.
///
/// The OS passes this type to ACPICA when acquiring a lock, and they are returned when releasing the lock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AcpiCpuFlags(pub FfiAcpiCpuFlags);
