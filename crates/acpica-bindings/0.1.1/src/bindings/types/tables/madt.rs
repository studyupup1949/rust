use crate::bindings::types::FfiAcpiTableHeader;

use super::FfiAcpiSubtableHeader;

///  MADT - Multiple APIC Description Table
///         Version 3
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableMadt {
    pub header: FfiAcpiTableHeader,
    pub address: u32,
    pub flags: u32,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiMadtType {
    LocalApic = 0,
    IoApic = 1,
    InterruptOverride = 2,
    NmiSource = 3,
    LocalApicNmi = 4,
    LocalApicOverride = 5,
    IoSapic = 6,
    LocalSapic = 7,
    InterruptSource = 8,
    LocalX2apic = 9,
    LocalX2apicNmi = 10,
    GenericInterrupt = 11,
    GenericDistributor = 12,
    GenericMsiFrame = 13,
    GenericRedistributor = 14,
    GenericTranslator = 15,
    MultiprocWakeup = 16,
    Reserved = 17,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiMadtLocalApic {
    pub header: FfiAcpiSubtableHeader,
    pub processor_id: u8,
    pub id: u8,
    pub lapic_flags: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiMadtIoApic {
    pub header: FfiAcpiSubtableHeader,
    pub id: u8,
    pub reserved: u8,
    pub address: u32,
    pub global_irq_base: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiMadtInterruptOverride {
    pub header: FfiAcpiSubtableHeader,
    pub bus: u8,
    pub source_irq: u8,
    pub global_irq: u32,
    pub inti_flags: u16,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiMadtNmiSource {
    pub header: FfiAcpiSubtableHeader,
    pub inti_flags: u16,
    pub global_irq: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiMadtLocalApicNmi {
    pub header: FfiAcpiSubtableHeader,
    pub processor_id: u8,
    pub inti_flags: u16,
    pub lint: u8,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiMadtLocalApicOverride {
    pub header: FfiAcpiSubtableHeader,
    pub reserved: u16,
    pub address: u64,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiMadtIoSapic {
    pub header: FfiAcpiSubtableHeader,
    pub id: u8,
    pub reserved: u8,
    pub global_irq_base: u32,
    pub address: u64,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiMadtLocalSapic {
    pub header: FfiAcpiSubtableHeader,
    pub processor_id: u8,
    pub id: u8,
    pub eid: u8,
    pub reserved: [u8; 3usize],
    pub lapic_flags: u32,
    pub uid: u32,
    pub uid_string: [i8; 1usize],
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiMadtInterruptSource {
    pub header: FfiAcpiSubtableHeader,
    pub inti_flags: u16,
    pub source_type: u8,
    pub id: u8,
    pub eid: u8,
    pub io_sapic_vector: u8,
    pub global_irq: u32,
    pub flags: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiMadtLocalX2apic {
    pub header: FfiAcpiSubtableHeader,
    pub reserved: u16,
    pub local_apic_id: u32,
    pub lapic_flags: u32,
    pub uid: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiMadtLocalX2apicNmi {
    pub header: FfiAcpiSubtableHeader,
    pub inti_flags: u16,
    pub uid: u32,
    pub lint: u8,
    pub reserved: [u8; 3usize],
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiMadtGenericInterrupt {
    pub header: FfiAcpiSubtableHeader,
    pub reserved: u16,
    pub cpu_interface_number: u32,
    pub uid: u32,
    pub flags: u32,
    pub parking_version: u32,
    pub performance_interrupt: u32,
    pub parked_address: u64,
    pub base_address: u64,
    pub gicv_base_address: u64,
    pub gich_base_address: u64,
    pub vgic_interrupt: u32,
    pub gicr_base_address: u64,
    pub arm_mpidr: u64,
    pub efficiency_class: u8,
    pub reserved2: [u8; 1usize],
    pub spe_interrupt: u16,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiMadtGenericDistributor {
    pub header: FfiAcpiSubtableHeader,
    pub reserved: u16,
    pub gic_id: u32,
    pub base_address: u64,
    pub global_irq_base: u32,
    pub version: u8,
    pub reserved2: [u8; 3usize],
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiMadtGicVersion {
    VersionNone = 0,
    VersionV1 = 1,
    VersionV2 = 2,
    VersionV3 = 3,
    VersionV4 = 4,
    VersionReserved = 5,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiMadtGenericMsiFrame {
    pub header: FfiAcpiSubtableHeader,
    pub reserved: u16,
    pub msi_frame_id: u32,
    pub base_address: u64,
    pub flags: u32,
    pub spi_count: u16,
    pub spi_base: u16,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiMadtGenericRedistributor {
    pub header: FfiAcpiSubtableHeader,
    pub reserved: u16,
    pub base_address: u64,
    pub length: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiMadtGenericTranslator {
    pub header: FfiAcpiSubtableHeader,
    pub reserved: u16,
    pub translation_id: u32,
    pub base_address: u64,
    pub reserved2: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiMadtMultiprocWakeup {
    pub header: FfiAcpiSubtableHeader,
    pub mailbox_version: u16,
    pub reserved: u32,
    pub base_address: u64,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub(crate) struct FfiAcpiMadtMultiprocWakeupMailbox {
    pub command: u16,
    pub reserved: u16,
    pub apic_id: u32,
    pub wakeup_vector: u64,
    pub reserved_os: [u8; 2032usize],
    pub reserved_firmware: [u8; 2048usize],
}
