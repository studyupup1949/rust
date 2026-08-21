//! The [`Uefi`] type for reading the `UEFI` table

use core::fmt::Debug;

use crate::bindings::types::tables::misc::FfiAcpiTableUefi;

use super::AcpiTableHeader;

/// The `UEFI` ACPI table, which
pub struct Uefi<'a>(&'a FfiAcpiTableUefi);

impl<'a> Uefi<'a> {
    pub(crate) fn from_ffi(ffi_table: &'a FfiAcpiTableUefi) -> Self {
        Self(ffi_table)
    }

    /// Gets the table's header
    #[must_use]
    pub fn header(&self) -> AcpiTableHeader {
        AcpiTableHeader::from_ffi(&self.0.header)
    }

    /// Gets the UUID identifier for this table.
    #[must_use]
    pub fn identifier(&self) -> [u8; 16] {
        self.0.identifier
    }

    /// Gets the offset in bytes of the start of the table's data from the start of the table.
    #[must_use]
    pub fn data_offset(&self) -> u16 {
        self.0.data_offset
    }

    /// Gets the table's data as an array of bytes
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.header().content()[self.data_offset() as usize..]
    }
}

impl<'a> Debug for Uefi<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AcpiTableUefi")
            .field("header", &self.header())
            .field("identifier", &self.identifier())
            .field("data_offset", &self.data_offset())
            .finish()
    }
}
