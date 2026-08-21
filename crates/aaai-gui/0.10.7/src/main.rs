//! aaai-gui — GUI entry point (iced + snora).
//!
//! i18n is initialised here with rust-i18n. The application reads the system
//! locale and falls back to English.

mod app;
mod i18n;
mod style;
mod theme;
mod views;
use rust_i18n::t;
use aaai_core::profile::prefs::Theme as AppTheme;

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
    .title(|_: &app::App| t!("app.title").to_string())
    .theme(|app: &app::App| match app.theme {
        AppTheme::Dark   => iced::Theme::Dark,
        AppTheme::Light  => iced::Theme::Light,
        AppTheme::System => iced::Theme::Light, // fallback
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
