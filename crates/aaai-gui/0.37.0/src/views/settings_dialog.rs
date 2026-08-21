//! Settings dialog view (RFC 036).
//!
//! Builds the dialog box content.  The modal overlay (backdrop + centering)
//! is assembled in `App::view()` using `iced::widget::stack!`.

use iced::{
    Color, Element, Length, Padding,
    widget::{button, column, container, row, scrollable, space, text, text_input},
};
use rust_i18n::t;

use aaai_core::profile::prefs::UserPrefs;
use crate::app::Message;

/// Build the settings dialog box (without the backdrop overlay).
///
/// `draft` is the mutable copy being edited; `locale` is the *currently active*
/// locale code so the language picker can show the right selection.
pub fn view<'a>(draft: &'a UserPrefs, locale: &'a str, tokens: &'a snora::design::Tokens) -> Element<'a, Message> {
    // ── Title ─────────────────────────────────────────────────────────
    let title = text(t!("settings.title").to_string())
        .size(16)
        .font(iced::Font { weight: iced::font::Weight::Bold, ..Default::default() });

    // ── Language section ──────────────────────────────────────────────
    let lang_label = text(t!("settings.language").to_string()).size(13);

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
    .padding(Padding::from([4.0, 8.0]));

    let language_section = column![
        lang_label,
        lang_pick,
    ].spacing(6);

    // ── Ignored directories section ───────────────────────────────────
    let ignored_label = text(t!("settings.ignored_dirs").to_string())
        .size(13)
        .font(iced::Font { weight: iced::font::Weight::Semibold, ..Default::default() });

    let ignored_hint = text(t!("settings.ignored_dirs_hint").to_string())
        .size(11)
        .color(Color::from_rgb(0.5, 0.5, 0.5));

    let placeholder = t!("settings.dir_placeholder").to_string();

    let dir_rows: Vec<Element<'_, Message>> = draft
        .global_ignored_dirs
        .iter()
        .enumerate()
        .map(|(i, dir)| {
            let input = text_input(&placeholder, dir)
                .on_input(move |s| Message::SettingsIgnoreDirEdit(i, s))
                .padding(Padding::from([3.0, 6.0]))
                .size(12)
                .width(Length::Fill);

            let remove_btn = button(
                text("×").size(12).color(Color::from_rgb(0.55, 0.55, 0.55))
            )
            .on_press(Message::SettingsIgnoreDirRemove(i))
            .padding(Padding::from([3.0, 6.0]))
            .style({ let t = tokens.clone(); move |_th, s| crate::style::btn_ghost(&t, s) });

            row![input, remove_btn]
                .spacing(4)
                .align_y(iced::Alignment::Center)
                .into()
        })
        .collect();

    let add_btn = button(text(t!("settings.add_dir").to_string()).size(12))
        .on_press(Message::SettingsIgnoreDirAdd)
        .padding(Padding::from([4.0, 8.0]))
        .style({ let t = tokens.clone(); move |_th, s| crate::style::btn_ghost(&t, s) });

    let dir_list = scrollable(
        column(dir_rows).spacing(4),
    )
    .height(Length::Shrink);

    let ignored_section = column![
        ignored_label,
        ignored_hint,
        dir_list,
        add_btn,
    ].spacing(6);

    // ── Action buttons ────────────────────────────────────────────────
    let cancel_btn = button(
        text(t!("settings.cancel").to_string()).size(13)
    )
    .on_press(Message::CloseSettings)
    .padding(Padding::from([6.0, 14.0]))
    .style({ let t = tokens.clone(); move |_th, s| crate::style::btn_secondary(&t, s) });

    let save_btn = button(
        text(t!("settings.save").to_string()).size(13)
    )
    .on_press(Message::SaveSettings)
    .padding(Padding::from([6.0, 14.0]));

    let actions = row![
        space().width(Length::Fill),
        cancel_btn,
        save_btn,
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // ── Dialog box ────────────────────────────────────────────────────
    // ── Theme picker (RFC 093) ─────────────────────────────────────────────
    let theme_label = text(t!("settings.theme").to_string())
        .size(13)
        .font(iced::Font { weight: iced::font::Weight::Semibold, ..Default::default() });

    let theme_options: Vec<aaai_core::profile::prefs::Theme> =
        aaai_core::profile::prefs::Theme::choices().to_owned();

    let theme_labels: Vec<String> = theme_options.iter().map(|th| {
        let key = match th {
            aaai_core::profile::prefs::Theme::Light  => "settings.theme_light",
            aaai_core::profile::prefs::Theme::Dark   => "settings.theme_dark",
            aaai_core::profile::prefs::Theme::System => "settings.theme_system",
            // RFC 094: HighContrastLight => "settings.theme_hc_light",
            //          HighContrastDark  => "settings.theme_hc_dark",
        };
        t!(key).to_string()
    }).collect();

    let active_theme_label = {
        let key = match draft.theme {
            aaai_core::profile::prefs::Theme::Light  => "settings.theme_light",
            aaai_core::profile::prefs::Theme::Dark   => "settings.theme_dark",
            aaai_core::profile::prefs::Theme::System => "settings.theme_system",
            // RFC 094: HighContrastLight => "settings.theme_hc_light",
            //          HighContrastDark  => "settings.theme_hc_dark",
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
                .unwrap_or(aaai_core::profile::prefs::Theme::Light);
            Message::SettingsThemeChanged(matched)
        },
    )
    .width(Length::Fill);

    let theme_section = column![theme_label, theme_pick].spacing(6);

    let body = column![
        title,
        separator(),
        theme_section,
        separator(),
        language_section,
        separator(),
        ignored_section,
        separator(),
        actions,
    ]
    .spacing(14)
    .width(Length::Fixed(400.0));

    container(body)
        .padding(Padding::from([24.0, 28.0]))
        .style(dialog_style)
        .into()
}

fn separator<'a>() -> Element<'a, Message> {
    container(space().height(1))
        .width(Length::Fill)
        .height(Length::Fixed(1.0))
        .style(|_| container::Style {
            background: Some(iced::Background::Color(
                Color::from_rgb(0.88, 0.88, 0.90),
            )),
            ..Default::default()
        })
        .into()
}

fn dialog_style(theme: &iced::Theme) -> container::Style {
    let _ = theme;
    container::Style {
        background: Some(iced::Background::Color(Color::WHITE)),
        border: iced::Border {
            color: Color::from_rgb(0.78, 0.78, 0.82),
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
