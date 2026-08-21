//! Settings dialog view (RFC 036).
//!
//! Builds the dialog box content.  The modal overlay (backdrop + centering)
//! is assembled in `App::view()` using `iced::widget::stack!`.

use iced::{
    Color, Element, Length, Padding,
    widget::{button, column, container, row, scrollable, space, text, text_input},
};
use rust_i18n::t;

use aaai::profile::prefs::UserPrefs;
use crate::app::Message;

/// Build the settings dialog box (without the backdrop overlay).
///
/// `draft` is the mutable copy being edited; `locale` is the *currently active*
/// locale code so the language picker can show the right selection.
pub fn view<'a>(draft: &'a UserPrefs, locale: &'a str, tokens: &'a snora::design::Tokens) -> Element<'a, Message> {
    // ── Title ─────────────────────────────────────────────────────────
    let title = text(t!("settings.title").to_string())
        .size(tokens.typography.title.size)
        .line_height(tokens.typography.title.line_height)
        .font(iced::Font { weight: iced::font::Weight::Bold, ..Default::default() });

    // ── Language section ──────────────────────────────────────────────
    let lang_label = text(t!("settings.language").to_string())
        .size(tokens.typography.label.size)
        .line_height(tokens.typography.label.line_height);

    // Use the existing SUPPORTED_LOCALES from i18n module.
    // Language picker: labels are the own-language names ("English", "日本語").
    let labels: Vec<&str> = crate::i18n::SUPPORTED_LOCALES.iter()
        .map(|(_, label)| *label)
        .collect();

    // Active selection: prefer the draft language if set, else the live locale.
    let active_code = if !draft.language.is_empty() { &draft.language } else { locale };
    let active_label = crate::i18n::SUPPORTED_LOCALES
        .iter()
        .find(|(c, _)| *c == active_code)
        .map(|(_, l)| *l)
        .unwrap_or("English");

    let lang_pick = iced::widget::pick_list(
        labels,
        Some(active_label),
        |label: &str| {
            let code = crate::i18n::SUPPORTED_LOCALES
                .iter()
                .find(|(_, l)| *l == label)
                .map(|(c, _)| c.to_string())
                .unwrap_or_default();
            Message::SettingsLanguageChanged(code)
        },
    )
    .text_size(13)
    .padding(Padding::from([tokens.spacing.xs, tokens.spacing.sm]));

    let language_section = column![
        lang_label,
        lang_pick,
    ].spacing(tokens.spacing.sm);

    // ── Ignored directories section ───────────────────────────────────
    let ignored_label = text(t!("settings.ignored_dirs").to_string())
        .size(tokens.typography.label.size)
        .line_height(tokens.typography.label.line_height)
        .font(iced::Font { weight: iced::font::Weight::Semibold, ..Default::default() });

    let ignored_hint = text(t!("settings.ignored_dirs_hint").to_string())
        .size(tokens.typography.body_small.size)
        .line_height(tokens.typography.body_small.line_height)
        .color(crate::style::to_iced(tokens.palette.text_muted));

    let placeholder = t!("settings.dir_placeholder").to_string();

    let dir_rows: Vec<Element<'_, Message>> = draft
        .global_ignored_dirs
        .iter()
        .enumerate()
        .map(|(i, dir)| {
            let input = text_input(&placeholder, dir)
                .on_input(move |s| Message::SettingsIgnoreDirEdit(i, s))
                .padding(Padding::from([tokens.spacing.xs, tokens.spacing.sm]))
                .size(tokens.typography.body_small.size)
                .line_height(tokens.typography.body_small.line_height)
                .width(Length::Fill);

            let remove_btn = button(
                text("×")
                    .size(tokens.typography.label.size)
                    .line_height(tokens.typography.label.line_height)
                    .color(crate::style::to_iced(tokens.palette.text_muted))
            )
            .on_press(Message::SettingsIgnoreDirRemove(i))
            .padding(Padding::from([tokens.spacing.xs, tokens.spacing.sm]))
            .style({ let t = tokens.clone(); move |_th, s| crate::style::btn_ghost(&t, s) });

            row![input, remove_btn]
                .spacing(tokens.spacing.xs)
                .align_y(iced::Alignment::Center)
                .into()
        })
        .collect();

    let add_btn = button(
        text(t!("settings.add_dir").to_string())
            .size(tokens.typography.label.size)
            .line_height(tokens.typography.label.line_height),
    )
        .on_press(Message::SettingsIgnoreDirAdd)
        .padding(Padding::from([tokens.spacing.xs, tokens.spacing.sm]))
        .style({ let t = tokens.clone(); move |_th, s| crate::style::btn_ghost(&t, s) });

    let dir_list = scrollable(
        column(dir_rows).spacing(tokens.spacing.xs),
    )
    .height(Length::Shrink);

    let ignored_section = column![
        ignored_label,
        ignored_hint,
        dir_list,
        add_btn,
    ].spacing(tokens.spacing.sm);

    // ── Action buttons ────────────────────────────────────────────────
    let cancel_btn = button(
        text(t!("settings.cancel").to_string())
            .size(tokens.typography.label.size)
            .line_height(tokens.typography.label.line_height)
    )
    .on_press(Message::CloseSettings)
    .padding(Padding::from([tokens.spacing.sm, tokens.spacing.lg]))
    .style({ let t = tokens.clone(); move |_th, s| crate::style::btn_secondary(&t, s) });

    let save_btn = button(
        text(t!("settings.save").to_string())
            .size(tokens.typography.label.size)
            .line_height(tokens.typography.label.line_height)
    )
    .on_press(Message::SaveSettings)
    .padding(Padding::from([tokens.spacing.sm, tokens.spacing.lg]));

    let actions = row![
        space().width(Length::Fill),
        cancel_btn,
        save_btn,
    ]
    .spacing(tokens.spacing.sm)
    .align_y(iced::Alignment::Center);

    // ── Dialog box ────────────────────────────────────────────────────
    // ── Theme picker (RFC 093) ─────────────────────────────────────────────
    let theme_label = text(t!("settings.theme").to_string())
        .size(tokens.typography.label.size)
        .line_height(tokens.typography.label.line_height)
        .font(iced::Font { weight: iced::font::Weight::Semibold, ..Default::default() });

    let theme_options: Vec<aaai::profile::prefs::Theme> =
        aaai::profile::prefs::Theme::choices().to_owned();

    let theme_labels: Vec<String> = theme_options.iter().map(|th| {
        let key = match th {
            aaai::profile::prefs::Theme::Light             => "settings.theme_light",
            aaai::profile::prefs::Theme::Dark              => "settings.theme_dark",
            aaai::profile::prefs::Theme::System            => "settings.theme_system",
            aaai::profile::prefs::Theme::HighContrastLight => "settings.theme_high_contrast_light",
            aaai::profile::prefs::Theme::HighContrastDark  => "settings.theme_high_contrast_dark",
        };
        t!(key).to_string()
    }).collect();

    let active_theme_label = {
        let key = match draft.theme {
            aaai::profile::prefs::Theme::Light             => "settings.theme_light",
            aaai::profile::prefs::Theme::Dark              => "settings.theme_dark",
            aaai::profile::prefs::Theme::System            => "settings.theme_system",
            aaai::profile::prefs::Theme::HighContrastLight => "settings.theme_high_contrast_light",
            aaai::profile::prefs::Theme::HighContrastDark  => "settings.theme_high_contrast_dark",
        };
        t!(key).to_string()
    };

    let theme_pick = iced::widget::pick_list(
        theme_labels.clone(),
        Some(active_theme_label),
        move |selected: String| {
            // Map selected label back to Theme variant
            let matched = theme_options.iter().zip(theme_labels.iter())
                .find(|(_, lbl)| **lbl == selected)
                .map(|(th, _)| *th)
                .unwrap_or(aaai::profile::prefs::Theme::Light);
            Message::SettingsThemeChanged(matched)
        },
    )
    .width(Length::Fill);

    let theme_section = column![theme_label, theme_pick].spacing(tokens.spacing.sm);

    let body = column![
        title,
        separator(tokens),
        theme_section,
        separator(tokens),
        language_section,
        separator(tokens),
        ignored_section,
        separator(tokens),
        actions,
    ]
    .spacing(tokens.spacing.lg)
    .width(Length::Fixed(400.0));

    container(body)
        .padding(Padding::from([tokens.spacing.xl, tokens.spacing.xxl]))
        .style(move |_theme| dialog_style(tokens))
        .into()
}

fn separator<'a>(tokens: &snora::design::Tokens) -> Element<'a, Message> {
    let border = crate::style::to_iced(tokens.palette.border);
    container(space().height(1))
        .width(Length::Fill)
        .height(Length::Fixed(1.0))
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(border)),
            ..Default::default()
        })
        .into()
}

fn dialog_style(tokens: &snora::design::Tokens) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(crate::style::to_iced(tokens.palette.surface_raised))),
        border: iced::Border {
            color: crate::style::to_iced(tokens.palette.border),
            width: 1.0,
            radius: iced::border::Radius::from(8.0),
        },
        shadow: iced::Shadow {
            color: Color { a: 0.18, ..Color::BLACK },
            offset: iced::Vector { x: 0.0, y: 4.0 },
            blur_radius: 16.0,
        },
        ..Default::default()
    }
}
