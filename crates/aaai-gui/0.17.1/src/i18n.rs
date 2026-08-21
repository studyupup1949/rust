//! i18n helper re-exports.
//!
//! Use `t!("key")` from `rust_i18n` directly in view code.
//! This module provides the locale-switch message handler.


pub const SUPPORTED_LOCALES: &[(&str, &str)] = &[
    ("en", "English"),
    ("ja", "日本語"),
];

#[allow(dead_code)]
pub fn switch_locale(code: &str) {
    rust_i18n::set_locale(code);
}
