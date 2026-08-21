//! C FFI bindings for aam-rs.
//!
//! Compile with `--features ffi` (implied when building as `cdylib`).

#![allow(clippy::missing_safety_doc)]

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use crate::aam::AAM;
use crate::builder::InlineObject;
use crate::error::AamlError;
use crate::pipeline::formatter::FormattingOptions as FormatterRules;
#[cfg(feature = "reconstructer")]
use crate::reconstructer;

// nah::have_duplicate_code

fn first_error(errors: Vec<AamlError>) -> AamlError {
    errors
        .into_iter()
        .next()
        .unwrap_or_else(|| AamlError::ParseError {
            line: 1,
            content: String::new(),
            details: "unexpected empty parse error list".to_string(),
            diagnostics: None,
        })
}

// ── Opaque handle ────────────────────────────────────────────────────────────

pub struct AamHandle {
    inner: AAM,
    last_error: Option<CString>,
    reconstruct_instances: Vec<AAM>,
}

impl AamHandle {
    fn set_error(&mut self, err: &(impl ToString + ?Sized)) {
        let msg = err.to_string().replace('\0', "<NUL>");
        self.last_error = CString::new(msg).ok();
    }

    fn clear_error(&mut self) {
        self.last_error = None;
    }
}

// ── Lifecycle ────────────────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub extern "C" fn aam_new() -> *mut AamHandle {
    Box::into_raw(Box::new(AamHandle {
        inner: AAM::new(),
        last_error: None,
        reconstruct_instances: Vec::new(),
    }))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_free(handle: *mut AamHandle) {
    if !handle.is_null() {
        unsafe { drop(Box::from_raw(handle)) };
    }
}

// ── Parsing & Formatting ─────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_parse(handle: *mut AamHandle, content: *const c_char) -> i32 {
    if handle.is_null() || content.is_null() {
        return -1;
    }
    let handle = unsafe { &mut *handle };

    let Ok(content) = (unsafe { CStr::from_ptr(content) }).to_str() else {
        handle.set_error(&"invalid utf-8");
        return -1;
    };

    match AAM::parse(content) {
        Ok(aam) => {
            handle.inner = aam;
            handle.clear_error();
            0
        }
        Err(e) => {
            handle.set_error(&first_error(e));
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_load(handle: *mut AamHandle, path: *const c_char) -> i32 {
    if handle.is_null() || path.is_null() {
        return -1;
    }
    let handle = unsafe { &mut *handle };

    let Ok(path) = (unsafe { CStr::from_ptr(path) }).to_str() else {
        handle.set_error(&"invalid utf-8");
        return -1;
    };

    match AAM::load(path) {
        Ok(aam) => {
            handle.inner = aam;
            handle.clear_error();
            0
        }
        Err(e) => {
            handle.set_error(&first_error(e));
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_format(handle: *mut AamHandle, content: *const c_char) -> *mut c_char {
    if handle.is_null() || content.is_null() {
        return std::ptr::null_mut();
    }
    let handle_ref = unsafe { &mut *handle };

    let Ok(content_str) = (unsafe { CStr::from_ptr(content) }).to_str() else {
        handle_ref.set_error("invalid utf-8");
        return std::ptr::null_mut();
    };

    let rules = FormatterRules::default();
    match handle_ref.inner.format(content_str, &rules) {
        Ok(formatted) => to_c_string(&formatted),
        Err(e) => {
            handle_ref.set_error(&e);
            std::ptr::null_mut()
        }
    }
}

// ── Lookup ───────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_get(handle: *const AamHandle, key: *const c_char) -> *mut c_char {
    if handle.is_null() || key.is_null() {
        return std::ptr::null_mut();
    }
    let handle = unsafe { &*handle };

    let Ok(key) = (unsafe { CStr::from_ptr(key) }).to_str() else {
        return std::ptr::null_mut();
    };

    handle
        .inner
        .get(key)
        .map_or(std::ptr::null_mut(), to_c_string)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_find(handle: *const AamHandle, query: *const c_char) -> *mut c_char {
    if handle.is_null() || query.is_null() {
        return std::ptr::null_mut();
    }
    let handle = unsafe { &*handle };

    let Ok(query) = (unsafe { CStr::from_ptr(query) }).to_str() else {
        return std::ptr::null_mut();
    };

    to_c_string_map(handle.inner.find(query))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_deep_search(
    handle: *const AamHandle,
    pattern: *const c_char,
) -> *mut c_char {
    if handle.is_null() || pattern.is_null() {
        return std::ptr::null_mut();
    }
    let handle = unsafe { &*handle };

    let Ok(pattern) = (unsafe { CStr::from_ptr(pattern) }).to_str() else {
        return std::ptr::null_mut();
    };

    to_c_string_map(handle.inner.deep_search(pattern))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_reverse_search(
    handle: *const AamHandle,
    value: *const c_char,
) -> *mut c_char {
    if handle.is_null() || value.is_null() {
        return std::ptr::null_mut();
    }
    let handle = unsafe { &*handle };

    let Ok(value) = (unsafe { CStr::from_ptr(value) }).to_str() else {
        return std::ptr::null_mut();
    };

    to_c_string_list(&handle.inner.reverse_search(value))
}

// ── Schema Reconstruction ────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_reconstruct_push(
    handle: *mut AamHandle,
    content: *const c_char,
) -> i32 {
    if handle.is_null() || content.is_null() {
        return -1;
    }
    let handle = unsafe { &mut *handle };

    let Ok(content) = (unsafe { CStr::from_ptr(content) }).to_str() else {
        handle.set_error("invalid utf-8");
        return -1;
    };

    match AAM::parse(content) {
        Ok(aam) => {
            handle.reconstruct_instances.push(aam);
            handle.clear_error();
            0
        }
        Err(e) => {
            handle.set_error(&first_error(e));
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_reconstruct_schema(
    handle: *const AamHandle,
    schema_name: *const c_char,
) -> *mut c_char {
    if handle.is_null() || schema_name.is_null() {
        return std::ptr::null_mut();
    }
    let handle = unsafe { &*handle };

    let Ok(schema_name) = (unsafe { CStr::from_ptr(schema_name) }).to_str() else {
        return std::ptr::null_mut();
    };

    if handle.reconstruct_instances.is_empty() {
        return std::ptr::null_mut();
    }

    #[cfg(feature = "reconstructer")]
    {
        let schema = reconstructer::reconstruct_from_aam_instances(&handle.reconstruct_instances);
        let formatted = reconstructer::format_schema(schema_name, &schema);
        return to_c_string(&formatted);
    }

    #[cfg(not(feature = "reconstructer"))]
    {
        let _ = schema_name;
        std::ptr::null_mut()
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_reconstruct_clear(handle: *mut AamHandle) {
    if !handle.is_null() {
        let handle = unsafe { &mut *handle };
        handle.reconstruct_instances.clear();
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_schema_names(handle: *const AamHandle) -> *mut c_char {
    if handle.is_null() {
        return std::ptr::null_mut();
    }
    let handle = unsafe { &*handle };

    handle
        .inner
        .schemas()
        .map_or(std::ptr::null_mut(), |schemas| {
            let keys: Vec<&str> = schemas.keys().map(smol_str::SmolStr::as_str).collect();
            to_c_string_list(&keys)
        })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_type_names(handle: *const AamHandle) -> *mut c_char {
    if handle.is_null() {
        return std::ptr::null_mut();
    }
    let handle = unsafe { &*handle };

    handle.inner.types().map_or(std::ptr::null_mut(), |types| {
        let keys: Vec<&str> = types.keys().map(smol_str::SmolStr::as_str).collect();
        to_c_string_list(&keys)
    })
}

// ── Memory management ────────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_string_free(s: *mut c_char) {
    if !s.is_null() {
        unsafe { drop(CString::from_raw(s)) };
    }
}

// ── Error reporting ──────────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_last_error(handle: *const AamHandle) -> *const c_char {
    if handle.is_null() {
        return std::ptr::null();
    }
    let handle = unsafe { &*handle };
    handle
        .last_error
        .as_ref()
        .map_or(std::ptr::null(), |cs| cs.as_ptr())
}

// ── Private helpers ──────────────────────────────────────────────────────────

fn to_c_string(s: &str) -> *mut c_char {
    let safe = s.replace('\0', "<NUL>");
    match CString::new(safe) {
        Ok(cs) => cs.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

fn to_c_string_list(list: &[&str]) -> *mut c_char {
    if list.is_empty() {
        return std::ptr::null_mut();
    }
    let joined = list.join(",");
    to_c_string(&joined)
}

fn to_c_string_map(map: Vec<(&str, &str)>) -> *mut c_char {
    if map.is_empty() {
        return std::ptr::null_mut();
    }
    let joined = map
        .into_iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("\n");
    to_c_string(&joined)
}

// ── InlineObject FFI ─────────────────────────────────────────────────────────

/// Opaque handle for an inline object.
pub struct AamInlineObjectHandle {
    inner: InlineObject,
}

#[unsafe(no_mangle)]
pub extern "C" fn aam_inline_new() -> *mut AamInlineObjectHandle {
    Box::into_raw(Box::new(AamInlineObjectHandle {
        inner: InlineObject::new(),
    }))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_inline_free(handle: *mut AamInlineObjectHandle) {
    if !handle.is_null() {
        unsafe { drop(Box::from_raw(handle)) };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_inline_add(
    handle: *mut AamInlineObjectHandle,
    key: *const c_char,
    value: *const c_char,
) -> i32 {
    if handle.is_null() || key.is_null() || value.is_null() {
        return -1;
    }
    let handle = unsafe { &mut *handle };
    let Ok(key) = (unsafe { CStr::from_ptr(key) }).to_str() else {
        return -1;
    };
    let Ok(value) = (unsafe { CStr::from_ptr(value) }).to_str() else {
        return -1;
    };
    handle.inner.add_field(key, value);
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_inline_to_string(handle: *const AamInlineObjectHandle) -> *mut c_char {
    if handle.is_null() {
        return std::ptr::null_mut();
    }
    let handle = unsafe { &*handle };
    to_c_string(&handle.inner.to_string())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_parse_inline_to_map(content: *const c_char) -> *mut c_char {
    if content.is_null() {
        return std::ptr::null_mut();
    }
    let Ok(s) = (unsafe { CStr::from_ptr(content) }).to_str() else {
        return std::ptr::null_mut();
    };
    match crate::builder::parse_inline_to_map(s) {
        Ok(map) => {
            let joined = map
                .into_iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join("\n");
            to_c_string(&joined)
        }
        Err(_) => std::ptr::null_mut(),
    }
}
