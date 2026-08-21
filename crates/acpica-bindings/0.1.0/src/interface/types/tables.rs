//! Contains the types for specific ACPI tables.

use core::fmt::Debug;

use crate::bindings::types::tables::FfiAcpiTableHeader;

pub mod fadt;
pub mod madt;
pub mod mcfg;
pub mod uefi;

pub use madt::Madt;
pub use uefi::Uefi;

/// Master ACPI Table Header. This common header is used by all ACPI tables
/// except the RSDP and FACS.
///
/// For more info on individual fields, see [the ACPI spec]
///
/// [the ACPI spec]: https://uefi.org/specs/ACPI/6.5/05_ACPI_Software_Programming_Model.html#system-description-table-header
pub struct AcpiTableHeader<'a>(&'a FfiAcpiTableHeader);

impl<'a> AcpiTableHeader<'a> {
    pub(crate) fn from_ffi(ffi_header: &'a FfiAcpiTableHeader) -> Self {
        Self(ffi_header)
    }

    pub(crate) fn as_ffi(&self) -> &FfiAcpiTableHeader {
        self.0
    }
}

impl<'a> Debug for AcpiTableHeader<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AcpiTableHeader")
            .field("signature", &self.signature())
            .field("length", &self.length())
            .field("revision", &self.revision())
            .field("checksum", &self.checksum())
            .field("oem_id", &self.oem_id())
            .field("oem_table_id", &self.oem_table_id())
            .field("oem_revision", &self.oem_revision())
            .field("asl_compiler_id", &self.creator_id())
            .field("asl_compiler_revision", &self.creator_revision())
            .finish()
    }
}

impl<'a> AcpiTableHeader<'a> {
    /// The table's signature. This is a 4-byte ASCII string indicating what kind of data the table contains.
    #[must_use]
    #[allow(clippy::missing_panics_doc)] // ACPICA already checks that the string is ASCII
    pub fn signature(&self) -> &str {
        core::str::from_utf8(&self.0.signature).unwrap()
    }
    /// The length of the table in bytes
    #[must_use]
    pub fn length(&self) -> u32 {
        self.0.length
    }
    /// The revision number of the table
    #[must_use]
    pub fn revision(&self) -> u8 {
        self.0.revision
    }
    /// The table's checksum. This byte is set to make all the bytes in the table sum to 0 (mod 0xff)
    #[must_use]
    pub fn checksum(&self) -> u8 {
        self.0.checksum
    }
    /// A string identifying the OEM
    #[must_use]
    #[allow(clippy::missing_panics_doc)] // ACPICA already checks that the string is ASCII
    pub fn oem_id(&self) -> &str {
        core::str::from_utf8(&self.0.oem_id).unwrap()
    }
    /// An OEM-provided string to identify this particular instance of a table
    #[must_use]
    #[allow(clippy::missing_panics_doc)] // ACPICA already checks that the string is ASCII
    pub fn oem_table_id(&self) -> &str {
        core::str::from_utf8(&self.0.oem_table_id).unwrap()
    }
    /// The table's revision number (larger is newer)
    #[must_use]
    pub fn oem_revision(&self) -> u32 {
        self.0.oem_revision
    }
    /// An ASCII string identifying the tool used to create the table.
    /// If the table contains AML code, this identifies the ASL compiler.
    #[must_use]
    #[allow(clippy::missing_panics_doc)] // ACPICA already checks that the string is ASCII
    pub fn creator_id(&self) -> &str {
        core::str::from_utf8(&self.0.asl_compiler_id).unwrap()
    }
    /// The revision number of the tool used to create the table (larger is newer)
    /// If the table contains AML code, this is the revision number of the ASL compiler.
    #[must_use]
    pub fn creator_revision(&self) -> u32 {
        self.0.asl_compiler_revision
    }

    /// Gets the rest of the table (all the table's data apart from the header) as a byte slice
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn content(&self) -> &'a [u8] {
        // SAFETY: The rest of the table is the data, starting at the byte offset `data_offset` from the start of the table
        unsafe {
            let whole_table = core::slice::from_raw_parts(
                (self.0 as *const FfiAcpiTableHeader).cast::<u8>(),
                self.length().try_into().unwrap(),
            );

            &whole_table[36..]
        }
    }
}
