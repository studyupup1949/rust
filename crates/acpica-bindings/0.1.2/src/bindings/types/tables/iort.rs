use crate::bindings::types::FfiAcpiTableHeader;

///  IORT - IO Remapping Table
///
///  Conforms to \"IO Remapping Table System Software on ARM Platforms\",
///  Document number: ARM DEN 0049E.b, Feb 2021
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableIort {
    pub header: FfiAcpiTableHeader,
    pub node_count: u32,
    pub node_offset: u32,
    pub reserved: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiIortNode {
    pub node_type: u8,
    pub length: u16,
    pub revision: u8,
    pub identifier: u32,
    pub mapping_count: u32,
    pub mapping_offset: u32,
    pub node_data: [i8; 1usize],
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiIortNodeType {
    ItsGroup = 0,
    NamedComponent = 1,
    PciRootComplex = 2,
    Smmu = 3,
    SmmuV3 = 4,
    Pmcg = 5,
    Rmr = 6,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiIortIdMapping {
    pub input_base: u32,
    pub id_count: u32,
    pub output_base: u32,
    pub output_reference: u32,
    pub flags: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiIortMemoryAccess {
    pub cache_coherency: u32,
    pub hints: u8,
    pub reserved: u16,
    pub memory_flags: u8,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiIortItsGroup {
    pub its_count: u32,
    pub identifiers: [u32; 1usize],
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiIortNamedComponent {
    pub node_flags: u32,
    pub memory_properties: u64,
    pub memory_address_limit: u8,
    pub device_name: [i8; 1usize],
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiIortRootComplex {
    pub memory_properties: u64,
    pub ats_attribute: u32,
    pub pci_segment_number: u32,
    pub memory_address_limit: u8,
    pub reserved: [u8; 3usize],
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiIortSmmu {
    pub base_address: u64,
    pub span: u64,
    pub model: u32,
    pub flags: u32,
    pub global_interrupt_offset: u32,
    pub context_interrupt_count: u32,
    pub context_interrupt_offset: u32,
    pub pmu_interrupt_count: u32,
    pub pmu_interrupt_offset: u32,
    pub interrupts: [u64; 1usize],
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiIortSmmuGsi {
    pub n_sg_irpt: u32,
    pub n_sg_irpt_flags: u32,
    pub n_sg_cfg_irpt: u32,
    pub n_sg_cfg_irpt_flags: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiIortSmmuV3 {
    pub base_address: u64,
    pub flags: u32,
    pub reserved: u32,
    pub vatos_address: u64,
    pub model: u32,
    pub event_gsiv: u32,
    pub pri_gsiv: u32,
    pub gerr_gsiv: u32,
    pub sync_gsiv: u32,
    pub pxm: u32,
    pub id_mapping_index: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiIortPmcg {
    pub page0_base_address: u64,
    pub overflow_gsiv: u32,
    pub node_reference: u32,
    pub page1_base_address: u64,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiIortRmr {
    pub flags: u32,
    pub rmr_count: u32,
    pub rmr_offset: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiIortRmrDesc {
    pub base_address: u64,
    pub length: u64,
    pub reserved: u32,
}
