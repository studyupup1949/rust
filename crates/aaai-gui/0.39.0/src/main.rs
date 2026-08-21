//! aaai-gui — GUI entry point (iced + snora).
//!
//! i18n is initialised here with rust-i18n. The application reads the system
//! locale and falls back to English.

mod app;
mod design_tokens;
mod error;
mod i18n;
mod style;
mod theme;
mod util;
mod views;
use rust_i18n::t;
use aaai::profile::prefs::Theme as AppTheme;

// Initialise rust-i18n. Locale files live in `locales/`.
rust_i18n::i18n!("locales", fallback = "en");

fn main() -> iced::Result {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("warn"),
    )
    .init();

    // Detect system locale (default to Japanese as the primary locale).
    let locale = detect_locale();
    rust_i18n::set_locale(&locale);

    iced::application(
        app::App::default,
        app::App::update,
        app::App::view,
    )
    .subscription(app::App::subscription)
    .title(|app: &app::App| {
        // RFC 042 — dynamic title: `aaai — {filename}{dirty?}` on Main screen.
        // RFC 058 — append `(N pending)` when Pending > 0.
        let base = t!("app.title").to_string();
        if matches!(app.screen, app::Screen::Main) {
            let fname: Option<&str> = if !app.definition_path.is_empty() {
                std::path::Path::new(&app.definition_path)
                    .file_name()
                    .and_then(|n| n.to_str())
            } else {
                None
            };
            let dirty = if app.dirty { " ●" } else { "" };
            let pending = app.audit_result.as_ref()
                .map(|r| r.summary.pending)
                .unwrap_or(0);
            let pending_str = if pending > 0 {
                format!(" ({pending} pending)")
            } else {
                String::new()
            };
            if let Some(name) = fname {
                return format!("{} — {}{}{}", base, name, dirty, pending_str);
            } else if app.dirty || pending > 0 {
                return format!("{}{}{}", base, dirty, pending_str);
            }
        }
        base
    })
    .theme(|app: &app::App| match app.theme {
        AppTheme::Dark   => iced::Theme::Dark,
        AppTheme::Light  => iced::Theme::Light,
        AppTheme::System => iced::Theme::Light, // fallback
        // HC themes use the matching iced base for built-in widget chrome;
        // aaai's own widgets get the high-contrast snora tokens (RFC 092).
        AppTheme::HighContrastLight => iced::Theme::Light,
        AppTheme::HighContrastDark  => iced::Theme::Dark,
    })
    .run()
}

fn detect_locale() -> String {
    // Read LANG / LANGUAGE env vars; fall back to "en".
    let lang = std::env::var("LANG")
        .or_else(|_| std::env::var("LANGUAGE"))
        .unwrap_or_default();
    if lang.starts_with("ja") {
        "ja".to_string()
    } else {
        "en".to_string()
    }
}
