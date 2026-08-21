//! The [`AcpiHandler`] trait, which is the interface with which ACPICA calls OS functions.

mod bindings;
mod handler_trait;

pub use handler_trait::AcpiHandler;

use alloc::ffi::CString;

use crate::{
    bindings::functions::{
        AcpiDebugTrace, AcpiEnterSleepState, AcpiEnterSleepStatePrep, AcpiGetTimer,
    },
    AcpicaOperation,
};

use super::status::AcpiError;

impl<const TI: bool, const TL: bool, const E: bool, const I: bool> AcpicaOperation<TI, TL, E, I> {
    /// Rust binding to the ACPICA `AcpiDebugTrace` function.
    ///
    /// # Panics
    /// * If the OS interface has not been set up using [`register_interface`]
    /// * If `name` contains null bytes, including at the end.
    ///
    /// TODO: Find enums for layer, level, and flags
    ///
    /// [`register_interface`]: super::register_interface
    pub fn debug_trace(
        &self,
        name: &str,
        level: u32,
        layer: u32,
        flags: u32,
    ) -> Result<(), AcpiError> {
        let ffi_name = CString::new(name).expect("name should not contain null bytes");

        // SAFETY: The passed pointer is valid as it was taken from a CString
        unsafe { AcpiDebugTrace(ffi_name.as_ptr(), level, layer, flags).as_result() }
    }
}

// TODO: check what level of initialization these really need
impl AcpicaOperation<true, true, true, true> {
    /// TODO: docs
    pub unsafe fn enter_sleep_state_prep(&mut self, state: u8) -> Result<(), AcpiError> {
        // SAFETY: TODO
        unsafe { AcpiEnterSleepStatePrep(state).as_result() }
    }

    /// TODO: docs
    pub unsafe fn enter_sleep_state(&mut self, state: u8) -> Result<(), AcpiError> {
        // SAFETY: TODO
        unsafe { AcpiEnterSleepState(state).as_result() }
    }

    /// TODO: docs
    pub fn get_timer(&mut self) -> u32 {
        let mut x = 0;

        // SAFETY: TODO
        unsafe { AcpiGetTimer(&mut x).as_result().unwrap() }

        x
    }
}
