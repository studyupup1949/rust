use super::{FfiAcpiHandle, FfiAcpiIoAddress};

pub(crate) type FfiAcpiObjectType = u32;

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) union FfiAcpiObject {
    pub object_type: FfiAcpiObjectType,
    pub integer: FfiObjectTypeInteger,
    pub string: FfiObjectTypeString,
    pub buffer: FfiObjectTypeBuffer,
    pub package: FfiObjectTypePackage,
    pub reference: FfiObjectTypeReference,
    pub processor: FfiObjectTypeProcessor,
    pub power_resource: FfiObjectTypePowerResource,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiObjectTypeInteger {
    pub object_type: FfiAcpiObjectType,
    pub value: u64,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiObjectTypeString {
    pub object_type: FfiAcpiObjectType,
    pub length: u32,
    pub pointer: *mut i8,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiObjectTypeBuffer {
    pub object_type: FfiAcpiObjectType,
    pub length: u32,
    pub pointer: *mut u8,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiObjectTypePackage {
    pub object_type: FfiAcpiObjectType,
    pub count: u32,
    pub elements: *mut FfiAcpiObject,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiObjectTypeReference {
    pub object_type: FfiAcpiObjectType,
    pub actual_type: FfiAcpiObjectType,
    pub handle: FfiAcpiHandle,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiObjectTypeProcessor {
    pub object_type: FfiAcpiObjectType,
    pub proc_id: u32,
    pub pblk_address: FfiAcpiIoAddress,
    pub pblk_length: u32,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiObjectTypePowerResource {
    pub object_type: FfiAcpiObjectType,
    pub system_level: u32,
    pub resource_order: u32,
}
