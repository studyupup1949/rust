use std::ffi::NulError;
use std::panic::PanicHookInfo;
use std::str::Utf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("ptr is null")]
    NPE,
    #[error("string is null")]
    StringNull,
    #[error("type invalid, expect:{0}")]
    TypeInvalid(&'static str),
    #[error("bytes invalid, {0}")]
    BytesInvalid(String),
    #[error("serde error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("utf8 error: {0}")]
    UTF8Error(#[from] Utf8Error),
    #[error("null error: {0}")]
    NulError(#[from] NulError),
}

pub fn format_panic_info(info: &PanicHookInfo) -> String {
    let payload = info.payload();
    let payload_str = if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "Box<dyn Any>".to_string()
    };

    let location_str = info
        .location()
        .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
        .unwrap_or_else(|| "unknown location".to_string());

    format!("panicked at '{}', {}", payload_str, location_str)
}

#[cfg(target_arch = "wasm32")]
mod print {
    use crate::error::format_panic_info;
    use crate::preclude::AbiString;

    #[allow(improper_ctypes)]
    unsafe extern "C" {
        fn console_error(ptr: AbiString);
    }
    pub fn print_error(s: String) {
        unsafe {
            console_error(AbiString::new(s));
        }
    }
    #[unsafe(no_mangle)]
    pub extern "C" fn __init() {
        std::panic::set_hook(Box::new(|f| crate::error::print_error(format_panic_info(f))));
    }
}
#[cfg(not(target_arch = "wasm32"))]
mod print {
    use crate::preclude::AbiString;
    use std::sync::RwLock;

    pub type ErrorCallback = extern "C" fn(AbiString);
    pub(crate) static ERROR_CALLBACK: RwLock<Option<ErrorCallback>> = RwLock::new(None);
    pub(crate) fn update_error_callback(cb: ErrorCallback) {
        ERROR_CALLBACK.write().unwrap().replace(cb);
    }
    pub fn print_error(s: String) {
        if let Ok(lock) = ERROR_CALLBACK.read() {
            if let Some(func) = lock.as_ref() {
                func(AbiString::new(s));
            }
        }
    }
    #[unsafe(no_mangle)]
    pub extern "C" fn __init(cb: ErrorCallback) {
        update_error_callback(cb);
    }
}

pub use print::*;
