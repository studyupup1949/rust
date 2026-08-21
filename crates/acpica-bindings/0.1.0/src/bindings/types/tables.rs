pub mod aest;
pub mod cedt;
pub mod csrt;
pub mod dbg2;
pub mod dmar;
pub mod drtm;
pub mod einj;
pub mod erst;
pub mod fadt;
pub mod fpdt;
pub mod gtdt;
pub mod hest;
pub mod hmat;
pub mod hpet;
pub mod ibft;
pub mod iort;
pub mod ivrs;
pub mod madt;
pub mod misc;
pub mod mpst;
pub mod nfit;
pub mod pcct;
pub mod phat;
pub mod pmtt;
pub mod pptt;
pub mod prmt;
pub mod rasf;
pub mod rsdt;
pub mod sdev;
pub mod srat;
pub mod trusted_platform;
pub mod viot;
pub mod watchdog;
pub mod windows;

///  Master ACPI Table Header. This common header is used by all ACPI tables
///  except the RSDP and FACS.
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableHeader {
    pub signature: [u8; 4usize],
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oem_id: [u8; 6usize],
    pub oem_table_id: [u8; 8usize],
    pub oem_revision: u32,
    pub asl_compiler_id: [u8; 4usize],
    pub asl_compiler_revision: u32,
}

///  Common subtable headers
///
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiSubtableHeader {
    pub subtable_type: u8,
    pub length: u8,
}

///  RSDP - Root System Description Pointer (Signature is \"RSD PTR \")
///         Version 2
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableRsdp {
    pub signature: [i8; 8usize],
    pub checksum: u8,
    pub oem_id: [i8; 6usize],
    pub revision: u8,
    pub rsdt_physical_address: u32,
    pub length: u32,
    pub xsdt_physical_address: u64,
    pub extended_checksum: u8,
    pub reserved: [u8; 3usize],
}
///  RSDP - Root System Description Pointer (Signature is \"RSD PTR \")
///         Version 2
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiRsdpCommon {
    pub signature: [i8; 8usize],
    pub checksum: u8,
    pub oem_id: [i8; 6usize],
    pub revision: u8,
    pub rsdt_physical_address: u32,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiRsdpExtension {
    pub length: u32,
    pub xsdt_physical_address: u64,
    pub extended_checksum: u8,
    pub reserved: [u8; 3usize],
}
