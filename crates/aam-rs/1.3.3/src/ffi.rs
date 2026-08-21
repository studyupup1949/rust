//! C FFI bindings for aam-rs.
//!
//! Compile with `--features ffi` (implied when building as `cdylib`).
//!
//! # Memory rules
//! - Strings returned by `aam_find_*` are heap-allocated — free them with [`aam_string_free`].
//! - The [`AamlHandle`] itself must be freed with [`aam_free`] when done.
//! - Error strings returned by [`aam_last_error`] are owned by the handle
//!   and are only valid until the next API call on that handle. **Do not** free them.

#![allow(clippy::missing_safety_doc)]

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use crate::aaml::AAML;

// ── Opaque handle ────────────────────────────────────────────────────────────

/// Opaque handle to an AAML parser instance.
///
/// Always create via [`aam_new`] and destroy via [`aam_free`].
pub struct AamlHandle {
    inner: AAML,
    /// Last error stored as a null-terminated C string so the pointer returned
    /// by `aam_last_error` is always safe to pass to C.
    last_error: Option<CString>,
}

impl AamlHandle {
    fn set_error(&mut self, err: impl ToString) {
        let msg = err.to_string().replace('\0', "<NUL>");
        self.last_error = CString::new(msg).ok();
    }

    fn clear_error(&mut self) {
        self.last_error = None;
    }
}

// ── Lifecycle ────────────────────────────────────────────────────────────────

/// Creates a new AAML handle with all default commands registered.
///
/// Returns `NULL` only if the allocator is out of memory.
/// The caller **must** free the handle with [`aam_free`] when done.
#[unsafe(no_mangle)]
pub extern "C" fn aam_new() -> *mut AamlHandle {
    Box::into_raw(Box::new(AamlHandle {
        inner: AAML::new(),
        last_error: None,
    }))
}

/// Frees an AAML handle previously created by [`aam_new`].
///
/// No-op when `handle` is `NULL`.
///
/// # Safety
/// `handle` must be a valid pointer returned by [`aam_new`] and must not be
/// used again after this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_free(handle: *mut AamlHandle) {
    if !handle.is_null() {
        unsafe { drop(Box::from_raw(handle)) };
    }
}

// ── Parsing ──────────────────────────────────────────────────────────────────

/// Parses AAML content from a null-terminated C string and replaces the
/// current state of `handle` with the result.
///
/// Returns `0` on success and `-1` on error. On error, [`aam_last_error`]
/// returns a human-readable description.
///
/// # Safety
/// Both `handle` and `content` must be valid non-null pointers.
/// `content` must point to a null-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_parse(handle: *mut AamlHandle, content: *const c_char) -> i32 {
    if handle.is_null() || content.is_null() {
        return -1;
    }
    let handle = unsafe { &mut *handle };

    let content = match unsafe { CStr::from_ptr(content) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            handle.set_error(e);
            return -1;
        }
    };

    match AAML::parse(content) {
        Ok(aaml) => {
            handle.inner = aaml;
            handle.clear_error();
            0
        }
        Err(e) => {
            handle.set_error(e);
            -1
        }
    }
}

/// Loads an AAML file from disk and replaces the current state of `handle`.
///
/// Returns `0` on success and `-1` on error. On error, [`aam_last_error`]
/// returns a human-readable description.
///
/// # Safety
/// Both `handle` and `path` must be valid non-null pointers.
/// `path` must point to a null-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_load(handle: *mut AamlHandle, path: *const c_char) -> i32 {
    if handle.is_null() || path.is_null() {
        return -1;
    }
    let handle = unsafe { &mut *handle };

    let path = match unsafe { CStr::from_ptr(path) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            handle.set_error(e);
            return -1;
        }
    };

    match AAML::load(path) {
        Ok(aaml) => {
            handle.inner = aaml;
            handle.clear_error();
            0
        }
        Err(e) => {
            handle.set_error(e);
            -1
        }
    }
}

/// Parses AAML content and *merges* it into the existing state of `handle`
/// without resetting previously loaded keys.
///
/// Returns `0` on success and `-1` on error.
///
/// # Safety
/// Both `handle` and `content` must be valid non-null pointers.
/// `content` must point to a null-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_merge(handle: *mut AamlHandle, content: *const c_char) -> i32 {
    if handle.is_null() || content.is_null() {
        return -1;
    }
    let handle = unsafe { &mut *handle };

    let content = match unsafe { CStr::from_ptr(content) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            handle.set_error(e);
            return -1;
        }
    };

    match handle.inner.merge_content(content) {
        Ok(()) => {
            handle.clear_error();
            0
        }
        Err(e) => {
            handle.set_error(e);
            -1
        }
    }
}

// ── Lookup ───────────────────────────────────────────────────────────────────

/// Looks up `key` in the AAML map (forward lookup, then reverse lookup).
///
/// Returns a newly heap-allocated null-terminated C string that **must** be
/// freed with [`aam_string_free`], or `NULL` if the key was not found.
///
/// # Safety
/// Both `handle` and `key` must be valid non-null pointers.
/// `key` must point to a null-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_find_obj(
    handle: *const AamlHandle,
    key: *const c_char,
) -> *mut c_char {
    if handle.is_null() || key.is_null() {
        return std::ptr::null_mut();
    }
    let handle = unsafe { &*handle };

    let key = match unsafe { CStr::from_ptr(key) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    match handle.inner.find_obj(key) {
        Some(v) => to_c_string(v.as_str()),
        None => std::ptr::null_mut(),
    }
}

/// Reverse lookup: finds the *key* whose stored value equals `value`.
///
/// Returns a newly heap-allocated null-terminated C string that **must** be
/// freed with [`aam_string_free`], or `NULL` if no matching key exists.
///
/// # Safety
/// Both `handle` and `value` must be valid non-null pointers.
/// `value` must point to a null-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_find_key(
    handle: *const AamlHandle,
    value: *const c_char,
) -> *mut c_char {
    if handle.is_null() || value.is_null() {
        return std::ptr::null_mut();
    }
    let handle = unsafe { &*handle };

    let value = match unsafe { CStr::from_ptr(value) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    match handle.inner.find_key(value) {
        Some(v) => to_c_string(v.as_str()),
        None => std::ptr::null_mut(),
    }
}

/// Deep lookup: follows the chain `key → value → key` until a terminal value
/// is reached or a cycle is detected.
///
/// Returns a newly heap-allocated null-terminated C string that **must** be
/// freed with [`aam_string_free`], or `NULL` if the chain is empty.
///
/// # Safety
/// Both `handle` and `key` must be valid non-null pointers.
/// `key` must point to a null-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_find_deep(
    handle: *const AamlHandle,
    key: *const c_char,
) -> *mut c_char {
    if handle.is_null() || key.is_null() {
        return std::ptr::null_mut();
    }
    let handle = unsafe { &*handle };

    let key = match unsafe { CStr::from_ptr(key) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    match handle.inner.find_deep(key) {
        Some(v) => to_c_string(v.as_str()),
        None => std::ptr::null_mut(),
    }
}

// ── Memory management ────────────────────────────────────────────────────────

/// Frees a C string returned by any `aam_find_*` function.
///
/// No-op when `s` is `NULL`.
///
/// # Safety
/// `s` must be a pointer previously returned by [`aam_find_obj`],
/// [`aam_find_key`], or [`aam_find_deep`], and must not have been freed already.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_string_free(s: *mut c_char) {
    if !s.is_null() {
        unsafe { drop(CString::from_raw(s)) };
    }
}

// ── Error reporting ──────────────────────────────────────────────────────────

/// Returns a pointer to the last error message stored in `handle`, or `NULL`
/// if the previous operation succeeded.
///
/// The returned pointer is owned by `handle` and remains valid only until the
/// next API call on that handle. **Do not** free it with [`aam_string_free`].
///
/// # Safety
/// `handle` must be a valid non-null pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_last_error(handle: *const AamlHandle) -> *const c_char {
    if handle.is_null() {
        return std::ptr::null();
    }
    let handle = unsafe { &*handle };
    match &handle.last_error {
        Some(cs) => cs.as_ptr(),
        None => std::ptr::null(),
    }
}

// ── Private helpers ──────────────────────────────────────────────────────────

/// Converts `s` to a heap-allocated null-terminated C string.
///
/// Interior NUL bytes are replaced so `CString::new` cannot fail.
/// The caller is responsible for freeing the result with [`aam_string_free`].
fn to_c_string(s: &str) -> *mut c_char {
    let safe = s.replace('\0', "<NUL>");
    CString::new(safe).unwrap().into_raw()
}
