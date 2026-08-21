//! Resolves an [`aaai_core::profile::prefs::Theme`] preference to a
//! [`snora::design::Tokens`] bundle (RFC 092, extended by RFC 094).
//!
//! The bundle is stored in [`crate::app::App::design_tokens`] and updated
//! whenever the theme preference changes via `Message::SetTheme`. Every view
//! function that needs token-driven styling reads it from `app.design_tokens`.

use aaai_core::profile::prefs::Theme as AppTheme;
use snora::design::Tokens;

/// Return the resolved [`Tokens`] for the given [`AppTheme`].
///
/// `System` falls back to the light preset until iced 0.14 exposes a reliable
/// OS dark-mode query (RFC 093 §5.1).
pub fn tokens_for(theme: &AppTheme) -> Tokens {
    match theme {
        AppTheme::Light             => Tokens::light(),
        AppTheme::Dark              => Tokens::dark(),
        AppTheme::System            => Tokens::light(),
        AppTheme::HighContrastLight => Tokens::high_contrast_light(),
        AppTheme::HighContrastDark  => Tokens::high_contrast_dark(),
    }
}
