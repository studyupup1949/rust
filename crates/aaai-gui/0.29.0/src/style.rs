//! Style helpers for aaai GUI.

use iced::{Background, Color, widget::container};

pub fn card_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb(0.97, 0.97, 0.98))),
        border: iced::Border {
            color: Color::from_rgb(0.85, 0.85, 0.87),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

pub fn panel_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb(0.95, 0.96, 0.97))),
        border: iced::Border {
            color: Color::from_rgb(0.82, 0.82, 0.85),
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

/// RFC 022 — visual chrome for empty-state placeholder panels (file
/// tree / diff panel / inspector / Opening onboarding). Intentionally
/// understated: transparent background, soft 1-px border, gentle
/// rounded corners. The container chrome is the *only* signal — actual
/// guidance text is supplied by the caller via i18n keys under
/// `empty_state.*` (en/ja both populated).
pub fn empty_state_panel_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: None,
        border: iced::Border {
            color: Color::from_rgb(0.82, 0.82, 0.85),
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}
