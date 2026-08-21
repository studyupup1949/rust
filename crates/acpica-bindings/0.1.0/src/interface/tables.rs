use crate::{
    bindings::{
        functions::{AcpiGetTable, AcpiGetTableByIndex},
        types::tables::FfiAcpiTableHeader,
    },
    status::AcpiError,
    types::tables::{fadt::Fadt, mcfg::Mcfg, AcpiTableHeader, Madt, Uefi},
    AcpicaOperation,
};

impl<const TL: bool, const E: bool, const I: bool> AcpicaOperation<true, TL, E, I> {
    fn get_tables_of_type(&self, signature: [u8; 4]) -> impl Iterator<Item = AcpiTableHeader> {
        let mut i = 1;

        core::iter::from_fn(move || {
            let mut table = core::ptr::null_mut();
            // Move the signature into an array for borrow checking reasons
            // And so that if ACPICA mutates this string it's not UB
            let mut signature = signature;

            // SAFETY: The signature is valid
            let r = unsafe { AcpiGetTable(signature.as_mut_ptr().cast(), i, &mut table) };

            i += 1;

            match r.as_result() {
                Ok(()) => {
                    assert!(!table.is_null());
                    // SAFETY: The returned pointer is valid
                    unsafe { Some(AcpiTableHeader::from_ffi(&*table.cast_const())) }
                }
                Err(AcpiError::BadParameter) => panic!("ACPICA reported bad parameter"),
                Err(_) => None,
            }
        })
    }

    /// Gets a table by its signature, if it is present on the system.
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn table(&self, signature: [u8; 4]) -> Option<AcpiTableHeader> {
        let mut table = core::ptr::null_mut();
        // Move the signature into an array for borrow checking reasons
        // And so that if ACPICA mutates this string it's not UB
        let mut signature = signature;

        // SAFETY: The signature is valid
        let r = unsafe { AcpiGetTable(signature.as_mut_ptr().cast(), 1, &mut table) };

        match r.as_result() {
            Ok(()) => {
                assert!(!table.is_null());
                // SAFETY: The returned pointer is valid
                unsafe { Some(AcpiTableHeader::from_ffi(&*table)) }
            }
            Err(_) => None,
        }
    }

    /// Returns an iterator over all the tables detected by ACPICA.
    /// If you want to find a specific common table, there may be a dedicated method for finding it instead, for instance [`dsdt`], [`madt`], [`fadt`], etc.
    ///
    /// [`dsdt`]: AcpicaOperation::dsdt
    /// [`madt`]: AcpicaOperation::madt
    /// [`fadt`]: AcpicaOperation::fadt
    #[allow(clippy::missing_panics_doc)]
    pub fn tables(&self) -> impl Iterator<Item = AcpiTableHeader> {
        let mut i = 1;

        core::iter::from_fn(move || {
            let mut ptr = core::ptr::null_mut();

            // SAFETY:
            let r = unsafe { AcpiGetTableByIndex(i, &mut ptr) };

            i += 1;

            match r.as_result() {
                Ok(()) => {
                    assert!(!ptr.is_null());
                    // SAFETY: The returned pointer is valid
                    unsafe { Some(AcpiTableHeader::from_ffi(&*ptr)) }
                }
                Err(_) => None,
            }
        })
    }

    /// Gets an iterator over all of the loaded SSDT tables.
    pub fn ssdt_tables(&self) -> impl Iterator<Item = AcpiTableHeader> {
        self.get_tables_of_type(*b"SSDT")
    }

    /// Gets an iterator over all of the loaded UEFI tables.
    pub fn uefi_tables(&self) -> impl Iterator<Item = Uefi> {
        self.get_tables_of_type(*b"UEFI").map(|header| {
            let ptr = header.as_ffi() as *const FfiAcpiTableHeader;
            let ptr = ptr.cast();
            // SAFETY: The signature is "UEFI" so the table can be cast to an FfiAcpiTableUefi
            unsafe { Uefi::from_ffi(&*ptr) }
        })
    }

    /// Gets the DSDT
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn dsdt(&self) -> AcpiTableHeader {
        self.table(*b"DSDT")
            .expect("System should have contained DSDT")
    }

    /// Gets the MADT
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn madt(&self) -> Madt {
        let ptr = self
            .table(*b"APIC")
            .expect("System should have contained MADT")
            .as_ffi() as *const FfiAcpiTableHeader;
        let ptr = ptr.cast();
        // SAFETY: The signature is "APIC" so the table is an MADT
        unsafe { Madt::from_ffi(&*ptr) }
    }

    /// Gets the FADT
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn fadt(&self) -> Fadt {
        let ptr = self
            .table(*b"FACP")
            .expect("System should have contained FADT")
            .as_ffi() as *const FfiAcpiTableHeader;
        let ptr = ptr.cast();
        // SAFETY: The signature is "APIC" so the table is an MADT
        unsafe { Fadt::from_ffi(&*ptr) }
    }

    /// Gets the MCFG
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn mcfg(&self) -> Option<Mcfg> {
        let ptr = self.table(*b"MCFG")?.as_ffi() as *const FfiAcpiTableHeader;

        let ptr = ptr.cast();
        // SAFETY: The signature is "APIC" so the table is an MADT
        unsafe { Some(Mcfg::from_ffi(&*ptr)) }
    }
}
