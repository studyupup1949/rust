use super::types::tables::fadt::FfiAcpiTableFadt;

#[allow(dead_code)]
extern "C" {
    pub(crate) static mut AcpiGbl_EnableInterpreterSlack: u8;

    pub(crate) static mut AcpiGbl_AutoSerializeMethods: u8;

    pub(crate) static mut AcpiGbl_CreateOsiMethod: u8;

    pub(crate) static mut AcpiGbl_UseDefaultRegisterWidths: u8;

    pub(crate) static mut AcpiGbl_EnableTableValidation: u8;

    pub(crate) static mut AcpiGbl_EnableAmlDebugObject: u8;

    pub(crate) static mut AcpiGbl_CopyDsdtLocally: u8;

    pub(crate) static mut AcpiGbl_DoNotUseXsdt: u8;

    pub(crate) static mut AcpiGbl_Use32BitFadtAddresses: u8;

    pub(crate) static mut AcpiGbl_Use32BitFacsAddresses: u8;

    pub(crate) static mut AcpiGbl_TruncateIoAddresses: u8;

    pub(crate) static mut AcpiGbl_DisableAutoRepair: u8;

    pub(crate) static mut AcpiGbl_DisableSsdtTableInstall: u8;

    pub(crate) static mut AcpiGbl_RuntimeNamespaceOverride: u8;

    pub(crate) static mut AcpiGbl_OsiData: u8;

    pub(crate) static mut AcpiGbl_ReducedHardware: bool;

    pub(crate) static mut AcpiGbl_MaxLoopIterations: u32;

    pub(crate) static mut AcpiGbl_IgnorePackageResolutionErrors: bool;

    pub(crate) static mut AcpiGbl_TraceFlags: u32;

    pub(crate) static mut AcpiGbl_TraceMethodName: *const i8;

    pub(crate) static mut AcpiGbl_TraceDbgLevel: u32;

    pub(crate) static mut AcpiGbl_TraceDbgLayer: u32;

    pub(crate) static mut AcpiDbgLevel: u32;

    pub(crate) static mut AcpiDbgLayer: u32;

    pub(crate) static mut AcpiGbl_DisplayDebugTimer: u8;

    pub(crate) static mut AcpiGbl_FADT: FfiAcpiTableFadt;

    pub(crate) static mut AcpiCurrentGpeCount: u32;

    pub(crate) static mut AcpiGbl_SystemAwakeAndRunning: bool;
}
