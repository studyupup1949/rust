use crate::bindings::types::FfiAcpiTableHeader;

///  WAET - Windows ACPI Emulated devices Table
///         Version 1
///
///  Conforms to \"Windows ACPI Emulated Devices Table\", version 1.0, April 6, 2009
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableWaet {
    pub header: FfiAcpiTableHeader,
    pub flags: u32,
}

///  WPBT - Windows Platform Environment Table (ACPI 6.0)
///         Version 1
///
///  Conforms to \"Windows Platform Binary Table (WPBT)\" 29 November 2011
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableWpbt {
    pub header: FfiAcpiTableHeader,
    pub handoff_size: u32,
    pub handoff_address: u64,
    pub layout: u8,
    pub table_type: u8,
    pub arguments_length: u16,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiWpbtUnicode {
    pub unicode_string: *mut u16,
}

///  WSMT - Windows SMM Security Mitigations Table
///         Version 1
///
///  Conforms to \"Windows SMM Security Mitigations Table\",
///  Version 1.0, April 18, 2016
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableWsmt {
    pub header: FfiAcpiTableHeader,
    pub protection_flags: u32,
}
