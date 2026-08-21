pub mod functions;
pub mod object;
pub(crate) mod tables;

use core::{marker::PhantomData, slice};

// pub(crate) use tables::*;
use tables::FfiAcpiTableHeader;

use self::object::{FfiAcpiObject, FfiAcpiObjectType};

/// The last field of a C struct can be an array with no fixed size.
/// This struct is the rust equivalent.
/// The type can be cast to a slice using the [`as_slice`] and [`as_mut_slice`] methods.
///
/// [`as_slice`]: IncompleteArrayField::as_slice
/// [`as_mut_slice`]: IncompleteArrayField::as_mut_slice
#[repr(C)]
#[derive(Default)]
pub(crate) struct IncompleteArrayField<T>(PhantomData<T>, [T; 0]);
impl<T> IncompleteArrayField<T> {
    #[inline]
    pub(crate) const fn new() -> Self {
        IncompleteArrayField(PhantomData, [])
    }
    #[inline]
    pub(crate) fn as_ptr(&self) -> *const T {
        (self as *const Self).cast::<T>()
    }
    #[inline]
    pub(crate) fn as_mut_ptr(&mut self) -> *mut T {
        (self as *mut Self).cast::<T>()
    }
    /// # Safety
    /// The [`IncompleteArrayField`] must have at least `len` elements in it
    #[inline]
    pub(crate) unsafe fn as_slice(&self, len: usize) -> &[T] {
        // SAFETY: The list has at least [`len`] elements, so this is sound
        unsafe { slice::from_raw_parts(self.as_ptr(), len) }
    }
    /// # Safety
    /// The [`IncompleteArrayField`] must have at least `len` elements in it
    #[inline]
    pub(crate) unsafe fn as_mut_slice(&mut self, len: usize) -> &mut [T] {
        // SAFETY: The list has at least [`len`] elements, so this is sound
        unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), len) }
    }
}
impl<T> ::core::fmt::Debug for IncompleteArrayField<T> {
    fn fmt(&self, fmt: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        fmt.write_str("__IncompleteArrayField")
    }
}
pub(crate) type FfiAcpiSize = usize;
pub(crate) type FfiAcpiIoAddress = u64;
pub(crate) type FfiAcpiPhysicalAddress = usize;
pub(crate) type FfiAcpiName = u32;
pub(crate) type FfiAcpiString = *mut i8;
pub(crate) type FfiAcpiHandle = *mut ::core::ffi::c_void;
pub(crate) type FfiAcpiOwnerId = u16;
pub(crate) type FfiAcpiEventStatus = u32;
pub(crate) type FfiAcpiAdtSpaceType = u8;
pub(crate) type FfiAcpiCpuFlags = FfiAcpiSize;

///  GAS - Generic Address Structure (ACPI 2.0+)
///
///  Note: Since this structure is used in the ACPI tables, it is byte aligned.
///  If misaligned access is not supported by the hardware, accesses to the
///  64-bit Address field must be performed with care.
///
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiGenericAddress {
    pub(crate) space_id: u8,
    pub(crate) bit_width: u8,
    pub(crate) bit_offset: u8,
    pub(crate) access_width: u8,
    pub(crate) address: u64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiObjectList {
    pub(crate) count: u32,
    pub(crate) pointer: *mut FfiAcpiObject,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiBuffer<'a> {
    pub(crate) length: FfiAcpiSize,
    pub(crate) pointer: *mut ::core::ffi::c_void,
    pub _p: PhantomData<&'a mut [u8]>,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiPredefinedNames {
    pub(crate) name: *const i8,
    pub(crate) object_type: u8,
    pub(crate) val: *mut i8,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiStatistics {
    pub(crate) sci_count: u32,
    pub(crate) gpe_count: u32,
    pub(crate) fixed_event_count: [u32; 5usize],
    pub(crate) method_count: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiPnpDeviceId {
    pub(crate) length: u32,
    pub(crate) string: *mut i8,
}
#[repr(C)]
#[derive(Debug)]
pub(crate) struct FfiAcpiPnpDeviceIdList {
    pub(crate) count: u32,
    pub(crate) list_size: u32,
    pub(crate) ids: IncompleteArrayField<FfiAcpiPnpDeviceId>,
}
#[repr(C)]
#[derive(Debug)]
pub(crate) struct FfiAcpiDeviceInfo {
    pub(crate) info_size: u32,
    pub(crate) name: [u8; 4],
    pub(crate) object_type: FfiAcpiObjectType,
    pub(crate) param_count: u8,
    pub(crate) valid: u16,
    pub(crate) flags: u8,
    pub(crate) highest_dstates: [u8; 4usize],
    pub(crate) lowest_dstates: [u8; 5usize],
    pub(crate) address: u64,
    pub(crate) hardware_id: FfiAcpiPnpDeviceId,
    pub(crate) unique_id: FfiAcpiPnpDeviceId,
    pub(crate) class_code: FfiAcpiPnpDeviceId,
    pub(crate) compatible_id_list: FfiAcpiPnpDeviceIdList,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiPciId {
    pub(crate) segment: u16,
    pub(crate) bus: u16,
    pub(crate) device: u16,
    pub(crate) function: u16,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiMemMapping {
    pub(crate) physical_address: FfiAcpiPhysicalAddress,
    pub(crate) logical_address: *mut u8,
    pub(crate) length: FfiAcpiSize,
    pub(crate) next_mm: *mut FfiAcpiMemMapping,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiMemSpaceContext {
    pub(crate) length: u32,
    pub(crate) address: FfiAcpiPhysicalAddress,
    pub(crate) cur_mm: *mut FfiAcpiMemMapping,
    pub(crate) first_mm: *mut FfiAcpiMemMapping,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiMemoryList {
    pub(crate) list_name: *const i8,
    pub(crate) list_head: *mut ::core::ffi::c_void,
    pub(crate) object_size: u16,
    pub(crate) max_depth: u16,
    pub(crate) current_depth: u16,
}
#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub enum FfiAcpiTraceEventType {
    Method = 0,
    Opcode = 1,
    Region = 2,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiExceptionInfo {
    pub(crate) name: *mut i8,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) union FfiAcpiNameUnion {
    pub(crate) integer: u32,
    pub(crate) ascii: [i8; 4usize],
}
#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct FfiAcpiTableDesc {
    pub(crate) address: FfiAcpiPhysicalAddress,
    pub(crate) pointer: *mut FfiAcpiTableHeader,
    pub(crate) length: u32,
    pub(crate) signature: FfiAcpiNameUnion,
    pub(crate) owner_id: FfiAcpiOwnerId,
    pub(crate) flags: u8,
    pub(crate) validation_count: u16,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiWheaHeader {
    pub(crate) action: u8,
    pub(crate) instruction: u8,
    pub(crate) flags: u8,
    pub(crate) reserved: u8,
    pub(crate) register_region: FfiAcpiGenericAddress,
    pub(crate) value: u64,
    pub(crate) mask: u64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiUuid {
    pub(crate) data: [u8; 16usize],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiVendorUuid {
    pub(crate) subtype: u8,
    pub(crate) data: [u8; 16usize],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiResourceIrq {
    pub(crate) descriptor_length: u8,
    pub(crate) triggering: u8,
    pub(crate) polarity: u8,
    pub(crate) shareable: u8,
    pub(crate) wake_capable: u8,
    pub(crate) interrupt_count: u8,
    pub(crate) interrupts: [u8; 1usize],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiResourceDma {
    pub(crate) resource_type: u8,
    pub(crate) bus_master: u8,
    pub(crate) transfer: u8,
    pub(crate) channel_count: u8,
    pub(crate) channels: [u8; 1usize],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiResourceStartDependent {
    pub(crate) descriptor_length: u8,
    pub(crate) compatibility_priority: u8,
    pub(crate) performance_robustness: u8,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiResourceIo {
    pub(crate) io_decode: u8,
    pub(crate) alignment: u8,
    pub(crate) address_length: u8,
    pub(crate) minimum: u16,
    pub(crate) maximum: u16,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiResourceFixedIo {
    pub(crate) address: u16,
    pub(crate) address_length: u8,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiResourceFixedDma {
    pub(crate) request_lines: u16,
    pub(crate) channels: u16,
    pub(crate) width: u8,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiResourceVendor {
    pub(crate) byte_length: u16,
    pub(crate) byte_data: [u8; 1usize],
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiResourceVendorTyped {
    pub(crate) byte_length: u16,
    pub(crate) uuid_subtype: u8,
    pub(crate) uuid: [u8; 16usize],
    pub(crate) byte_data: [u8; 1usize],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiResourceEndTag {
    pub(crate) checksum: u8,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiResourceMemory24 {
    pub(crate) write_protect: u8,
    pub(crate) minimum: u16,
    pub(crate) maximum: u16,
    pub(crate) alignment: u16,
    pub(crate) address_length: u16,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiResourceMemory32 {
    pub(crate) write_protect: u8,
    pub(crate) minimum: u32,
    pub(crate) maximum: u32,
    pub(crate) alignment: u32,
    pub(crate) address_length: u32,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiResourceFixedMemory32 {
    pub(crate) write_protect: u8,
    pub(crate) address: u32,
    pub(crate) address_length: u32,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiMemoryAttribute {
    pub(crate) write_protect: u8,
    pub(crate) caching: u8,
    pub(crate) range_type: u8,
    pub(crate) translation: u8,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiIoAttribute {
    pub(crate) range_type: u8,
    pub(crate) translation: u8,
    pub(crate) translation_type: u8,
    pub(crate) reserved1: u8,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) union FfiAcpiResourceAttribute {
    pub(crate) mem: FfiAcpiMemoryAttribute,
    pub(crate) io: FfiAcpiIoAttribute,
    pub(crate) type_specific: u8,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiResourceLabel {
    pub(crate) string_length: u16,
    pub(crate) string_ptr: *mut i8,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiResourceSource {
    pub(crate) index: u8,
    pub(crate) string_length: u16,
    pub(crate) string_ptr: *mut i8,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiAddress16Attribute {
    pub(crate) granularity: u16,
    pub(crate) minimum: u16,
    pub(crate) maximum: u16,
    pub(crate) translation_offset: u16,
    pub(crate) address_length: u16,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiAddress32Attribute {
    pub(crate) granularity: u32,
    pub(crate) minimum: u32,
    pub(crate) maximum: u32,
    pub(crate) translation_offset: u32,
    pub(crate) address_length: u32,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiAddress64Attribute {
    pub(crate) granularity: u64,
    pub(crate) minimum: u64,
    pub(crate) maximum: u64,
    pub(crate) translation_offset: u64,
    pub(crate) address_length: u64,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct FfiAcpiResourceAddress {
    pub(crate) resource_type: u8,
    pub(crate) producer_consumer: u8,
    pub(crate) decode: u8,
    pub(crate) min_address_fixed: u8,
    pub(crate) max_address_fixed: u8,
    pub(crate) info: FfiAcpiResourceAttribute,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct FfiAcpiResourceAddress16 {
    pub(crate) resource_type: u8,
    pub(crate) producer_consumer: u8,
    pub(crate) decode: u8,
    pub(crate) min_address_fixed: u8,
    pub(crate) max_address_fixed: u8,
    pub(crate) info: FfiAcpiResourceAttribute,
    pub(crate) address: FfiAcpiAddress16Attribute,
    pub(crate) resource_source: FfiAcpiResourceSource,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct FfiAcpiResourceAddress32 {
    pub(crate) resource_type: u8,
    pub(crate) producer_consumer: u8,
    pub(crate) decode: u8,
    pub(crate) min_address_fixed: u8,
    pub(crate) max_address_fixed: u8,
    pub(crate) info: FfiAcpiResourceAttribute,
    pub(crate) address: FfiAcpiAddress32Attribute,
    pub(crate) resource_source: FfiAcpiResourceSource,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct FfiAcpiResourceAddress64 {
    pub(crate) resource_type: u8,
    pub(crate) producer_consumer: u8,
    pub(crate) decode: u8,
    pub(crate) min_address_fixed: u8,
    pub(crate) max_address_fixed: u8,
    pub(crate) info: FfiAcpiResourceAttribute,
    pub(crate) address: FfiAcpiAddress64Attribute,
    pub(crate) resource_source: FfiAcpiResourceSource,
}
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub(crate) struct FfiAcpiResourceExtendedAddress64 {
    pub(crate) resource_type: u8,
    pub(crate) producer_consumer: u8,
    pub(crate) decode: u8,
    pub(crate) min_address_fixed: u8,
    pub(crate) max_address_fixed: u8,
    pub(crate) info: FfiAcpiResourceAttribute,
    pub(crate) revision_id: u8,
    pub(crate) address: FfiAcpiAddress64Attribute,
    pub(crate) type_specific: u64,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiResourceExtendedIrq {
    pub(crate) producer_consumer: u8,
    pub(crate) triggering: u8,
    pub(crate) polarity: u8,
    pub(crate) shareable: u8,
    pub(crate) wake_capable: u8,
    pub(crate) interrupt_count: u8,
    pub(crate) resource_source: FfiAcpiResourceSource,
    pub(crate) interrupts: [u32; 1usize],
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiResourceGenericRegister {
    pub(crate) space_id: u8,
    pub(crate) bit_width: u8,
    pub(crate) bit_offset: u8,
    pub(crate) access_size: u8,
    pub(crate) address: u64,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiResourceGpio {
    pub(crate) revision_id: u8,
    pub(crate) connection_type: u8,
    pub(crate) producer_consumer: u8,
    pub(crate) pin_config: u8,
    pub(crate) shareable: u8,
    pub(crate) wake_capable: u8,
    pub(crate) io_restriction: u8,
    pub(crate) triggering: u8,
    pub(crate) polarity: u8,
    pub(crate) drive_strength: u16,
    pub(crate) debounce_timeout: u16,
    pub(crate) pin_table_length: u16,
    pub(crate) vendor_length: u16,
    pub(crate) resource_source: FfiAcpiResourceSource,
    pub(crate) pin_table: *mut u16,
    pub(crate) vendor_data: *mut u8,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiResourceCommonSerialbus {
    pub(crate) revision_id: u8,
    pub(crate) resource_type: u8,
    pub(crate) producer_consumer: u8,
    pub(crate) slave_mode: u8,
    pub(crate) connection_sharing: u8,
    pub(crate) type_revision_id: u8,
    pub(crate) type_data_length: u16,
    pub(crate) vendor_length: u16,
    pub(crate) resource_source: FfiAcpiResourceSource,
    pub(crate) vendor_data: *mut u8,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiResourceI2cSerialbus {
    pub(crate) revision_id: u8,
    pub(crate) resource_type: u8,
    pub(crate) producer_consumer: u8,
    pub(crate) slave_mode: u8,
    pub(crate) connection_sharing: u8,
    pub(crate) type_revision_id: u8,
    pub(crate) type_data_length: u16,
    pub(crate) vendor_length: u16,
    pub(crate) resource_source: FfiAcpiResourceSource,
    pub(crate) vendor_data: *mut u8,
    pub(crate) access_mode: u8,
    pub(crate) slave_address: u16,
    pub(crate) connection_speed: u32,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiResourceSpiSerialbus {
    pub(crate) revision_id: u8,
    pub(crate) resource_type: u8,
    pub(crate) producer_consumer: u8,
    pub(crate) slave_mode: u8,
    pub(crate) connection_sharing: u8,
    pub(crate) type_revision_id: u8,
    pub(crate) type_data_length: u16,
    pub(crate) vendor_length: u16,
    pub(crate) resource_source: FfiAcpiResourceSource,
    pub(crate) vendor_data: *mut u8,
    pub(crate) wire_mode: u8,
    pub(crate) device_polarity: u8,
    pub(crate) data_bit_length: u8,
    pub(crate) clock_phase: u8,
    pub(crate) clock_polarity: u8,
    pub(crate) device_selection: u16,
    pub(crate) connection_speed: u32,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiResourceUartSerialbus {
    pub(crate) revision_id: u8,
    pub(crate) resource_type: u8,
    pub(crate) producer_consumer: u8,
    pub(crate) slave_mode: u8,
    pub(crate) connection_sharing: u8,
    pub(crate) type_revision_id: u8,
    pub(crate) type_data_length: u16,
    pub(crate) vendor_length: u16,
    pub(crate) resource_source: FfiAcpiResourceSource,
    pub(crate) vendor_data: *mut u8,
    pub(crate) endian: u8,
    pub(crate) data_bits: u8,
    pub(crate) stop_bits: u8,
    pub(crate) flow_control: u8,
    pub(crate) parity: u8,
    pub(crate) lines_enabled: u8,
    pub(crate) rx_fifo_size: u16,
    pub(crate) tx_fifo_size: u16,
    pub(crate) default_baud_rate: u32,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiResourceCsi2Serialbus {
    pub(crate) revision_id: u8,
    pub(crate) resource_type: u8,
    pub(crate) producer_consumer: u8,
    pub(crate) slave_mode: u8,
    pub(crate) connection_sharing: u8,
    pub(crate) type_revision_id: u8,
    pub(crate) type_data_length: u16,
    pub(crate) vendor_length: u16,
    pub(crate) resource_source: FfiAcpiResourceSource,
    pub(crate) vendor_data: *mut u8,
    pub(crate) local_port_instance: u8,
    pub(crate) phy_type: u8,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiResourcePinFunction {
    pub(crate) revision_id: u8,
    pub(crate) pin_config: u8,
    pub(crate) shareable: u8,
    pub(crate) function_number: u16,
    pub(crate) pin_table_length: u16,
    pub(crate) vendor_length: u16,
    pub(crate) resource_source: FfiAcpiResourceSource,
    pub(crate) pin_table: *mut u16,
    pub(crate) vendor_data: *mut u8,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiResourcePinConfig {
    pub(crate) revision_id: u8,
    pub(crate) producer_consumer: u8,
    pub(crate) shareable: u8,
    pub(crate) pin_config_type: u8,
    pub(crate) pin_config_value: u32,
    pub(crate) pin_table_length: u16,
    pub(crate) vendor_length: u16,
    pub(crate) resource_source: FfiAcpiResourceSource,
    pub(crate) pin_table: *mut u16,
    pub(crate) vendor_data: *mut u8,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiResourcePinGroup {
    pub(crate) revision_id: u8,
    pub(crate) producer_consumer: u8,
    pub(crate) pin_table_length: u16,
    pub(crate) vendor_length: u16,
    pub(crate) pin_table: *mut u16,
    pub(crate) resource_label: FfiAcpiResourceLabel,
    pub(crate) vendor_data: *mut u8,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiResourcePinGroupFunction {
    pub(crate) revision_id: u8,
    pub(crate) producer_consumer: u8,
    pub(crate) shareable: u8,
    pub(crate) function_number: u16,
    pub(crate) vendor_length: u16,
    pub(crate) resource_source: FfiAcpiResourceSource,
    pub(crate) resource_source_label: FfiAcpiResourceLabel,
    pub(crate) vendor_data: *mut u8,
}
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiResourcePinGroupConfig {
    pub(crate) revision_id: u8,
    pub(crate) producer_consumer: u8,
    pub(crate) shareable: u8,
    pub(crate) pin_config_type: u8,
    pub(crate) pin_config_value: u32,
    pub(crate) vendor_length: u16,
    pub(crate) resource_source: FfiAcpiResourceSource,
    pub(crate) resource_source_label: FfiAcpiResourceLabel,
    pub(crate) vendor_data: *mut u8,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) union FfiAcpiResourceData {
    pub(crate) irq: FfiAcpiResourceIrq,
    pub(crate) dma: FfiAcpiResourceDma,
    pub(crate) start_dpf: FfiAcpiResourceStartDependent,
    pub(crate) io: FfiAcpiResourceIo,
    pub(crate) fixed_io: FfiAcpiResourceFixedIo,
    pub(crate) fixed_dma: FfiAcpiResourceFixedDma,
    pub(crate) vendor: FfiAcpiResourceVendor,
    pub(crate) vendor_typed: FfiAcpiResourceVendorTyped,
    pub(crate) end_tag: FfiAcpiResourceEndTag,
    pub(crate) memory24: FfiAcpiResourceMemory24,
    pub(crate) memory32: FfiAcpiResourceMemory32,
    pub(crate) fixed_memory32: FfiAcpiResourceFixedMemory32,
    pub(crate) address16: FfiAcpiResourceAddress16,
    pub(crate) address32: FfiAcpiResourceAddress32,
    pub(crate) address64: FfiAcpiResourceAddress64,
    pub(crate) ext_address64: FfiAcpiResourceExtendedAddress64,
    pub(crate) extended_irq: FfiAcpiResourceExtendedIrq,
    pub(crate) generic_reg: FfiAcpiResourceGenericRegister,
    pub(crate) gpio: FfiAcpiResourceGpio,
    pub(crate) i2c_serial_bus: FfiAcpiResourceI2cSerialbus,
    pub(crate) spi_serial_bus: FfiAcpiResourceSpiSerialbus,
    pub(crate) uart_serial_bus: FfiAcpiResourceUartSerialbus,
    pub(crate) csi2_serial_bus: FfiAcpiResourceCsi2Serialbus,
    pub(crate) common_serial_bus: FfiAcpiResourceCommonSerialbus,
    pub(crate) pin_function: FfiAcpiResourcePinFunction,
    pub(crate) pin_config: FfiAcpiResourcePinConfig,
    pub(crate) pin_group: FfiAcpiResourcePinGroup,
    pub(crate) pin_group_function: FfiAcpiResourcePinGroupFunction,
    pub(crate) pin_group_config: FfiAcpiResourcePinGroupConfig,
    pub(crate) address: FfiAcpiResourceAddress,
}
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub(crate) struct FfiAcpiResource {
    pub(crate) resource_type: u32,
    pub(crate) length: u32,
    pub(crate) data: FfiAcpiResourceData,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiPciRoutingTable {
    pub(crate) length: u32,
    pub(crate) pin: u32,
    pub(crate) address: u64,
    pub(crate) source_index: u32,
    pub(crate) source: [i8; 4usize],
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)] // FFI type so variants are not explicitly constructed
pub(crate) enum FfiAcpiExecuteType {
    GlobalLockHandler = 0,
    NotifyHandler = 1,
    GpeHandler = 2,
    DebuggerMainThread = 3,
    DebuggerExecThread = 4,
    EcPollHandler = 5,
    EcBurstHandler = 6,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiSignalFatalInfo {
    pub(crate) resource_type: u32,
    pub(crate) code: u32,
    pub(crate) argument: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiFdeInfo {
    pub(crate) floppy0: u32,
    pub(crate) floppy1: u32,
    pub(crate) floppy2: u32,
    pub(crate) floppy3: u32,
    pub(crate) tape: u32,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiGrtInfo {
    pub(crate) year: u16,
    pub(crate) month: u8,
    pub(crate) day: u8,
    pub(crate) hour: u8,
    pub(crate) minute: u8,
    pub(crate) second: u8,
    pub(crate) valid: u8,
    pub(crate) milliseconds: u16,
    pub(crate) timezone: u16,
    pub(crate) daylight: u8,
    pub(crate) reserved: [u8; 3usize],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiGtmInfo {
    pub(crate) pio_speed0: u32,
    pub(crate) dma_speed0: u32,
    pub(crate) pio_speed1: u32,
    pub(crate) dma_speed1: u32,
    pub(crate) flags: u32,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct FfiAcpiPldInfo {
    pub(crate) revision: u8,
    pub(crate) ignore_color: u8,
    pub(crate) red: u8,
    pub(crate) green: u8,
    pub(crate) blue: u8,
    pub(crate) width: u16,
    pub(crate) height: u16,
    pub(crate) user_visible: u8,
    pub(crate) dock: u8,
    pub(crate) lid: u8,
    pub(crate) panel: u8,
    pub(crate) vertical_position: u8,
    pub(crate) horizontal_position: u8,
    pub(crate) shape: u8,
    pub(crate) group_orientation: u8,
    pub(crate) group_token: u8,
    pub(crate) group_position: u8,
    pub(crate) bay: u8,
    pub(crate) ejectable: u8,
    pub(crate) ospm_eject_required: u8,
    pub(crate) cabinet_number: u8,
    pub(crate) card_cage_number: u8,
    pub(crate) reference: u8,
    pub(crate) rotation: u8,
    pub(crate) order: u8,
    pub(crate) reserved: u8,
    pub(crate) vertical_offset: u16,
    pub(crate) horizontal_offset: u16,
}
