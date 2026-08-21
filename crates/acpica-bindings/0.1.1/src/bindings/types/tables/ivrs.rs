use crate::bindings::types::FfiAcpiTableHeader;

///  IVRS - I/O Virtualization Reporting Structure
///         Version 1
///
///  Conforms to \"AMD I/O Virtualization Technology (IOMMU) Specification\",
///  Revision 1.26, February 2009.
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableIvrs {
    pub header: FfiAcpiTableHeader,
    pub info: u32,
    pub reserved: u64,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiIvrsHeader {
    pub header_type: u8,
    pub flags: u8,
    pub length: u16,
    pub device_id: u16,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiIvrsType {
    Hardware1 = 16,
    Hardware2 = 17,
    Hardware3 = 64,
    Memory1 = 32,
    Memory2 = 33,
    Memory3 = 34,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiIvrsHardware10 {
    pub header: FfiAcpiIvrsHeader,
    pub capability_offset: u16,
    pub base_address: u64,
    pub pci_segment_group: u16,
    pub info: u16,
    pub feature_reporting: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiIvrsHardware11 {
    pub header: FfiAcpiIvrsHeader,
    pub capability_offset: u16,
    pub base_address: u64,
    pub pci_segment_group: u16,
    pub info: u16,
    pub attributes: u32,
    pub efr_register_image: u64,
    pub reserved: u64,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiIvrsDeHeader {
    pub header_type: u8,
    pub id: u16,
    pub data_setting: u8,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiIvrsDeviceEntryType {
    Pad4 = 0,
    All = 1,
    Select = 2,
    Start = 3,
    End = 4,
    Pad8 = 64,
    NotUsed = 65,
    AliasSelect = 66,
    AliasStart = 67,
    ExtSelect = 70,
    ExtStart = 71,
    Special = 72,
    Hid = 240,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiIvrsDevice4 {
    pub header: FfiAcpiIvrsDeHeader,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiIvrsDevice8a {
    pub header: FfiAcpiIvrsDeHeader,
    pub reserved1: u8,
    pub used_id: u16,
    pub reserved2: u8,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiIvrsDevice8b {
    pub header: FfiAcpiIvrsDeHeader,
    pub extended_data: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiIvrsDevice8c {
    pub header: FfiAcpiIvrsDeHeader,
    pub handle: u8,
    pub used_id: u16,
    pub variety: u8,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiIvrsDeviceHid {
    pub header: FfiAcpiIvrsDeHeader,
    pub acpi_hid: u64,
    pub acpi_cid: u64,
    pub uid_type: u8,
    pub uid_length: u8,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiIvrsMemory {
    pub header: FfiAcpiIvrsDeHeader,
    pub aux_data: u16,
    pub reserved: u64,
    pub start_address: u64,
    pub memory_length: u64,
}
