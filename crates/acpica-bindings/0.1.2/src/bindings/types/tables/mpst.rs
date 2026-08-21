use crate::bindings::types::FfiAcpiTableHeader;

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiTableMpst {
    pub header: FfiAcpiTableHeader,
    pub channel_id: u8,
    pub reserved1: [u8; 3usize],
    pub power_node_count: u16,
    pub reserved2: u16,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiMpstChannel {
    pub channel_id: u8,
    pub reserved1: [u8; 3usize],
    pub power_node_count: u16,
    pub reserved2: u16,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiMpstPowerNode {
    pub flags: u8,
    pub reserved1: u8,
    pub node_id: u16,
    pub length: u32,
    pub range_address: u64,
    pub range_length: u64,
    pub num_power_states: u32,
    pub num_physical_components: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiMpstPowerState {
    pub power_state: u8,
    pub info_index: u8,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiMpstComponent {
    pub component_id: u16,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiMpstDataHdr {
    pub characteristics_count: u16,
    pub reserved: u16,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiMpstPowerData {
    pub(crate) structure_id: u8,
    pub flags: u8,
    pub reserved1: u16,
    pub average_power: u32,
    pub power_saving: u32,
    pub exit_latency: u64,
    pub reserved2: u64,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiMpstShared {
    pub signature: u32,
    pub pcc_command: u16,
    pub pcc_status: u16,
    pub command_register: u32,
    pub status_register: u32,
    pub power_state_id: u32,
    pub power_node_id: u32,
    pub energy_consumed: u64,
    pub average_power: u64,
}
