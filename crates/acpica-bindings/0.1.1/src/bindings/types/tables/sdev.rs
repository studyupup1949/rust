use crate::bindings::types::FfiAcpiTableHeader;

///  SDEV - Secure Devices Table (ACPI 6.2)
///         Version 1
///
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableSdev {
    pub header: FfiAcpiTableHeader,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiSdevHeader {
    pub header_type: u8,
    pub flags: u8,
    pub length: u16,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiSdevType {
    NamespaceDevice = 0,
    PcieEndpointDevice = 1,
    Reserved = 2,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiSdevNamespace {
    pub header: FfiAcpiSdevHeader,
    pub device_id_offset: u16,
    pub device_id_length: u16,
    pub vendor_data_offset: u16,
    pub vendor_data_length: u16,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiSdevSecureComponent {
    pub secure_component_offset: u16,
    pub secure_component_length: u16,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiSdevComponent {
    pub header: FfiAcpiSdevHeader,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiSacType {
    IdComponent = 0,
    MemComponent = 1,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiSdevIdComponent {
    pub header: FfiAcpiSdevHeader,
    pub hardware_id_offset: u16,
    pub hardware_id_length: u16,
    pub subsystem_id_offset: u16,
    pub subsystem_id_length: u16,
    pub hardware_revision: u16,
    pub hardware_rev_present: u8,
    pub class_code_present: u8,
    pub pci_base_class: u8,
    pub pci_sub_class: u8,
    pub pci_programming_xface: u8,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiSdevMemComponent {
    pub header: FfiAcpiSdevHeader,
    pub reserved: u32,
    pub memory_base_address: u64,
    pub memory_length: u64,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiSdevPcie {
    pub header: FfiAcpiSdevHeader,
    pub segment: u16,
    pub start_bus: u16,
    pub path_offset: u16,
    pub path_length: u16,
    pub vendor_data_offset: u16,
    pub vendor_data_length: u16,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiSdevPciePath {
    pub device: u8,
    pub function: u8,
}
