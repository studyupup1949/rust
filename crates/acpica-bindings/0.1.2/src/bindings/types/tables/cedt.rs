use crate::{bindings::types::FfiAcpiTableHeader, bindings::types::IncompleteArrayField};

///  CEDT - CXL Early Discovery Table
///         Version 1
///
///  Conforms to the \"CXL Early Discovery Table\" (CXL 2.0)
///
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableCedt {
    pub header: FfiAcpiTableHeader,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiCedtHeader {
    pub header_type: u8,
    pub reserved: u8,
    pub length: u16,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiCedtType {
    Chbs = 0,
    Cfmws = 1,
    Reserved = 2,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiCedtChbs {
    pub header: FfiAcpiCedtHeader,
    pub uid: u32,
    pub cxl_version: u32,
    pub reserved: u32,
    pub base: u64,
    pub length: u64,
}

#[repr(C, packed)]
pub(crate) struct FfiAcpiCedtCfmws {
    pub header: FfiAcpiCedtHeader,
    pub reserved1: u32,
    pub base_hpa: u64,
    pub window_size: u64,
    pub interleave_ways: u8,
    pub interleave_arithmetic: u8,
    pub reserved2: u16,
    pub granularity: u32,
    pub restrictions: u16,
    pub qtg_id: u16,
    interleave_targets: IncompleteArrayField<u32>,
}
