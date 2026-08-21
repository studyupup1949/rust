use crate::{
    bindings::types::FfiAcpiTableHeader,
    bindings::types::{FfiAcpiGenericAddress, IncompleteArrayField},
};

use super::FfiAcpiSubtableHeader;

///  ASF - Alert Standard Format table (Signature \"ASF!\")
///        Revision 0x10
///
///  Conforms to the Alert Standard Format Specification V2.0, 23 April 2003
///
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableAsf {
    pub header: FfiAcpiTableHeader,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiAsfHeader {
    pub header_type: u8,
    pub reserved: u8,
    pub length: u16,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiAsfType {
    Info = 0,
    Alert = 1,
    Control = 2,
    Boot = 3,
    Address = 4,
    Reserved = 5,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiAsfInfo {
    pub header: FfiAcpiAsfHeader,
    pub min_reset_value: u8,
    pub min_poll_interval: u8,
    pub system_id: u16,
    pub mfg_id: u32,
    pub flags: u8,
    pub reserved2: [u8; 3usize],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiAsfAlert {
    pub header: FfiAcpiAsfHeader,
    pub assert_mask: u8,
    pub deassert_mask: u8,
    pub alerts: u8,
    pub data_length: u8,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiAsfAlertData {
    pub address: u8,
    pub command: u8,
    pub mask: u8,
    pub value: u8,
    pub sensor_type: u8,
    pub data_type: u8,
    pub offset: u8,
    pub source_type: u8,
    pub severity: u8,
    pub sensor_number: u8,
    pub entity: u8,
    pub instance: u8,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiAsfRemote {
    pub header: FfiAcpiAsfHeader,
    pub controls: u8,
    pub data_length: u8,
    pub reserved2: u16,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiAsfControlData {
    pub function: u8,
    pub address: u8,
    pub command: u8,
    pub value: u8,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiAsfRmcp {
    pub header: FfiAcpiAsfHeader,
    pub capabilities: [u8; 7usize],
    pub completion_code: u8,
    pub enterprise_id: u32,
    pub command: u8,
    pub parameter: u16,
    pub boot_options: u16,
    pub oem_parameters: u16,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiAsfAddress {
    pub header: FfiAcpiAsfHeader,
    pub eprom_address: u8,
    pub devices: u8,
}

///  BERT - Boot Error Record Table (ACPI 4.0)
///         Version 1
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableBert {
    pub header: FfiAcpiTableHeader,
    pub region_length: u32,
    pub address: u64,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiBertRegion {
    pub block_status: u32,
    pub raw_data_offset: u32,
    pub raw_data_length: u32,
    pub data_length: u32,
    pub error_severity: u32,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiBertErrorSeverity {
    Correctable = 0,
    Fatal = 1,
    Corrected = 2,
    None = 3,
    Reserved = 4,
}

///  BGRT - Boot Graphics Resource Table (ACPI 5.0)
///         Version 1
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableBgrt {
    pub header: FfiAcpiTableHeader,
    pub version: u16,
    pub status: u8,
    pub image_type: u8,
    pub image_address: u64,
    pub image_offset_x: u32,
    pub image_offset_y: u32,
}

///  BOOT - Simple Boot Flag Table
///         Version 1
///
///  Conforms to the \"Simple Boot Flag Specification\", Version 2.1
///
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableBoot {
    pub header: FfiAcpiTableHeader,
    pub cmos_index: u8,
    pub reserved: [u8; 3usize],
}

///  CPEP - Corrected Platform Error Polling table (ACPI 4.0)
///         Version 1
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableCpep {
    pub header: FfiAcpiTableHeader,
    pub reserved: u64,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiCpepPolling {
    pub header: FfiAcpiSubtableHeader,
    pub id: u8,
    pub eid: u8,
    pub interval: u32,
}

///  DBGP - Debug Port table
///         Version 1
///
///  Conforms to the \"Debug Port Specification\", Version 1.00, 2/9/2000
///
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableDbgp {
    pub header: FfiAcpiTableHeader,
    pub port_type: u8,
    pub reserved: [u8; 3usize],
    pub debug_port: FfiAcpiGenericAddress,
}

///  ECDT - Embedded Controller Boot Resources Table
///         Version 1
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableEcdt {
    pub header: FfiAcpiTableHeader,
    pub control: FfiAcpiGenericAddress,
    pub data: FfiAcpiGenericAddress,
    pub uid: u32,
    pub gpe: u8,
    pub id: [u8; 1usize],
}

///  BDAT - BIOS Data ACPI Table
///
///  Conforms to \"BIOS Data ACPI Table\", Interface Specification v4.0 Draft 5
///  Nov 2020
///
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableBdat {
    pub header: FfiAcpiTableHeader,
    pub gas: FfiAcpiGenericAddress,
}

///  LPIT - Low Power Idle Table
///
///  Conforms to \"ACPI Low Power Idle Table (LPIT)\" July 2014.
///
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableLpit {
    pub header: FfiAcpiTableHeader,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiLpitHeader {
    pub header_type: u32,
    pub length: u32,
    pub unique_id: u16,
    pub reserved: u16,
    pub flags: u32,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiLpitType {
    NativeCstate = 0,
    Reserved = 1,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiLpitNative {
    pub header: FfiAcpiLpitHeader,
    pub entry_trigger: FfiAcpiGenericAddress,
    pub residency: u32,
    pub latency: u32,
    pub residency_counter: FfiAcpiGenericAddress,
    pub counter_frequency: u64,
}

///  MCFG - PCI Memory Mapped Configuration table and subtable
///         Version 1
///
///  Conforms to \"PCI Firmware Specification\", Revision 3.0, June 20, 2005
///
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableMcfg {
    pub header: FfiAcpiTableHeader,
    pub reserved: [u8; 8usize],
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiMcfgAllocation {
    pub address: u64,
    pub pci_segment: u16,
    pub start_bus_number: u8,
    pub end_bus_number: u8,
    pub reserved: u32,
}

///  MCHI - Management Controller Host Interface Table
///         Version 1
///
///  Conforms to \"Management Component Transport Protocol (MCTP) Host
///  Interface Specification\", Revision 1.0.0a, October 13, 2009
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableMchi {
    pub header: FfiAcpiTableHeader,
    pub interface_type: u8,
    pub protocol: u8,
    pub protocol_data: u64,
    pub interrupt_type: u8,
    pub gpe: u8,
    pub pci_device_flag: u8,
    pub global_interrupt: u32,
    pub control_register: FfiAcpiGenericAddress,
    pub pci_segment: u8,
    pub pci_bus: u8,
    pub pci_device: u8,
    pub pci_function: u8,
}

///  MSCT - Maximum System Characteristics Table (ACPI 4.0)
///         Version 1
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableMsct {
    pub header: FfiAcpiTableHeader,
    pub proximity_offset: u32,
    pub max_proximity_domains: u32,
    pub max_clock_domains: u32,
    pub max_address: u64,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiMsctProximity {
    pub revision: u8,
    pub length: u8,
    pub range_start: u32,
    pub range_end: u32,
    pub processor_capacity: u32,
    pub memory_capacity: u64,
}

///  MSDM - Microsoft Data Management table
///
///  Conforms to \"Microsoft Software Licensing Tables (SLIC and MSDM)\",
///  November 29, 2011. Copyright 2011 Microsoft
///
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableMsdm {
    pub header: FfiAcpiTableHeader,
}

///  PDTT - Platform Debug Trigger Table (ACPI 6.2)
///         Version 0
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTablePdtt {
    pub header: FfiAcpiTableHeader,
    pub trigger_count: u8,
    pub reserved: [u8; 3usize],
    pub array_offset: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiPdttChannel {
    pub subchannel_id: u8,
    pub flags: u8,
}

///  RGRT - Regulatory Graphics Resource Table
///         Version 1
///
///  Conforms to \"ACPI RGRT\" available at:
///  <https://microsoft.github.io/mu/dyn/mu_plus/MsCorePkg/AcpiRGRT/feature_acpi_rgrt/>
///
#[repr(C, packed)]
pub(crate) struct FfiAcpiTableRgrt {
    pub header: FfiAcpiTableHeader,
    pub version: u16,
    pub image_type: u8,
    pub reserved: u8,
    image: IncompleteArrayField<u8>,
}

#[repr(u32)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum FfiAcpiRgrtImageType {
    TypeReserved0 = 0,
    ImageTypePng = 1,
    TypeReserved = 2,
}

///  SBST - Smart Battery Specification Table
///         Version 1
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableSbst {
    pub header: FfiAcpiTableHeader,
    pub warning_level: u32,
    pub low_level: u32,
    pub critical_level: u32,
}

///  SDEI - Software Delegated Exception Interface Descriptor Table
///
///  Conforms to \"Software Delegated Exception Interface (SDEI)\" ARM DEN0054A,
///  May 8th, 2017. Copyright 2017 ARM Ltd.
///
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableSdei {
    pub header: FfiAcpiTableHeader,
}

///  SVKL - Storage Volume Key Location Table (ACPI 6.4)
///         From: \"Guest-Host-Communication Interface (GHCI) for Intel
///         Trust Domain Extensions (Intel TDX)\".
///         Version 1
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableSvkl {
    pub header: FfiAcpiTableHeader,
    pub count: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiSvklKey {
    pub key_type: u16,
    pub format: u16,
    pub size: u32,
    pub address: u64,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiSvklType {
    MainStorage = 0,
    Reserved = 1,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiSvklFormat {
    RawBinary = 0,
    Reserved = 1,
}

///  SLIC - Software Licensing Description Table
///
///  Conforms to \"Microsoft Software Licensing Tables (SLIC and MSDM)\",
///  November 29, 2011. Copyright 2011 Microsoft
///
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableSlic {
    pub header: FfiAcpiTableHeader,
}

///  SLIT - System Locality Distance Information Table
///         Version 1
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableSlit {
    pub header: FfiAcpiTableHeader,
    pub locality_count: u64,
    pub entry: [u8; 1usize],
}

///  SPCR - Serial Port Console Redirection table
///         Version 2
///
///  Conforms to \"Serial Port Console Redirection Table\",
///  Version 1.03, August 10, 2015
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableSpcr {
    pub header: FfiAcpiTableHeader,
    pub interface_type: u8,
    pub reserved: [u8; 3usize],
    pub serial_port: FfiAcpiGenericAddress,
    pub interrupt_type: u8,
    pub pc_interrupt: u8,
    pub interrupt: u32,
    pub baud_rate: u8,
    pub parity: u8,
    pub stop_bits: u8,
    pub flow_control: u8,
    pub terminal_type: u8,
    pub reserved1: u8,
    pub pci_device_id: u16,
    pub pci_vendor_id: u16,
    pub pci_bus: u8,
    pub pci_device: u8,
    pub pci_function: u8,
    pub pci_flags: u32,
    pub pci_segment: u8,
    pub reserved2: u32,
}

///  SPMI - Server Platform Management Interface table
///         Version 5
///
///  Conforms to \"Intelligent Platform Management Interface Specification
///  Second Generation v2.0\", Document Revision 1.0, February 12, 2004 with
///  June 12, 2009 markup.
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableSpmi {
    pub header: FfiAcpiTableHeader,
    pub interface_type: u8,
    pub reserved: u8,
    pub spec_revision: u16,
    pub interrupt_type: u8,
    pub gpe_number: u8,
    pub reserved1: u8,
    pub pci_device_flag: u8,
    pub interrupt: u32,
    pub ipmi_register: FfiAcpiGenericAddress,
    pub pci_segment: u8,
    pub pci_bus: u8,
    pub pci_device: u8,
    pub pci_function: u8,
    pub reserved2: u8,
}
#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiSpmiInterfaceTypes {
    NotUsed = 0,
    Keyboard = 1,
    Smi = 2,
    BlockTransfer = 3,
    Smbus = 4,
    Reserved = 5,
}

///  STAO - Status Override Table (_STA override) - ACPI 6.0
///         Version 1
///
///  Conforms to \"ACPI Specification for Status Override Table\"
///  6 January 2015
///
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableStao {
    pub header: FfiAcpiTableHeader,
    pub ignore_uart: u8,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableTcpaClient {
    pub minimum_log_length: u32,
    pub log_address: u64,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableTcpaServer {
    pub reserved: u16,
    pub minimum_log_length: u64,
    pub log_address: u64,
    pub spec_revision: u16,
    pub device_flags: u8,
    pub interrupt_flags: u8,
    pub gpe_number: u8,
    pub reserved2: [u8; 3usize],
    pub global_interrupt: u32,
    pub address: FfiAcpiGenericAddress,
    pub reserved3: u32,
    pub config_address: FfiAcpiGenericAddress,
    pub group: u8,
    pub bus: u8,
    pub device: u8,
    pub function: u8,
}

///  UEFI - UEFI Boot optimization Table
///         Version 1
///
///  Conforms to \"Unified Extensible Firmware Interface Specification\",
///  Version 2.3, May 8, 2009
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableUefi {
    pub header: FfiAcpiTableHeader,
    pub identifier: [u8; 16usize],
    pub data_offset: u16,
}

///  XENV - Xen Environment Table (ACPI 6.0)
///         Version 1
///
///  Conforms to \"ACPI Specification for Xen Environment Table\" 4 January 2015
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableXenv {
    pub header: FfiAcpiTableHeader,
    pub grant_table_address: u64,
    pub grant_table_size: u64,
    pub event_interrupt: u32,
    pub event_flags: u8,
}
