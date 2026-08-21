//! The [`Madt`] type

use core::{ffi::CStr, fmt::Debug};

use bitfield_struct::bitfield;
use log::debug;

use crate::{bindings::types::tables::madt::FfiAcpiTableMadt, types::AcpiPhysicalAddress};

use super::AcpiTableHeader;

/// The `MADT` ACPI table
pub struct Madt<'a>(&'a FfiAcpiTableMadt);

impl<'a> Madt<'a> {
    pub(crate) fn from_ffi(ffi_pointer: &'a FfiAcpiTableMadt) -> Self {
        Self(ffi_pointer)
    }

    /// Gets the physical address of the system's local APIC
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn local_apic_address(&self) -> AcpiPhysicalAddress {
        AcpiPhysicalAddress(self.0.address.try_into().unwrap())
    }

    /// Gets the table's header
    #[must_use]
    pub fn header(&self) -> AcpiTableHeader {
        AcpiTableHeader::from_ffi(&self.0.header)
    }

    /// Whether the system also has a PC-AT-compatible dual-8259 setup
    #[must_use]
    pub fn pcat_compatible(&self) -> bool {
        MadtFlags::from(self.0.flags).pcat_compatible()
    }

    /// Gets an iterator over the records in the table
    pub fn records(&'a self) -> impl Iterator<Item = MadtRecord> + 'a {
        // This struct is unsound to construct except from a valid MADT, so there has to be a record here.
        let content = self.header().content();

        let mut records_rest = &content[8..];

        core::iter::from_fn(move || {
            if records_rest.is_empty() {
                None
            } else {
                let (record, rest) = MadtRecord::read(records_rest);
                records_rest = rest;

                Some(record)
            }
        })
    }
}

/// An error occurring when attempting to fetch the IO APIC address from the
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum IoApicAddressFetchError {
    /// There was no [`IoApic`] record in the MADT
    ///
    /// [`IoApic`]: MadtRecord::IoApic
    NoRecord,
}

impl<'a> Madt<'a> {
    /// Returns the physical address of the IO APIC
    pub fn io_apic_address(&self) -> Result<AcpiPhysicalAddress, IoApicAddressFetchError> {
        let mut records = self.records();

        let address = records
            .find_map(|record| {
                if let MadtRecord::IoApic { address, .. } = record {
                    Some(address)
                } else {
                    None
                }
            })
            .ok_or(IoApicAddressFetchError::NoRecord)?;

        // If there are more IO APICs after the first one
        if records.any(|record| matches!(&record, MadtRecord::IoApic { .. })) {
            todo!("Multiple I/O APICs");
        }

        Ok(AcpiPhysicalAddress(address.try_into().unwrap()))
    }
}

impl<'a> Debug for Madt<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AcpiTableMadt")
            .field("header", &self.header())
            .field("local_apic_address", &self.local_apic_address())
            .field("pcat_compatible", &self.pcat_compatible())
            .finish()
    }
}

/// Flags for the [`ProcessorLocalApic`][MadtRecord::ProcessorLocalApic] record type
#[bitfield(u32)]
pub struct ApicFlags {
    /// Whether the processor is ready for use
    enabled: bool,
    /// Whether the processor can be turned on by the OS, if it is not already on.
    online_capable: bool,

    #[bits(30)]
    _reserved: (),
}

/// Under what condition the interrupt is triggered
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterruptPolarity {
    /// Conforms to the specification of the bus
    ConformsToBus,
    /// Active high - the interrupt is triggered while the line is on
    ActiveHigh,
    /// Reserved
    Reserved,
    /// Active low - the interrupt is triggered while the line is off
    ActiveLow,
}

impl InterruptPolarity {
    /// Constructs an [`InterruptPolarity`] from its bit representation
    const fn from_bits(bits: u16) -> Self {
        match bits {
            0 => Self::ConformsToBus,
            1 => Self::ActiveHigh,
            2 => Self::Reserved,
            3 => Self::ActiveLow,
            _ => unreachable!(),
        }
    }

    /// Converts an [`InterruptPolarity`] into its bit representation
    const fn into_bits(self) -> u16 {
        match self {
            Self::ConformsToBus => 0,
            Self::ActiveHigh => 1,
            Self::Reserved => 2,
            Self::ActiveLow => 3,
        }
    }
}

/// How often the interrupt is triggered while the condition is met
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterruptTriggerMode {
    /// Conforms to the specification of the bus
    ConformsToBus,
    /// The interrupt is triggered once when the condition becomes true
    EdgeTriggered,
    /// Reserved
    Reserved,
    /// The interrupt is triggered continuously while the condition is true
    LevelTriggered,
}

impl InterruptTriggerMode {
    /// Constructs an [`InterruptTriggerMode`] from its bit representation
    const fn from_bits(bits: u16) -> Self {
        match bits {
            0 => Self::ConformsToBus,
            1 => Self::EdgeTriggered,
            2 => Self::Reserved,
            3 => Self::LevelTriggered,
            _ => unreachable!(),
        }
    }

    /// Converts an [`InterruptTriggerMode`] into its bit representation
    const fn into_bits(self) -> u16 {
        match self {
            Self::ConformsToBus => 0,
            Self::EdgeTriggered => 1,
            Self::Reserved => 2,
            Self::LevelTriggered => 3,
        }
    }
}

/// TODO: This is called MPS INTI flags in the spec, what does that stand for, rename this struct?
#[bitfield(u16)]
pub struct InterruptVectorFlags {
    #[bits(2)]
    polarity: InterruptPolarity,
    #[bits(2)]
    trigger_mode: InterruptTriggerMode,

    #[bits(12)]
    _reserved: (),
}

/// A record in the [`Madt`]
#[derive(Debug)]
pub enum MadtRecord<'a> {
    /// Record declaring the presence of a processor and associated APIC
    ProcessorLocalApic {
        /// An ID which the OS uses to match this record to an object in the DSDT (TODO: link)
        processor_id: u8,
        /// The processor's local APIC ID
        apic_id: u8,
        /// Flags about the processor and APIC
        flags: ApicFlags,
    },
    /// Record declaring the presence of an I/O APIC, which is accessible by all processors and handles I/O interrupts
    /// such as mouse and keyboard events
    IoApic {
        /// The ID of the I/O APIC
        id: u8,
        #[doc(hidden)]
        reserved0: u8,
        /// The physical address of the I/O APIC's registers
        address: u32,
        /// The global system interrupt number where this I/O APIC’s interrupt inputs start.
        /// The number of interrupt inputs is determined by the I/O APIC’s Max Redir Entry register. (TODO: link)
        global_system_interrupt_base: u32,
    },
    /// Record describing a variation from the default in the I/O APIC's IRQ mappings.
    IoApicInterruptSourceOverride {
        /// Should always be 0
        bus_source: u8,
        /// The source IRQ e.g. 0 for the timer interrupt
        irq_source: u8,
        /// What IRQ the interrupt will trigger on the I/O APIC
        global_system_interrupt: u32,
        /// Under what conditions the interrupt is triggered
        flags: InterruptVectorFlags,
    },
    /// Record specifying which I/O interrupt inputs should be enabled as non-maskable.
    /// Non-maskable interrupts are not available for use by devices.
    IoApicNonMaskableInterruptSource {
        /// Under what conditions the interrupt is triggered
        flags: InterruptVectorFlags,
        /// The global system interrupt which this NMI will signal
        global_system_interrupt: u32,
    },
    /// Record describing how non-maskable interrupts are connected to local APICs.
    LocalApicNonMaskableInterrupts {
        /// An ID which the OS uses to match this record to an object in the DSDT (TODO: link)
        processor_id: u8,
        /// Under what conditions the interrupt is triggered
        flags: InterruptVectorFlags,
        /// Local APIC interrupt input (LINTn) to which the NMI is connected
        lint: u8,
    },
    /// An override of the 32-bit [`local_apic_address`][Madt::local_apic_address] field
    /// to extend the address to 64 bits.
    LocalApicAddressOverride {
        #[doc(hidden)]
        reserved0: u16,
        /// The new address
        address: u64,
    },
    /// Similar to [`IoApic`][MadtRecord::IoApic] but for an I/O SAPIC.
    /// This data should be used instead of the data in [`IoApic`][MadtRecord::IoApic] if both records are
    /// present with the same `id`.
    IoSapic {
        /// The ID of the I/O SAPIC
        id: u8,
        #[doc(hidden)]
        reserved0: u8,
        /// The global system interrupt number where this I/O APIC’s interrupt inputs start.
        /// The number of interrupt inputs is determined by the I/O APIC’s Max Redir Entry register. (TODO: link)
        global_system_interrupt_base: u32,
        /// The physical address of the I/O SAPIC's registers
        address: u64,
    },
    /// Similar to [`ProcessorLocalApic`][MadtRecord::ProcessorLocalApic] but for an I/O SAPIC.
    /// This data should be used instead of the data in [`ProcessorLocalApic`][MadtRecord::ProcessorLocalApic] if both records are
    /// present with the same `id`.
    LocalSapic {
        /// The ID of the I/O SAPIC
        id: u8,
        /// The processor's local SAPIC ID
        local_sapic_id: u8,
        /// The processor's local SAPIC EID
        local_sapic_eid: u8,
        #[doc(hidden)]
        reserved0: [u8; 3],
        /// Local SAPIC flags
        flags: ApicFlags,
        /// A value used to match this record to an object in the DSDT (TODO: link)
        acpi_processor_uid_value: u32,
        /// A string used to match this record to an object in the DSDT (TODO: link)
        acpi_processor_uid_string: &'a str,
    },
    /// Record which communicates which I/O SAPIC interrupt inputs are connected to the platform interrupt sources.
    PlatformInterruptSources,
    /// Similar to [`ProcessorLocalApic`][MadtRecord::ProcessorLocalApic] but for an X2APIC.
    ProcessorLocalX2Apic {
        #[doc(hidden)]
        reserved: u16,
        /// The ID of the local X2APIC
        id: u32,
        /// Flags for the X2APIC
        flags: ApicFlags,
        /// A value used to match this record to an object in the DSDT (TODO: link)
        acpi_id: u32,
    },
    /// TODO
    LocalX2ApicNonMaskableInterrupt,
    /// TODO
    GicCpuInterface,
    /// TODO
    GicDistributor,
    /// TODO
    GicMsiFrame,
    /// TODO
    GicRedistributor,
    /// TODO
    GicInterruptTranslationService,
    /// TODO
    MultiprocessorWakeup,
    /// TODO
    CorePic,
    /// TODO
    LegacyIoPic,
    /// TODO
    HyperTransportPic,
    /// TODO
    ExtendIoPic,
    /// TODO
    MsiPic,
    /// TODO
    BridgeIoPic,
    /// TODO
    LowPinCountPic,

    /// Reserved, OEM-specified, or unknown record type
    Reserved,
}

impl<'a> MadtRecord<'a> {
    /// Reads the record from the start of the given byte slice, returning the record along with the rest of the byte slice
    ///
    /// # Safety
    /// `read_type` must be sound to transmute from the equivalent byte slice
    fn read(from: &'a [u8]) -> (Self, &'a [u8]) {
        macro_rules! read {
            ($read_type: ty, $from: expr) => {{
                const SIZE_OF_TYPE: usize = ::core::mem::size_of::<$read_type>();

                let from: &mut &[u8] = &mut $from;

                let (arr, rest) = from.split_at(SIZE_OF_TYPE);
                let arr: [u8; SIZE_OF_TYPE] = arr.try_into().unwrap();

                *from = rest;

                #[allow(clippy::useless_transmute)] // This macro can be used with [u8; _] input type, which would make this transmute be to the same type
                // SAFETY: `read` type is sound to transmute
                unsafe { core::mem::transmute::<_, $read_type>(arr) }
            }};
        }

        // The first byte of a record is always a number indicating the type of record
        let variant = from[0];
        // The second byte is always the length of the record in bytes
        let length: u8 = from[1];

        let (from, rest) = from.split_at(length as usize);
        // The first two bytes are the variant and length, and aren't part of the data
        let mut from = &from[2..];

        let record = match variant {
            0x00 => Self::ProcessorLocalApic {
                processor_id: read!(u8, from),
                apic_id: read!(u8, from),
                flags: read!(ApicFlags, from),
            },
            0x01 => Self::IoApic {
                id: read!(u8, from),
                reserved0: read!(u8, from),
                address: read!(u32, from),
                global_system_interrupt_base: read!(u32, from),
            },
            0x02 => Self::IoApicInterruptSourceOverride {
                bus_source: read!(u8, from),
                irq_source: read!(u8, from),
                global_system_interrupt: read!(u32, from),
                flags: read!(InterruptVectorFlags, from),
            },
            0x03 => Self::IoApicNonMaskableInterruptSource {
                flags: read!(InterruptVectorFlags, from),
                global_system_interrupt: read!(u32, from),
            },
            0x04 => Self::LocalApicNonMaskableInterrupts {
                processor_id: read!(u8, from),
                flags: read!(InterruptVectorFlags, from),
                lint: read!(u8, from),
            },
            0x05 => Self::LocalApicAddressOverride {
                reserved0: read!(u16, from),
                address: read!(u64, from),
            },
            0x06 => Self::IoSapic {
                id: read!(u8, from),
                reserved0: read!(u8, from),
                global_system_interrupt_base: read!(u32, from),
                address: read!(u64, from),
            },
            0x07 => Self::LocalSapic {
                id: read!(u8, from),
                local_sapic_id: read!(u8, from),
                local_sapic_eid: read!(u8, from),
                reserved0: read!([u8; 3], from),
                flags: read!(ApicFlags, from),
                acpi_processor_uid_value: read!(u32, from),
                acpi_processor_uid_string: {
                    let cstr = CStr::from_bytes_until_nul(from)
                        .expect("Should have found null byte within record");
                    cstr.to_str()
                        .expect("Processor UID string should have been valid utf-8")
                },
            },
            0x08 => Self::PlatformInterruptSources,
            0x09 => Self::ProcessorLocalX2Apic {
                reserved: read!(u16, from),
                id: read!(u32, from),
                flags: read!(ApicFlags, from),
                acpi_id: read!(u32, from),
            },
            0x0A => Self::LocalX2ApicNonMaskableInterrupt,
            0x0B => Self::GicCpuInterface,
            0x0C => Self::GicDistributor,
            0x0D => Self::GicMsiFrame,
            0x0E => Self::GicRedistributor,
            0x0F => Self::GicInterruptTranslationService,
            0x10 => Self::MultiprocessorWakeup,
            0x11 => Self::CorePic,
            0x12 => Self::LegacyIoPic,
            0x13 => Self::HyperTransportPic,
            0x14 => Self::ExtendIoPic,
            0x15 => Self::MsiPic,
            0x16 => Self::BridgeIoPic,
            0x17 => Self::LowPinCountPic,

            _ => Self::Reserved,
        };

        (record, rest)
    }
}

/// The flags present on an MADT
#[bitfield(u32)]
pub struct MadtFlags {
    /// Whether also has a PC-AT-compatible dual-8259 setup.
    /// The 8259 vectors must be disabled (that is, masked) when enabling the ACPI APIC operation.
    pcat_compatible: bool,

    #[bits(31)]
    _reserved: (),
}
