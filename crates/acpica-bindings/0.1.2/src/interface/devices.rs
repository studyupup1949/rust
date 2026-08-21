//! Code to manage AML devices

use core::{
    ffi::{c_void, CStr},
    fmt::Debug,
    ptr::{addr_of_mut, null_mut},
};

use alloc::string::String;
use bitfield_struct::bitfield;

use crate::{
    bindings::{
        consts::{ACPI_FULL_PATHNAME, ACPI_NS_ROOT_PATH, ACPI_TYPE_DEVICE},
        functions::{
            AcpiGetHandle, AcpiGetIrqRoutingTable, AcpiGetName, AcpiGetObjectInfo,
            AcpiWalkNamespace,
        },
        types::{FfiAcpiBuffer, FfiAcpiDeviceInfo, FfiAcpiHandle, FfiAcpiPnpDeviceId},
    },
    status::{AcpiError, AcpiErrorAsStatusExt, AcpiStatus},
    types::{object::AcpiObjectType, AcpiPhysicalAddress},
    AcpicaOperation,
};

/// A handle to an object in the AML namespace
#[derive(Debug)]
pub struct AcpiHandle(FfiAcpiHandle);

/// Information about a device in the AML namespace
pub struct DeviceInfo<'a>(&'a FfiAcpiDeviceInfo);

#[bitfield(u16)]
struct AcpiDeviceInfoValidFields {
    #[bits(1)]
    _reserved: (),

    adr: bool,
    hid: bool,
    uid: bool,

    #[bits(1)]
    _reserved: (),

    cid: bool,
    cls: bool,

    #[bits(1)]
    _reserved: (),

    sxds: bool,
    sxws: bool,

    #[bits(6)]
    _reserved: (),
}

/// Flags relating to a [`DeviceInfo`] struct
#[bitfield(u8)]
pub struct DeviceInfoFlags {
    pci_root_bridge: bool,

    #[bits(7)]
    _reserved: (),
}

impl<'a> DeviceInfo<'a> {
    pub(crate) fn from_ffi(ffi_ptr: &'a FfiAcpiDeviceInfo) -> Self {
        Self(ffi_ptr)
    }

    #[must_use]
    fn valid(&self) -> AcpiDeviceInfoValidFields {
        AcpiDeviceInfoValidFields::from(self.0.valid)
    }

    /// Gets the device's name
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn name(&self) -> &str {
        core::str::from_utf8(&self.0.name).expect("Object name should have been valid utf-8")
    }

    /// Gets the type of the object in the AML namespace
    #[must_use]
    pub fn object_type(&self) -> AcpiObjectType {
        AcpiObjectType::from_type_id(self.0.object_type)
    }

    /// Gets the number of parameters, if the object is a method.
    #[must_use]
    pub fn param_count(&self) -> u8 {
        self.0.param_count
    }

    /// Gets the device's flags
    #[must_use]
    pub fn flags(&self) -> DeviceInfoFlags {
        DeviceInfoFlags::from(self.0.flags)
    }

    /// TODO: What do these numbers mean?
    /// ACPICA source says '`_SxD` values: `0xFF` indicates not valid'
    #[must_use]
    pub fn highest_dstates(&self) -> [u8; 4usize] {
        self.0.highest_dstates
    }

    /// TODO: What do these numbers mean?
    /// ACPICA source says '`_SxW` values: `0xFF` indicates not valid'    
    #[must_use]
    pub fn lowest_dstates(&self) -> [u8; 5usize] {
        self.0.lowest_dstates
    }

    /// The address of the device's memory mapped registers
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn address(&self) -> Option<AcpiPhysicalAddress> {
        if self.0.address == 0 {
            None
        } else {
            Some(AcpiPhysicalAddress(self.0.address.try_into().unwrap()))
        }
    }

    /// A string identifying the device's hardware, for instance a chip ID
    #[must_use]
    pub fn hardware_id(&self) -> Option<&str> {
        // SAFETY: The device ID was provided by ACPICA so it points to valid memory
        unsafe { pnp_device_to_str(&self.0.hardware_id) }
    }

    /// A string uniquely identifying the device
    #[must_use]
    pub fn unique_id(&self) -> Option<&str> {
        // SAFETY: The device ID was provided by ACPICA so it points to valid memory
        unsafe { pnp_device_to_str(&self.0.unique_id) }
    }

    /// TODO: What's this?
    #[must_use]
    pub fn class_code(&self) -> Option<&str> {
        // SAFETY: The device ID was provided by ACPICA so it points to valid memory
        unsafe { pnp_device_to_str(&self.0.class_code) }
    }

    /// A list of IDs which are compatible with this device
    #[allow(clippy::missing_panics_doc)]
    pub fn compatible_id_list(&self) -> impl Iterator<Item = &str> {
        // SAFETY: TODO
        let arr = unsafe {
            self.0
                .compatible_id_list
                .ids
                .as_slice(self.0.compatible_id_list.count.try_into().unwrap())
        };

        arr.iter().filter_map(|device| 
                // SAFETY: TODO
                unsafe { pnp_device_to_str(device) })
    }
}

impl<'a> Debug for DeviceInfo<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DeviceInfo")
            .field("name", &self.name())
            .field("object_type", &self.object_type())
            .field("param_count", &self.param_count())
            .field("valid", &self.valid())
            .field("flags", &self.flags())
            .field("highest_dstates", &self.highest_dstates())
            .field("lowest_dstates", &self.lowest_dstates())
            .field("address", &self.address())
            .field("hardware_id", &self.hardware_id())
            .field("unique_id", &self.unique_id())
            .field("class_code", &self.class_code())
            // .field("compatible_id_list", &self.compatible_id_list())
            // .field("compatible_id_list", )
            .finish()
    }
}

unsafe fn pnp_device_to_str(device: &FfiAcpiPnpDeviceId) -> Option<&str> {
    if device.string.is_null() {
        None
    } else {
        let bytes =
        // SAFETY: TODO
            unsafe { core::slice::from_raw_parts(device.string.cast(), device.length as _) };
        let bytes = bytes.split(|&v| v == 0).next().unwrap();
        let s = core::str::from_utf8(bytes).expect("PNP Device ID should have been valid utf-8");
        Some(s)
    }
}

/// One of the four PCI interrupt pins
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcpiPciPin {
    /// The `INTA#` interrupt pin
    IntA,
    /// The `INTB#` interrupt pin
    IntB,
    /// The `INTC#` interrupt pin
    IntC,
    /// The `INTD#` interrupt pin
    IntD,
}

impl AcpiPciPin {
    const fn from_pin_number(pin_number: u32) -> Self {
        match pin_number {
            0 => Self::IntA,
            1 => Self::IntB,
            2 => Self::IntC,
            3 => Self::IntD,
            _ => panic!("Invalid pin number"),
        }
    }
}

/// An entry representing the IRQ mapping for a device
#[derive(Debug, Clone, Copy)]
pub struct AcpiPciRoutingTableEntry<'a> {
    /// Which PCI interrupt pin the device is connected to
    pub pin: AcpiPciPin,
    /// An address relative to the bus the device is on.
    /// The device number is specified
    pub device_number: u16,
    /// The index of a type of interrupt generated by the device
    pub source_index: u32,
    /// A string
    pub source: &'a str,
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum AcpiIrqRoutingFunction {
//     All,
//     Specific(u16),
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub struct AcpiIrqRoutingAddress {
//     device: u16,
//     function: AcpiIrqRoutingFunction,
// }

// impl AcpiIrqRoutingAddress {
//     /// Check whether the the address matches the given device and function numbers
//     #[must_use]
//     pub fn matches(&self, device: u16, function: u16) -> bool {
//         if self.device == device {
//             match self.function {
//                 AcpiIrqRoutingFunction::All => true,
//                 AcpiIrqRoutingFunction::Specific(f) => function == f,
//             }
//         }
//         else {
//             false
//         }
//     }

//     fn from_u32(value: u32) -> Self {
//         let device = (value >> 16) as u16;
//         #[allow(clippy::cast_possible_truncation)]
//         let function = value as u16;

//         match function {
//             0xFFFF => Self { device, function: AcpiIrqRoutingFunction::All },
//             _ => Self { device, function: AcpiIrqRoutingFunction::Specific(function) }
//         }
//     }
// }

impl AcpiHandle {
    /// Gets the object's path in the AML namespace
    #[allow(clippy::missing_panics_doc)]
    pub fn path(&self) -> Result<String, AcpiError> {
        let (r, s) =
        // SAFETY: The arguments to this function are correct
            FfiAcpiBuffer::allocate(|b| unsafe { AcpiGetName(self.0, ACPI_FULL_PATHNAME, b) });

        let mut s = s.unwrap();
        r.as_result()?;

        if let Some(i) = s.iter().rposition(|x| *x != 0) {
            let new_len = i + 1;
            s.truncate(new_len);
        };

        Ok(String::from_utf8(s).expect("Path should have been valid utf-8"))
    }

    /// Gets the object's device info
    #[allow(clippy::missing_panics_doc)]
    pub fn get_info(&self) -> Result<DeviceInfo, AcpiError> {
        let mut ptr = null_mut();

        // SAFETY: The arguments to this function are correct
        let r = unsafe { AcpiGetObjectInfo(self.0, &mut ptr) };
        r.as_result()?;

        assert!(!ptr.is_null());

        // SAFETY: `ptr` is valid
        unsafe { Ok(DeviceInfo::from_ffi(&*ptr)) }
    }

    /// If the device is a PCI bus, this method returns the IRQ mapping for the devices on that bus.
    /// Note that this is only for pin-based interrupts, and can be ignored if using MSI.
    #[allow(clippy::missing_panics_doc)]
    pub fn get_irq_routing_table(
        &self,
    ) -> Result<Option<impl Iterator<Item = AcpiPciRoutingTableEntry>>, AcpiError> {
        // SAFETY: The arguments to this function are correct
        let (r, buffer) = FfiAcpiBuffer::allocate(|b| unsafe { AcpiGetIrqRoutingTable(self.0, b) });

        match r.as_result() {
            Ok(()) => (),
            Err(AcpiError::NotFound) => return Ok(None),
            Err(_) => r.as_result().unwrap(),
        }

        r.as_result()?;

        let buffer = buffer.unwrap();
        let mut i = 0;

        Ok(Some(core::iter::from_fn(move || {
            let length = u32::from_ne_bytes(buffer[i..i + 4].try_into().unwrap());
            let pin = u32::from_ne_bytes(buffer[i + 4..i + 8].try_into().unwrap());
            let address = u64::from_ne_bytes(buffer[i + 8..i + 16].try_into().unwrap());
            let source_index = u32::from_ne_bytes(buffer[i + 16..i + 20].try_into().unwrap());

            if length == 0 {
                None
            } else {
                // SAFETY: This property can be dereferenced as a null-terminated string
                let source = unsafe { CStr::from_ptr(buffer[i + 24..].as_ptr().offset(4).cast()) };

                i += length as usize;

                #[allow(clippy::cast_possible_truncation)]
                let device_number = {
                    // ACPI spec says this this word needs to be 0xFFFF (meaning all functions)
                    assert_eq!(
                        address as u16, 0xFFFF,
                        "Function number of PRT address field should be 0xFFFF"
                    );
                    (address >> 16) as u16
                };

                Some(AcpiPciRoutingTableEntry {
                    pin: AcpiPciPin::from_pin_number(pin),
                    device_number,
                    source_index,
                    source: source.to_str().unwrap(),
                })
            }
        })))
    }
}

struct ScanContext<'a, T, F>
where
    F: Fn(AcpiHandle, u32) -> Option<T>,
{
    function: &'a F,
    result: &'a mut Option<T>,
}

unsafe extern "C" fn scan_single_device<T, F>(
    handle: FfiAcpiHandle,
    nesting_level: u32,
    context_ptr: *mut c_void,
    return_value: *mut *mut c_void,
) -> AcpiStatus
where
    F: Fn(AcpiHandle, u32) -> Option<T>,
{
    if context_ptr.is_null() || return_value.is_null() {
        return AcpiError::BadParameter.to_acpi_status();
    }

    // SAFETY: `context` was passed by `scan_devices` so it is valid for this type
    let context: &mut ScanContext<T, F> = unsafe { &mut *context_ptr.cast() };

    let handle = AcpiHandle(handle);

    let v = (context.function)(handle, nesting_level);

    if let Some(result) = v {
        *context.result = Some(result);
        // SAFETY:
        unsafe { *return_value = context_ptr }
    }

    AcpiStatus::OK
}

// ACPICA needs a callback for after a device has been recursively scanned
unsafe extern "C" fn scan_single_device_ascending(
    _handle: FfiAcpiHandle,
    _nesting_level: u32,
    _context: *mut c_void,
    _return_value: *mut *mut c_void,
) -> AcpiStatus {
    AcpiStatus::OK
}

impl AcpicaOperation<true, true, true, true> {
    /// Calls a callback for each device in the AML namespace
    #[allow(clippy::missing_panics_doc)]
    pub fn scan_devices<F: Fn(AcpiHandle, u32) -> Option<T>, T>(&self, function: F) -> Option<T> {
        let mut out: Option<T> = None;
        let mut out_ptr: *mut T = null_mut();

        let mut handle: FfiAcpiHandle = null_mut();

        // SAFETY: These arguments are valid
        unsafe {
            let mut root_path_copy = *ACPI_NS_ROOT_PATH;

            AcpiGetHandle(null_mut(), root_path_copy.as_mut_ptr().cast(), &mut handle)
        };

        let mut context = ScanContext {
            function: &function,
            result: &mut out,
        };

        // SAFETY: The arguments are correct
        let return_value = unsafe {
            AcpiWalkNamespace(
                ACPI_TYPE_DEVICE,
                handle,
                u32::MAX,
                scan_single_device::<T, F>,
                scan_single_device_ascending,
                core::ptr::addr_of_mut!(context).cast(),
                addr_of_mut!(out_ptr).cast(),
            )
        };

        // AcpiWalkNamespace only returns BadParameter as an error, so if this fails it is this library's fault.
        return_value.as_result().unwrap();

        if let Some(ref o) = out {
            assert_eq!(o as *const T, out_ptr.cast());
        } else {
            assert!(out_ptr.is_null());
        }

        out
    }
}
