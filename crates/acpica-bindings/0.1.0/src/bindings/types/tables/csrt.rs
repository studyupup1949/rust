use crate::bindings::types::FfiAcpiTableHeader;

///  CSRT - Core System Resource Table
///         Version 0
///
///  Conforms to the \"Core System Resource Table (CSRT)\", November 14, 2011
///
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableCsrt {
    pub header: FfiAcpiTableHeader,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiCsrtGroup {
    pub length: u32,
    pub vendor_id: u32,
    pub subvendor_id: u32,
    pub device_id: u16,
    pub subdevice_id: u16,
    pub revision: u16,
    pub reserved: u16,
    pub shared_info_length: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiCsrtSharedInfo {
    pub major_version: u16,
    pub minor_version: u16,
    pub mmio_base_low: u32,
    pub mmio_base_high: u32,
    pub gsi_interrupt: u32,
    pub interrupt_polarity: u8,
    pub interrupt_mode: u8,
    pub num_channels: u8,
    pub dma_address_width: u8,
    pub base_request_line: u16,
    pub num_handshake_signals: u16,
    pub max_block_size: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiCsrtDescriptor {
    pub length: u32,
    pub descriptor_type: u16,
    pub subtype: u16,
    pub uid: u32,
}
