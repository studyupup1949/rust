use crate::interface::status::AcpiStatus;

use super::types::{
    functions::{
        FfiAcpiAdrSpaceHandler, FfiAcpiAdrSpaceSetup, FfiAcpiEventHandler, FfiAcpiExceptionHandler,
        FfiAcpiGblEventHandler, FfiAcpiGpeHandler, FfiAcpiInitHandler, FfiAcpiInterfaceHandler,
        FfiAcpiNotifyHandler, FfiAcpiObjectHandler, FfiAcpiSciHandler, FfiAcpiTableHandler,
        FfiAcpiWalkCallback, FfiAcpiWalkResourceCallback,
    },
    object::FfiAcpiObjectType,
    tables::FfiAcpiTableHeader,
    FfiAcpiAdtSpaceType, FfiAcpiBuffer, FfiAcpiDeviceInfo, FfiAcpiEventStatus,
    FfiAcpiGenericAddress, FfiAcpiHandle, FfiAcpiObjectList, FfiAcpiPhysicalAddress,
    FfiAcpiPldInfo, FfiAcpiResource, FfiAcpiResourceAddress64, FfiAcpiSize, FfiAcpiStatistics,
    FfiAcpiString, FfiAcpiTableDesc, FfiAcpiVendorUuid,
};

#[allow(dead_code)]
extern "C" {
    pub(crate) fn AcpiInitializeTables(
        InitialStorage: *mut FfiAcpiTableDesc,
        InitialTableCount: u32,
        AllowResize: bool,
    ) -> AcpiStatus;

    pub(crate) fn AcpiInitializeSubsystem() -> AcpiStatus;

    pub(crate) fn AcpiEnableSubsystem(Flags: u32) -> AcpiStatus;

    pub(crate) fn AcpiInitializeObjects(Flags: u32) -> AcpiStatus;

    pub(crate) fn AcpiTerminate() -> AcpiStatus;

    pub(crate) fn AcpiEnable() -> AcpiStatus;

    pub(crate) fn AcpiDisable() -> AcpiStatus;

    pub(crate) fn AcpiSubsystemStatus() -> AcpiStatus;

    pub(crate) fn AcpiGetSystemInfo(RetBuffer: *mut FfiAcpiBuffer) -> AcpiStatus;

    pub(crate) fn AcpiGetStatistics(Stats: *mut FfiAcpiStatistics) -> AcpiStatus;

    pub(crate) fn AcpiFormatException(Exception: AcpiStatus) -> *const i8;

    pub(crate) fn AcpiPurgeCachedObjects() -> AcpiStatus;

    pub(crate) fn AcpiInstallInterface(InterfaceName: FfiAcpiString) -> AcpiStatus;

    pub(crate) fn AcpiRemoveInterface(InterfaceName: FfiAcpiString) -> AcpiStatus;

    pub(crate) fn AcpiUpdateInterfaces(Action: u8) -> AcpiStatus;

    pub(crate) fn AcpiCheckAddressRange(
        SpaceId: FfiAcpiAdtSpaceType,
        Address: FfiAcpiPhysicalAddress,
        Length: FfiAcpiSize,
        Warn: bool,
    ) -> u32;

    pub(crate) fn AcpiDecodePldBuffer(
        InBuffer: *mut u8,
        Length: FfiAcpiSize,
        ReturnBuffer: *mut *mut FfiAcpiPldInfo,
    ) -> AcpiStatus;

    pub(crate) fn AcpiInstallTable(Address: FfiAcpiPhysicalAddress, Physical: bool) -> AcpiStatus;

    pub(crate) fn AcpiLoadTable(Table: *mut FfiAcpiTableHeader, TableIdx: *mut u32) -> AcpiStatus;

    pub(crate) fn AcpiUnloadTable(TableIndex: u32) -> AcpiStatus;

    pub(crate) fn AcpiUnloadParentTable(Object: FfiAcpiHandle) -> AcpiStatus;

    pub(crate) fn AcpiLoadTables() -> AcpiStatus;

    pub(crate) fn AcpiReallocateRootTable() -> AcpiStatus;

    pub(crate) fn AcpiFindRootPointer(RsdpAddress: *mut FfiAcpiPhysicalAddress) -> AcpiStatus;

    pub(crate) fn AcpiGetTableHeader(
        Signature: FfiAcpiString,
        Instance: u32,
        OutTableHeader: *mut FfiAcpiTableHeader,
    ) -> AcpiStatus;

    pub(crate) fn AcpiGetTable(
        Signature: FfiAcpiString,
        Instance: u32,
        OutTable: *mut *mut FfiAcpiTableHeader,
    ) -> AcpiStatus;

    pub(crate) fn AcpiPutTable(Table: *mut FfiAcpiTableHeader);

    pub(crate) fn AcpiGetTableByIndex(
        TableIndex: u32,
        OutTable: *mut *mut FfiAcpiTableHeader,
    ) -> AcpiStatus;

    pub(crate) fn AcpiInstallTableHandler(
        Handler: FfiAcpiTableHandler,
        Context: *mut ::core::ffi::c_void,
    ) -> AcpiStatus;

    pub(crate) fn AcpiRemoveTableHandler(Handler: FfiAcpiTableHandler) -> AcpiStatus;

    pub(crate) fn AcpiWalkNamespace(
        Type: FfiAcpiObjectType,
        StartObject: FfiAcpiHandle,
        MaxDepth: u32,
        DescendingCallback: FfiAcpiWalkCallback,
        AscendingCallback: FfiAcpiWalkCallback,
        Context: *mut ::core::ffi::c_void,
        ReturnValue: *mut *mut ::core::ffi::c_void,
    ) -> AcpiStatus;

    pub(crate) fn AcpiGetDevices(
        HID: *mut i8,
        UserFunction: FfiAcpiWalkCallback,
        Context: *mut ::core::ffi::c_void,
        ReturnValue: *mut *mut ::core::ffi::c_void,
    ) -> AcpiStatus;

    pub(crate) fn AcpiGetName(
        Object: FfiAcpiHandle,
        NameType: u32,
        RetPathPtr: *mut FfiAcpiBuffer,
    ) -> AcpiStatus;

    pub(crate) fn AcpiGetHandle(
        Parent: FfiAcpiHandle,
        Pathname: FfiAcpiString,
        RetHandle: *mut FfiAcpiHandle,
    ) -> AcpiStatus;

    pub(crate) fn AcpiAttachData(
        Object: FfiAcpiHandle,
        Handler: FfiAcpiObjectHandler,
        Data: *mut ::core::ffi::c_void,
    ) -> AcpiStatus;

    pub(crate) fn AcpiDetachData(
        Object: FfiAcpiHandle,
        Handler: FfiAcpiObjectHandler,
    ) -> AcpiStatus;

    pub(crate) fn AcpiGetData(
        Object: FfiAcpiHandle,
        Handler: FfiAcpiObjectHandler,
        Data: *mut *mut ::core::ffi::c_void,
    ) -> AcpiStatus;

    pub(crate) fn AcpiDebugTrace(
        Name: *const i8,
        DebugLevel: u32,
        DebugLayer: u32,
        Flags: u32,
    ) -> AcpiStatus;

    pub(crate) fn AcpiEvaluateObject(
        Object: FfiAcpiHandle,
        Pathname: FfiAcpiString,
        ParameterObjects: *mut FfiAcpiObjectList,
        ReturnObjectBuffer: *mut FfiAcpiBuffer,
    ) -> AcpiStatus;

    pub(crate) fn AcpiEvaluateObjectTyped(
        Object: FfiAcpiHandle,
        Pathname: FfiAcpiString,
        ExternalParams: *mut FfiAcpiObjectList,
        ReturnBuffer: *mut FfiAcpiBuffer,
        ReturnType: FfiAcpiObjectType,
    ) -> AcpiStatus;

    pub(crate) fn AcpiGetObjectInfo(
        Object: FfiAcpiHandle,
        ReturnBuffer: *mut *mut FfiAcpiDeviceInfo,
    ) -> AcpiStatus;

    pub(crate) fn AcpiInstallMethod(Buffer: *mut u8) -> AcpiStatus;

    pub(crate) fn AcpiGetNextObject(
        Type: FfiAcpiObjectType,
        Parent: FfiAcpiHandle,
        Child: FfiAcpiHandle,
        OutHandle: *mut FfiAcpiHandle,
    ) -> AcpiStatus;

    pub(crate) fn AcpiGetType(Object: FfiAcpiHandle, OutType: *mut FfiAcpiObjectType)
        -> AcpiStatus;

    pub(crate) fn AcpiGetParent(Object: FfiAcpiHandle, OutHandle: *mut FfiAcpiHandle)
        -> AcpiStatus;

    pub(crate) fn AcpiInstallInitializationHandler(
        Handler: FfiAcpiInitHandler,
        Function: u32,
    ) -> AcpiStatus;

    pub(crate) fn AcpiInstallSciHandler(
        Address: FfiAcpiSciHandler,
        Context: *mut ::core::ffi::c_void,
    ) -> AcpiStatus;

    pub(crate) fn AcpiRemoveSciHandler(Address: FfiAcpiSciHandler) -> AcpiStatus;

    pub(crate) fn AcpiInstallGlobalEventHandler(
        Handler: FfiAcpiGblEventHandler,
        Context: *mut ::core::ffi::c_void,
    ) -> AcpiStatus;

    pub(crate) fn AcpiInstallFixedEventHandler(
        AcpiEvent: u32,
        Handler: FfiAcpiEventHandler,
        Context: *mut ::core::ffi::c_void,
    ) -> AcpiStatus;

    pub(crate) fn AcpiRemoveFixedEventHandler(
        AcpiEvent: u32,
        Handler: FfiAcpiEventHandler,
    ) -> AcpiStatus;

    pub(crate) fn AcpiInstallGpeHandler(
        GpeDevice: FfiAcpiHandle,
        GpeNumber: u32,
        Type: u32,
        Address: FfiAcpiGpeHandler,
        Context: *mut ::core::ffi::c_void,
    ) -> AcpiStatus;

    pub(crate) fn AcpiInstallGpeRawHandler(
        GpeDevice: FfiAcpiHandle,
        GpeNumber: u32,
        Type: u32,
        Address: FfiAcpiGpeHandler,
        Context: *mut ::core::ffi::c_void,
    ) -> AcpiStatus;

    pub(crate) fn AcpiRemoveGpeHandler(
        GpeDevice: FfiAcpiHandle,
        GpeNumber: u32,
        Address: FfiAcpiGpeHandler,
    ) -> AcpiStatus;

    pub(crate) fn AcpiInstallNotifyHandler(
        Device: FfiAcpiHandle,
        HandlerType: u32,
        Handler: FfiAcpiNotifyHandler,
        Context: *mut ::core::ffi::c_void,
    ) -> AcpiStatus;

    pub(crate) fn AcpiRemoveNotifyHandler(
        Device: FfiAcpiHandle,
        HandlerType: u32,
        Handler: FfiAcpiNotifyHandler,
    ) -> AcpiStatus;

    pub(crate) fn AcpiInstallAddressSpaceHandler(
        Device: FfiAcpiHandle,
        SpaceId: FfiAcpiAdtSpaceType,
        Handler: FfiAcpiAdrSpaceHandler,
        Setup: FfiAcpiAdrSpaceSetup,
        Context: *mut ::core::ffi::c_void,
    ) -> AcpiStatus;

    pub(crate) fn AcpiRemoveAddressSpaceHandler(
        Device: FfiAcpiHandle,
        SpaceId: FfiAcpiAdtSpaceType,
        Handler: FfiAcpiAdrSpaceHandler,
    ) -> AcpiStatus;

    pub(crate) fn AcpiInstallExceptionHandler(Handler: FfiAcpiExceptionHandler) -> AcpiStatus;

    pub(crate) fn AcpiInstallInterfaceHandler(Handler: FfiAcpiInterfaceHandler) -> AcpiStatus;

    pub(crate) fn AcpiAcquireGlobalLock(Timeout: u16, Handle: *mut u32) -> AcpiStatus;

    pub(crate) fn AcpiReleaseGlobalLock(Handle: u32) -> AcpiStatus;

    pub(crate) fn AcpiAcquireMutex(
        Handle: FfiAcpiHandle,
        Pathname: FfiAcpiString,
        Timeout: u16,
    ) -> AcpiStatus;

    pub(crate) fn AcpiReleaseMutex(Handle: FfiAcpiHandle, Pathname: FfiAcpiString) -> AcpiStatus;

    pub(crate) fn AcpiEnableEvent(Event: u32, Flags: u32) -> AcpiStatus;

    pub(crate) fn AcpiDisableEvent(Event: u32, Flags: u32) -> AcpiStatus;

    pub(crate) fn AcpiClearEvent(Event: u32) -> AcpiStatus;

    pub(crate) fn AcpiGetEventStatus(
        Event: u32,
        EventStatus: *mut FfiAcpiEventStatus,
    ) -> AcpiStatus;

    pub(crate) fn AcpiUpdateAllGpes() -> AcpiStatus;

    pub(crate) fn AcpiEnableGpe(GpeDevice: FfiAcpiHandle, GpeNumber: u32) -> AcpiStatus;

    pub(crate) fn AcpiDisableGpe(GpeDevice: FfiAcpiHandle, GpeNumber: u32) -> AcpiStatus;

    pub(crate) fn AcpiClearGpe(GpeDevice: FfiAcpiHandle, GpeNumber: u32) -> AcpiStatus;

    pub(crate) fn AcpiSetGpe(GpeDevice: FfiAcpiHandle, GpeNumber: u32, Action: u8) -> AcpiStatus;

    pub(crate) fn AcpiFinishGpe(GpeDevice: FfiAcpiHandle, GpeNumber: u32) -> AcpiStatus;

    pub(crate) fn AcpiMaskGpe(
        GpeDevice: FfiAcpiHandle,
        GpeNumber: u32,
        IsMasked: bool,
    ) -> AcpiStatus;

    pub(crate) fn AcpiMarkGpeForWake(GpeDevice: FfiAcpiHandle, GpeNumber: u32) -> AcpiStatus;

    pub(crate) fn AcpiSetupGpeForWake(
        ParentDevice: FfiAcpiHandle,
        GpeDevice: FfiAcpiHandle,
        GpeNumber: u32,
    ) -> AcpiStatus;

    pub(crate) fn AcpiSetGpeWakeMask(
        GpeDevice: FfiAcpiHandle,
        GpeNumber: u32,
        Action: u8,
    ) -> AcpiStatus;

    pub(crate) fn AcpiGetGpeStatus(
        GpeDevice: FfiAcpiHandle,
        GpeNumber: u32,
        EventStatus: *mut FfiAcpiEventStatus,
    ) -> AcpiStatus;

    pub(crate) fn AcpiDispatchGpe(GpeDevice: FfiAcpiHandle, GpeNumber: u32) -> u32;

    pub(crate) fn AcpiDisableAllGpes() -> AcpiStatus;

    pub(crate) fn AcpiEnableAllRuntimeGpes() -> AcpiStatus;

    pub(crate) fn AcpiEnableAllWakeupGpes() -> AcpiStatus;

    pub(crate) fn AcpiAnyGpeStatusSet() -> u32;

    pub(crate) fn AcpiGetGpeDevice(GpeIndex: u32, GpeDevice: *mut FfiAcpiHandle) -> AcpiStatus;

    pub(crate) fn AcpiInstallGpeBlock(
        GpeDevice: FfiAcpiHandle,
        GpeBlockAddress: *mut FfiAcpiGenericAddress,
        RegisterCount: u32,
        InterruptNumber: u32,
    ) -> AcpiStatus;

    pub(crate) fn AcpiRemoveGpeBlock(GpeDevice: FfiAcpiHandle) -> AcpiStatus;

    pub(crate) fn AcpiGetVendorResource(
        Device: FfiAcpiHandle,
        Name: *mut i8,
        Uuid: *mut FfiAcpiVendorUuid,
        RetBuffer: *mut FfiAcpiBuffer,
    ) -> AcpiStatus;

    pub(crate) fn AcpiGetCurrentResources(
        Device: FfiAcpiHandle,
        RetBuffer: *mut FfiAcpiBuffer,
    ) -> AcpiStatus;

    pub(crate) fn AcpiGetPossibleResources(
        Device: FfiAcpiHandle,
        RetBuffer: *mut FfiAcpiBuffer,
    ) -> AcpiStatus;

    pub(crate) fn AcpiGetEventResources(
        DeviceHandle: FfiAcpiHandle,
        RetBuffer: *mut FfiAcpiBuffer,
    ) -> AcpiStatus;

    pub(crate) fn AcpiWalkResourceBuffer(
        Buffer: *mut FfiAcpiBuffer,
        UserFunction: FfiAcpiWalkResourceCallback,
        Context: *mut ::core::ffi::c_void,
    ) -> AcpiStatus;

    pub(crate) fn AcpiWalkResources(
        Device: FfiAcpiHandle,
        Name: *mut i8,
        UserFunction: FfiAcpiWalkResourceCallback,
        Context: *mut ::core::ffi::c_void,
    ) -> AcpiStatus;

    pub(crate) fn AcpiSetCurrentResources(
        Device: FfiAcpiHandle,
        InBuffer: *mut FfiAcpiBuffer,
    ) -> AcpiStatus;

    pub(crate) fn AcpiGetIrqRoutingTable(
        Device: FfiAcpiHandle,
        RetBuffer: *mut FfiAcpiBuffer,
    ) -> AcpiStatus;

    pub(crate) fn AcpiResourceToAddress64(
        Resource: *mut FfiAcpiResource,
        Out: *mut FfiAcpiResourceAddress64,
    ) -> AcpiStatus;

    pub(crate) fn AcpiBufferToResource(
        AmlBuffer: *mut u8,
        AmlBufferLength: u16,
        ResourcePtr: *mut *mut FfiAcpiResource,
    ) -> AcpiStatus;

    pub(crate) fn AcpiReset() -> AcpiStatus;

    pub(crate) fn AcpiRead(Value: *mut u64, Reg: *mut FfiAcpiGenericAddress) -> AcpiStatus;

    pub(crate) fn AcpiWrite(Value: u64, Reg: *mut FfiAcpiGenericAddress) -> AcpiStatus;

    pub(crate) fn AcpiReadBitRegister(RegisterId: u32, ReturnValue: *mut u32) -> AcpiStatus;

    pub(crate) fn AcpiWriteBitRegister(RegisterId: u32, Value: u32) -> AcpiStatus;

    pub(crate) fn AcpiGetSleepTypeData(
        SleepState: u8,
        Slp_TypA: *mut u8,
        Slp_TypB: *mut u8,
    ) -> AcpiStatus;

    pub(crate) fn AcpiEnterSleepStatePrep(SleepState: u8) -> AcpiStatus;

    pub(crate) fn AcpiEnterSleepState(SleepState: u8) -> AcpiStatus;

    pub(crate) fn AcpiEnterSleepStateS4bios() -> AcpiStatus;

    pub(crate) fn AcpiLeaveSleepStatePrep(SleepState: u8) -> AcpiStatus;

    pub(crate) fn AcpiLeaveSleepState(SleepState: u8) -> AcpiStatus;

    pub(crate) fn AcpiSetFirmwareWakingVector(
        PhysicalAddress: FfiAcpiPhysicalAddress,
        PhysicalAddress64: FfiAcpiPhysicalAddress,
    ) -> AcpiStatus;

    pub(crate) fn AcpiGetTimerResolution(Resolution: *mut u32) -> AcpiStatus;

    pub(crate) fn AcpiGetTimer(Ticks: *mut u32) -> AcpiStatus;

    pub(crate) fn AcpiGetTimerDuration(
        StartTicks: u32,
        EndTicks: u32,
        TimeElapsed: *mut u32,
    ) -> AcpiStatus;

    pub(crate) fn AcpiError(ModuleName: *const i8, LineNumber: u32, Format: *const i8, ...);

    pub(crate) fn AcpiException(
        ModuleName: *const i8,
        LineNumber: u32,
        Status: AcpiStatus,
        Format: *const i8,
        ...
    );

    pub(crate) fn AcpiWarning(ModuleName: *const i8, LineNumber: u32, Format: *const i8, ...);

    pub(crate) fn AcpiInfo(Format: *const i8, ...);

    pub(crate) fn AcpiBiosError(ModuleName: *const i8, LineNumber: u32, Format: *const i8, ...);

    pub(crate) fn AcpiBiosException(
        ModuleName: *const i8,
        LineNumber: u32,
        Status: AcpiStatus,
        Format: *const i8,
        ...
    );

    pub(crate) fn AcpiBiosWarning(ModuleName: *const i8, LineNumber: u32, Format: *const i8, ...);

    pub(crate) fn AcpiInitializeDebugger() -> AcpiStatus;

    pub(crate) fn AcpiTerminateDebugger();

    pub(crate) fn AcpiRunDebugger(BatchBuffer: *mut i8);

    pub(crate) fn AcpiSetDebuggerThreadId(ThreadId: u64);
}
