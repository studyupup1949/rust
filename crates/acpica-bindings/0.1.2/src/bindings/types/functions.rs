use crate::interface::status::AcpiStatus;

use super::{FfiAcpiHandle, FfiAcpiName, FfiAcpiPhysicalAddress, FfiAcpiResource, FfiAcpiString};

pub(crate) type FfiAcpiOsdHandler = unsafe extern "C" fn(Context: *mut ::core::ffi::c_void) -> u32;
pub(crate) type FfiAcpiOsdExecCallback = unsafe extern "C" fn(Context: *mut ::core::ffi::c_void);
pub(crate) type FfiAcpiSciHandler = unsafe extern "C" fn(Context: *mut ::core::ffi::c_void) -> u32;
pub(crate) type FfiAcpiGblEventHandler = unsafe extern "C" fn(
    EventType: u32,
    Device: FfiAcpiHandle,
    EventNumber: u32,
    Context: *mut ::core::ffi::c_void,
);
pub(crate) type FfiAcpiEventHandler =
    unsafe extern "C" fn(Context: *mut ::core::ffi::c_void) -> u32;
pub(crate) type FfiAcpiGpeHandler = unsafe extern "C" fn(
    GpeDevice: FfiAcpiHandle,
    GpeNumber: u32,
    Context: *mut ::core::ffi::c_void,
) -> u32;
pub(crate) type FfiAcpiNotifyHandler =
    unsafe extern "C" fn(Device: FfiAcpiHandle, Value: u32, Context: *mut ::core::ffi::c_void);
pub(crate) type FfiAcpiObjectHandler =
    unsafe extern "C" fn(Object: FfiAcpiHandle, Data: *mut ::core::ffi::c_void);
pub(crate) type FfiAcpiInitHandler =
    unsafe extern "C" fn(Object: FfiAcpiHandle, Function: u32) -> AcpiStatus;
pub(crate) type FfiAcpiExceptionHandler = unsafe extern "C" fn(
    AmlStatus: AcpiStatus,
    Name: FfiAcpiName,
    Opcode: u16,
    AmlOffset: u32,
    Context: *mut ::core::ffi::c_void,
) -> AcpiStatus;
pub(crate) type FfiAcpiTableHandler = unsafe extern "C" fn(
    Event: u32,
    Table: *mut ::core::ffi::c_void,
    Context: *mut ::core::ffi::c_void,
) -> AcpiStatus;
pub(crate) type FfiAcpiAdrSpaceHandler = unsafe extern "C" fn(
    Function: u32,
    Address: FfiAcpiPhysicalAddress,
    BitWidth: u32,
    Value: *mut u64,
    HandlerContext: *mut ::core::ffi::c_void,
    RegionContext: *mut ::core::ffi::c_void,
) -> AcpiStatus;
pub(crate) type FfiAcpiAdrSpaceSetup = unsafe extern "C" fn(
    RegionHandle: FfiAcpiHandle,
    Function: u32,
    HandlerContext: *mut ::core::ffi::c_void,
    RegionContext: *mut *mut ::core::ffi::c_void,
) -> AcpiStatus;
pub(crate) type FfiAcpiWalkCallback = unsafe extern "C" fn(
    Object: FfiAcpiHandle,
    NestingLevel: u32,
    Context: *mut ::core::ffi::c_void,
    ReturnValue: *mut *mut ::core::ffi::c_void,
) -> AcpiStatus;
pub(crate) type FfiAcpiWalkResourceCallback = unsafe extern "C" fn(
    Resource: *mut FfiAcpiResource,
    Context: *mut ::core::ffi::c_void,
) -> AcpiStatus;
pub(crate) type FfiAcpiInterfaceHandler =
    unsafe extern "C" fn(InterfaceName: FfiAcpiString, Supported: u32) -> u32;
