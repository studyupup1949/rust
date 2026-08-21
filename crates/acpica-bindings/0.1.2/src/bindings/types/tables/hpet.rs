use crate::bindings::types::{FfiAcpiGenericAddress, FfiAcpiTableHeader};

///  HPET - High Precision Event Timer table
///         Version 1
///
///  Conforms to \"IA-PC HPET (High Precision Event Timers) Specification\",
///  Version 1.0a, October 2004
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableHpet {
    pub header: FfiAcpiTableHeader,
    pub id: u32,
    pub address: FfiAcpiGenericAddress,
    pub sequence: u8,
    pub minimum_tick: u16,
    pub flags: u8,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiHpetPageProtect {
    NoPageProtect = 0,
    PageProtect4 = 1,
    PageProtect64 = 2,
}
