use crate::bindings::types::FfiAcpiTableHeader;

#[allow(clippy::doc_markdown)]
///  IBFT - Boot Firmware Table
///         Version 1
///
///  Conforms to \"iSCSI Boot Firmware Table (iBFT) as Defined in ACPI 3.0b
///  Specification\", Version 1.01, March 1, 2007
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableIbft {
    pub header: FfiAcpiTableHeader,
    pub reserved: [u8; 12usize],
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiIbftHeader {
    pub header_type: u8,
    pub version: u8,
    pub length: u16,
    pub index: u8,
    pub flags: u8,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiIbftType {
    NotUsed = 0,
    Control = 1,
    Initiator = 2,
    Nic = 3,
    Target = 4,
    Extensions = 5,
    Reserved = 6,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiIbftControl {
    pub header: FfiAcpiIbftHeader,
    pub extensions: u16,
    pub initiator_offset: u16,
    pub nic0_offset: u16,
    pub target0_offset: u16,
    pub nic1_offset: u16,
    pub target1_offset: u16,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiIbftInitiator {
    pub header: FfiAcpiIbftHeader,
    pub sns_server: [u8; 16usize],
    pub slp_server: [u8; 16usize],
    pub primary_server: [u8; 16usize],
    pub secondary_server: [u8; 16usize],
    pub name_length: u16,
    pub name_offset: u16,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiIbftNic {
    pub header: FfiAcpiIbftHeader,
    pub ip_address: [u8; 16usize],
    pub subnet_mask_prefix: u8,
    pub origin: u8,
    pub gateway: [u8; 16usize],
    pub primary_dns: [u8; 16usize],
    pub secondary_dns: [u8; 16usize],
    pub dhcp: [u8; 16usize],
    pub vlan: u16,
    pub mac_address: [u8; 6usize],
    pub pci_address: u16,
    pub name_length: u16,
    pub name_offset: u16,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiIbftTarget {
    pub header: FfiAcpiIbftHeader,
    pub target_ip_address: [u8; 16usize],
    pub target_ip_socket: u16,
    pub target_boot_lun: [u8; 8usize],
    pub chap_type: u8,
    pub nic_association: u8,
    pub target_name_length: u16,
    pub target_name_offset: u16,
    pub chap_name_length: u16,
    pub chap_name_offset: u16,
    pub chap_secret_length: u16,
    pub chap_secret_offset: u16,
    pub reverse_chap_name_length: u16,
    pub reverse_chap_name_offset: u16,
    pub reverse_chap_secret_length: u16,
    pub reverse_chap_secret_offset: u16,
}
