use crate::bindings::types::FfiAcpiTableHeader;

///  TCPA - Trusted Computing Platform Alliance table
///         Version 2
///
///  TCG Hardware Interface Table for TPM 1.2 Clients and Servers
///
///  Conforms to \"TCG ACPI Specification, Family 1.2 and 2.0\",
///  Version 1.2, Revision 8
///  February 27, 2017
///
///  NOTE: There are two versions of the table with the same signature --
///  the client version and the server version. The common `platform_class`
///  field is used to differentiate the two types of tables.
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableTcpaHdr {
    pub header: FfiAcpiTableHeader,
    pub platform_class: u16,
}

///  TPM2 - Trusted Platform Module (TPM) 2.0 Hardware Interface Table
///         Version 4
///
///  TCG Hardware Interface Table for TPM 2.0 Clients and Servers
///
///  Conforms to \"TCG ACPI Specification, Family 1.2 and 2.0\",
///  Version 1.2, Revision 8
///  February 27, 2017
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableTpm23 {
    pub header: FfiAcpiTableHeader,
    pub reserved: u32,
    pub control_address: u64,
    pub start_method: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTmp23Trailer {
    pub reserved: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableTpm2 {
    pub header: FfiAcpiTableHeader,
    pub platform_class: u16,
    pub reserved: u16,
    pub control_address: u64,
    pub start_method: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTpm2Trailer {
    pub method_parameters: [u8; 12usize],
    pub minimum_log_length: u32,
    pub log_address: u64,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTpm2ArmSmc {
    pub global_interrupt: u32,
    pub interrupt_flags: u8,
    pub operation_flags: u8,
    pub reserved: u16,
    pub function_id: u32,
}
