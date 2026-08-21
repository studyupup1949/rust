//! i18n helper re-exports.
//!
//! Use `t!("key")` from `rust_i18n` directly in view code.
//! Locale switching is handled via `Message::SwitchLocale` → `rust_i18n::set_locale`.

pub const SUPPORTED_LOCALES: &[(&str, &str)] = &[
    ("en", "English"),
    ("ja", "日本語"),
];
