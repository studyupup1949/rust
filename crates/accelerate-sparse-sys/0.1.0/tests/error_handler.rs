//! Pins the one property that makes this shim safe to wrap: a parameter violation reaches the
//! caller as a status and a message, rather than killing the process.
//!
//! Accelerate's parameter checks call `_SparseTrap()` when no error callback is installed, and
//! its default options leave the callback unset. If this regresses, the test runner dies instead
//! of failing — read a crash here as that before anything else.

#![cfg(target_os = "macos")]

use accelerate_sparse_sys as sys;
use core::ffi::{CStr, c_char, c_int};
use std::ptr;
use std::sync::Mutex;

static MESSAGES: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Records what Accelerate reported. Deliberately allocation-light and panic-free: this is called
/// from C, and unwinding across that boundary is undefined behaviour.
unsafe extern "C" fn record(message: *const c_char) {
    if message.is_null() {
        return;
    }
    // SAFETY: Accelerate passes a NUL-terminated C string that stays valid for this call.
    let text = unsafe { CStr::from_ptr(message) }
        .to_string_lossy()
        .into_owned();
    if let Ok(mut messages) = MESSAGES.lock() {
        messages.push(text);
    }
}

#[test]
fn a_parameter_violation_returns_a_status_instead_of_trapping() {
    // SAFETY: `record` is callable from any thread and cannot unwind — every failure path inside
    // it returns normally.
    unsafe { sys::accsp_set_error_handler(Some(record)) };

    // A negative dimension is a parameter violation rather than a matrix that cannot be factored,
    // so Accelerate takes the checking path that would otherwise trap. The pattern arrays are
    // well formed, so nothing here reads out of bounds.
    let column_starts: [i64; 1] = [0];
    let row_indices: [c_int; 0] = [];
    let attributes = sys::accsp_attributes {
        kind: sys::ACCSP_MATRIX_SYMMETRIC,
        triangle: sys::ACCSP_TRIANGLE_LOWER,
        transpose: 0,
        block_size: 1,
    };

    let mut status = 0;
    // SAFETY: `column_starts` has the one entry a zero-column matrix requires and `row_indices`
    // is correspondingly empty, so the shim's copy stays inside both.
    let handle = unsafe {
        sys::accsp_symbolic_new(
            sys::ACCSP_KIND_CHOLESKY,
            -1,
            0,
            column_starts.as_ptr(),
            row_indices.as_ptr(),
            &attributes,
            ptr::null(),
            &mut status,
        )
    };

    // Reaching this line at all is most of the point: without the callback the process would
    // have aborted inside Accelerate.
    assert!(
        handle.is_null(),
        "a rejected pattern must not yield a handle"
    );
    assert_eq!(status, sys::ACCSP_STATUS_PARAMETER_ERROR);

    let messages = MESSAGES
        .lock()
        .expect("the recording mutex is not poisoned");
    assert!(
        !messages.is_empty(),
        "the callback must receive Accelerate's diagnostic, so it can be attached to the error"
    );
}
