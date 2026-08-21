//! The [`AcpiObject`] type

use core::ffi::CStr;

use log::{debug, warn};

use crate::bindings::{
    consts::{
        ACPI_TYPE_ANY, ACPI_TYPE_BUFFER, ACPI_TYPE_INTEGER, ACPI_TYPE_LOCAL_REFERENCE,
        ACPI_TYPE_PACKAGE, ACPI_TYPE_POWER, ACPI_TYPE_PROCESSOR, ACPI_TYPE_STRING,
    },
    types::{
        object::{FfiAcpiObject, FfiAcpiObjectType},
        FfiAcpiHandle, FfiAcpiIoAddress,
    },
};

/// A package [`AcpiObject`]
#[derive(Debug)]
pub struct AcpiObjectPackage {
    count: u32,
    elements: *mut FfiAcpiObject,
}

/// A string [`AcpiObject`]
#[derive(Debug)]
pub struct AcpiObjectString {
    length: u32,
    pointer: *mut u8,
}

/// A buffer [`AcpiObject`]
#[derive(Debug)]
pub struct AcpiObjectBuffer {
    length: u32,
    pointer: *mut u8,
}

/// A reference to another [`AcpiObject`]
#[derive(Debug)]
pub struct AcpiObjectReference {
    actual_type: FfiAcpiObjectType,
    handle: FfiAcpiHandle,
}

/// An [`AcpiObject`] describing the features of a processor
#[derive(Debug)]
pub struct AcpiObjectProcessor {
    proc_id: u32,
    pblk_address: FfiAcpiIoAddress,
    pblk_length: u32,
}

/// An [`AcpiObject`] describing a power resource
#[derive(Debug)]
pub struct AcpiObjectPowerResource {
    system_level: u32,
    resource_order: u32,
}

/// An object used in the processing of ACPI data, mostly AML execution.
#[derive(Debug)]
pub enum AcpiObject<'a> {
    /// The object can be any type, or the type is not known.
    /// From ACPICA comments: "\[Any\] is used to indicate a NULL package element or an unresolved named reference."
    Any,
    /// The object is an integer
    Integer(u64),
    /// The object is a string
    String(&'a str),
    /// The object is a buffer of bytes
    Buffer(&'a [u8]),
    /// The object is a package containing other AML data
    Package(AcpiObjectPackage),
    /// The object is a reference to another [`AcpiObject`]
    Reference(AcpiObjectReference),
    /// The object describes the features of a processor
    Processor(AcpiObjectProcessor),
    /// The object describes a power resource
    PowerResource(AcpiObjectPowerResource),
}

impl<'a> AcpiObject<'a> {
    /// Copies the data from an APCICA object into a safe representation.
    ///
    /// # Safety
    /// `pointer` must point to a valid `ACPI_OBJECT` struct.
    pub(crate) unsafe fn from_ffi(pointer: *const FfiAcpiObject) -> Self {
        // SAFETY: `pointer` is a valid acpi object
        let ffi_acpi_object = unsafe { *pointer };
        // SAFETY: Every variant of FfiAcpiObject has this field in this location,
        // so it can be accessed no matter what the real variant is.
        let object_type = unsafe { ffi_acpi_object.object_type };

        match object_type {
            _t @ ACPI_TYPE_ANY => Self::Any,
            // SAFETY: If the object is an integer then the `integer` field can be read
            _t @ ACPI_TYPE_INTEGER => unsafe { Self::Integer(ffi_acpi_object.integer.value) },
            _t @ ACPI_TYPE_STRING => {
                // SAFETY: If the object is a string then the `string` field can be read
                let string = unsafe { ffi_acpi_object.string };

                // SAFETY: The object is valid so the pointer and length are correct
                let bytes = unsafe {
                    core::slice::from_raw_parts(string.pointer.cast(), string.length as _)
                };

                let str = core::str::from_utf8(bytes).unwrap();

                Self::String(str)
            }
            _t @ ACPI_TYPE_BUFFER => {
                // SAFETY: If the object is a buffer then the `buffer` field can be read
                let buffer = unsafe { ffi_acpi_object.buffer };

                let bytes =
                // SAFETY: The object is valid so the pointer and length are correct
                    unsafe { core::slice::from_raw_parts(buffer.pointer, buffer.length as _) };

                Self::Buffer(bytes)
            }
            _t @ ACPI_TYPE_PACKAGE => {
                // SAFETY: If the object is a package then the `package` field can be read
                let package = unsafe { ffi_acpi_object.package };

                Self::Package(AcpiObjectPackage {
                    count: package.count,
                    elements: package.elements,
                })
            }
            _t @ ACPI_TYPE_LOCAL_REFERENCE => {
                // SAFETY: If the object is a reference then the `reference` field can be read
                let reference = unsafe { ffi_acpi_object.reference };

                Self::Reference(AcpiObjectReference {
                    actual_type: reference.actual_type,
                    handle: reference.handle,
                })
            }
            _t @ ACPI_TYPE_PROCESSOR => {
                // SAFETY: If the object is a processor then the `processor` field can be read
                let processor = unsafe { ffi_acpi_object.processor };

                Self::Processor(AcpiObjectProcessor {
                    proc_id: processor.proc_id,
                    pblk_address: processor.pblk_address,
                    pblk_length: processor.pblk_length,
                })
            }
            _t @ ACPI_TYPE_POWER => {
                // SAFETY: If the object is a power resource then the `power_resource` field can be read
                let power_resource = unsafe { ffi_acpi_object.power_resource };

                Self::PowerResource(AcpiObjectPowerResource {
                    system_level: power_resource.system_level,
                    resource_order: power_resource.resource_order,
                })
            }

            _ => Self::Any,
        }
    }

    /// Copies the data from an ACPI object, indicated by its type number and a pointer to the object (for integer objects, the pointer itself stores the value)
    ///
    /// # Safety
    /// `val` must be valid data of the type indicated by `object_type`
    pub(crate) unsafe fn from_type_and_val(object_type: u8, val: *mut i8) -> Self {
        match object_type.into() {
            _t @ ACPI_TYPE_ANY => Self::Any,
            _t @ ACPI_TYPE_INTEGER => Self::Integer(val as _),
            _t @ ACPI_TYPE_STRING => {
                // SAFETY: If the object is a string then the pointer points to a null terminated string
                let s = unsafe {
                    CStr::from_ptr(val)
                        .to_str()
                        .expect("String object should have been valid utf-8")
                };
                Self::String(s)
            }

            _ => {
                warn!(target: "acpi_object_from_type_and_val", "Check object type meaning. Type is {}, val is {:p}", object_type, val);
                Self::Any
            }
        }
    }

    /// Gets the type of the object
    #[must_use]
    pub fn get_type(&self) -> AcpiObjectType {
        match self {
            AcpiObject::Any => AcpiObjectType::Any,
            AcpiObject::Integer(_) => AcpiObjectType::Integer,
            AcpiObject::String(_) => AcpiObjectType::String,
            AcpiObject::Buffer(_) => AcpiObjectType::Buffer,
            AcpiObject::Package(_) => AcpiObjectType::Package,
            AcpiObject::Reference(_) => AcpiObjectType::Reference,
            AcpiObject::Processor(_) => AcpiObjectType::Processor,
            AcpiObject::PowerResource(_) => AcpiObjectType::PowerResource,
        }
    }
}

/// A type of an [`AcpiObject`]. This is used when the type of data is known but the value is not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcpiObjectType {
    /// The object can be any type, or the type is not known.
    /// From ACPICA comments: "\[Any\] is used to indicate a NULL package element or an unresolved named reference."
    Any,
    /// The object is an integer
    Integer,
    /// The object is a string
    String,
    /// The object is a buffer of bytes
    Buffer,
    /// The object is a package containing other AML data
    Package,
    /// The object is a reference to another [`AcpiObject`]
    Reference,
    /// The object describes the features of a processor
    Processor,
    /// The object describes a power resource
    PowerResource,
}

impl AcpiObjectType {
    pub(crate) fn from_type_id(id: FfiAcpiObjectType) -> Self {
        match id {
            _t @ ACPI_TYPE_ANY => Self::Any,
            _t @ ACPI_TYPE_INTEGER => Self::Integer,
            _t @ ACPI_TYPE_STRING => Self::String,
            _t @ ACPI_TYPE_BUFFER => Self::Buffer,
            _t @ ACPI_TYPE_PACKAGE => Self::Package,
            _t @ ACPI_TYPE_LOCAL_REFERENCE => Self::Reference,
            _t @ ACPI_TYPE_PROCESSOR => Self::Processor,
            _t @ ACPI_TYPE_POWER => Self::PowerResource,

            _ => Self::Any,
        }
    }
}
