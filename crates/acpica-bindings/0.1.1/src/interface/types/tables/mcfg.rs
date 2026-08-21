//! The [`Mcfg`] type

use core::fmt::Debug;

use crate::{bindings::types::tables::misc::FfiAcpiTableMcfg, types::AcpiPhysicalAddress};

use super::AcpiTableHeader;

/// The MCFG table, for detecting PCI root buses
pub struct Mcfg<'a>(&'a FfiAcpiTableMcfg);

impl<'a> Mcfg<'a> {
    pub(crate) fn from_ffi(ffi_ptr: &'a FfiAcpiTableMcfg) -> Self {
        Self(ffi_ptr)
    }

    /// Gets the table's header
    #[must_use]
    pub fn header(&self) -> AcpiTableHeader {
        AcpiTableHeader::from_ffi(&self.0.header)
    }

    /// Gets an iterator over the table's records
    ///
    /// # Panics
    /// If the table is malformed - if the table (excluding header and reserved bytes) is not a multiple of 16 bytes in length
    pub fn records(&self) -> impl Iterator<Item = McfgRecord> + '_ {
        let content = &self.header().content()[8..];
        let chunks_exact = content.chunks_exact(16);
        assert_eq!(chunks_exact.remainder(), []);

        chunks_exact.map(|chunk| {
            let base_address = u64::from_le_bytes(chunk[..8].try_into().unwrap());
            let segment = u16::from_le_bytes(chunk[8..10].try_into().unwrap());
            let min_bus_number = chunk[10];
            let max_bus_number = chunk[11];

            McfgRecord {
                base_address: AcpiPhysicalAddress(base_address.try_into().unwrap()),
                segment,
                min_bus_number,
                max_bus_number,
            }
        })
    }
}

impl<'a> Debug for Mcfg<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Mcfg")
            .field("header", &self.header())
            .finish()
    }
}

/// A record in the [`Mcfg`], describing a single PCI root bus.
#[derive(Debug)]
pub struct McfgRecord {
    /// The physical address of the controller's configuration registers
    pub base_address: AcpiPhysicalAddress,
    /// The PCI segment number of this controller
    pub segment: u16,
    /// The lowest bus number which this controller links to
    pub min_bus_number: u8,
    /// The highest bus number which this controller links to
    pub max_bus_number: u8,
}
