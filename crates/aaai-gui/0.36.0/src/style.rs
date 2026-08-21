//! Style helpers for aaai GUI (RFC 092 — design system adoption).
//!
//! Container style functions take [`snora::design::Tokens`] by value (moved
//! into the closure), matching snora's own helper convention. Callers pass
//! `app.design_tokens.clone()` — `Tokens` is cheap to clone.
//!
//! Button helpers return [`iced::widget::button::Style`] directly via the
//! snora-design style functions. Callers keep full control over widget
//! construction and padding (required for ABDD ≥ 44 px tap targets, RFC 014).

use iced::{Background, Border, Color, widget::container};
use snora::design::Tokens;
use snora::design::style::button  as snora_btn;
use snora::design::style::container as snora_container;

// ── Container styles ──────────────────────────────────────────────────────

/// Opening screen folder cards and inspector cards.
///
/// Delegates to `snora::design::style::container::card_surface` so background,
/// border colour, and corner radius track the active token preset.
pub fn card_style(tokens: Tokens) -> impl Fn(&iced::Theme) -> container::Style {
    move |_theme| snora_container::card_surface(&tokens)
}

/// Toolbar, filter bar, and bottom action bar.
///
/// No snora-design equivalent for a full-width panel with no border radius;
/// hand-rolled with a token-derived border colour.
pub fn panel_style(tokens: Tokens) -> impl Fn(&iced::Theme) -> container::Style {
    let border_color = to_iced(tokens.palette.border);
    let bg = Color::from_rgb(0.95, 0.96, 0.97);
    move |_theme| container::Style {
        background: Some(Background::Color(bg)),
        border: Border { color: border_color, width: 1.0, radius: 0.0.into() },
        ..Default::default()
    }
}

/// Empty-state placeholder panels (file tree, diff pane, inspector, onboarding).
///
/// Transparent fill, token-derived border colour, gentle rounded corners.
pub fn empty_state_panel_style(tokens: Tokens) -> impl Fn(&iced::Theme) -> container::Style {
    let border_color = to_iced(tokens.palette.border);
    move |_theme| container::Style {
        background: None,
        border: Border { color: border_color, width: 1.0, radius: 8.0.into() },
        ..Default::default()
    }
}

// ── Button style functions ────────────────────────────────────────────────
//
// Usage — callers clone tokens once into the closure, keeping own padding:
//
//   let t = app.design_tokens.clone();
//   button(text("Save").size(13))
//       .padding([10.0, 20.0])           // ABDD: ≥44px height
//       .on_press(Message::Save)
//       .style(move |_theme, s| btn_primary(&t, s))

/// Filled accent button — primary call to action.
pub fn btn_primary(t: &Tokens, s: iced::widget::button::Status)
    -> iced::widget::button::Style { snora_btn::primary(t, s) }

/// Outlined accent button — secondary action.
pub fn btn_secondary(t: &Tokens, s: iced::widget::button::Status)
    -> iced::widget::button::Style { snora_btn::secondary(t, s) }

/// No-fill button — low-emphasis tertiary action.
pub fn btn_ghost(t: &Tokens, s: iced::widget::button::Status)
    -> iced::widget::button::Style { snora_btn::ghost(t, s) }

/// Filled danger button — destructive action.
pub fn btn_danger(t: &Tokens, s: iced::widget::button::Status)
    -> iced::widget::button::Style { snora_btn::danger(t, s) }

// ── Utility ───────────────────────────────────────────────────────────────

pub(crate) fn to_iced(c: snora::design::Color) -> Color {
    Color { r: c.r, g: c.g, b: c.b, a: c.a }
}
