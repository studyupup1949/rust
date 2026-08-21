//! Opening screen — RFC 015 redesign.
//!
//! Design principles (from aaai_uiux_design.pdf p.2):
//!   "選ぶ → 見る → 理由を書く → 承認する → 保存する → 確認する"
//!
//! The first action is *selecting* folders (not typing paths).
//! Required 2 folders are most prominent; optional settings are collapsed.

use iced::{
    Alignment::Center, Color, Element, Length, Padding,
    widget::{button, column, container, row, scrollable, space, text, text_input},
};
use rust_i18n::t;

use crate::app::{App, Message};
use crate::style::card_style;

pub fn view(app: &App) -> Element<'_, Message> {
    // ── Welcome section ─────────────────────────────────────────────
    let title = text(t!("opening.title").to_string())
        .size(48)
        .font(iced::Font { weight: iced::font::Weight::Bold, ..Default::default() });
    let subtitle = text(t!("opening.subtitle").to_string()).size(16)
        .color(Color::from_rgb(0.45, 0.47, 0.52));
    let guide = text(t!("opening.guide").to_string())
        .size(14)
        .color(Color::from_rgb(0.30, 0.32, 0.38));

    // ── Required folder cards ───────────────────────────────────────
    let before_card = folder_picker_card(
        t!("opening.before_card").to_string(),
        &app.before_path,
        app.opening_validation.before_error.as_deref(),
        Message::PickBeforeFolder,
    );
    let after_card = folder_picker_card(
        t!("opening.after_card").to_string(),
        &app.after_path,
        app.opening_validation.after_error.as_deref(),
        Message::PickAfterFolder,
    );

    // ── Optional settings (collapsible) ─────────────────────────────
    let optional_section = optional_settings_section(app);

    // ── Start audit button ──────────────────────────────────────────
    let can_start = app.opening_validation.can_start()
        && !app.before_path.trim().is_empty()
        && !app.after_path.trim().is_empty()
        && !app.is_loading;
    let start_btn = button(
        text(t!("opening.start_button").to_string())
            .size(15)
            .font(iced::Font { weight: iced::font::Weight::Semibold, ..Default::default() }),
    )
    .on_press_maybe(if can_start { Some(Message::StartAudit) } else { None })
    .padding(Padding::from([12.0, 32.0]));

    // ── Recent projects ─────────────────────────────────────────────
    let recent_section = recent_projects_section(app);

    // ── Compose layout ──────────────────────────────────────────────
    let main_col = column![
        space().height(Length::Fixed(24.0)),
        title,
        subtitle,
        space().height(Length::Fixed(8.0)),
        guide,
        space().height(Length::Fixed(28.0)),
        before_card,
        space().height(Length::Fixed(12.0)),
        after_card,
        space().height(Length::Fixed(16.0)),
        optional_section,
        space().height(Length::Fixed(24.0)),
        container(start_btn).width(Length::Fill).center_x(Length::Fill),
        space().height(Length::Fixed(32.0)),
        recent_section,
        space().height(Length::Fixed(32.0)),
    ]
    .spacing(0)
    .align_x(Center)
    .max_width(720)
    .padding(Padding::from([16.0, 32.0]));

    scrollable(
        container(main_col)
            .width(Length::Fill)
            .center_x(Length::Fill),
    )
    .into()
}

// ──────────────────────────────────────────────────────────────────────
// Sub-components
// ──────────────────────────────────────────────────────────────────────

fn folder_picker_card<'a>(
    label: String,
    current_path: &'a str,
    error: Option<&'a str>,
    pick_msg: Message,
) -> Element<'a, Message> {
    let is_selected = !current_path.trim().is_empty();

    // Status line: ✓ /path/to/folder  or  ✗ 未選択
    let status_line: Element<'a, Message> = if is_selected {
        let valid = error.is_none();
        let icon = if valid { "✓" } else { "✗" };
        let color = if valid {
            Color::from_rgb(0.15, 0.60, 0.25)
        } else {
            Color::from_rgb(0.80, 0.15, 0.15)
        };
        row![
            text(icon).size(14).color(color),
            text(current_path).size(13)
                .font(iced::Font::MONOSPACE)
                .color(Color::from_rgb(0.25, 0.27, 0.32)),
        ]
        .spacing(6)
        .align_y(Center)
        .into()
    } else {
        row![
            text("✗").size(14).color(Color::from_rgb(0.65, 0.40, 0.10)),
            text(t!("opening.unselected").to_string()).size(13)
                .color(Color::from_rgb(0.55, 0.55, 0.60)),
        ]
        .spacing(6)
        .align_y(Center)
        .into()
    };

    let pick_btn_label = if is_selected {
        t!("opening.change_folder")
    } else {
        t!("opening.pick_folder")
    };
    let pick_btn = button(text(pick_btn_label.to_string()).size(13))
        .on_press(pick_msg)
        .padding(Padding::from([10.0, 18.0]));

    let card_label = text(format!("📁 {}", label))
        .size(14)
        .font(iced::Font { weight: iced::font::Weight::Semibold, ..Default::default() });

    let body = column![
        card_label,
        space().height(Length::Fixed(8.0)),
        row![
            status_line,
            space().width(Length::Fill),
            pick_btn,
        ]
        .spacing(12)
        .align_y(Center)
        .width(Length::Fill),
    ]
    .spacing(0);

    // Add error message if any
    let body_with_error: Element<'a, Message> = if let Some(err) = error {
        column![
            body,
            space().height(Length::Fixed(6.0)),
            text(format!("⚠ {}", err)).size(11)
                .color(Color::from_rgb(0.80, 0.30, 0.10)),
        ].into()
    } else {
        body.into()
    };

    container(body_with_error)
        .padding(Padding::from([14.0, 16.0]))
        .width(Length::Fill)
        .style(card_style)
        .into()
}

fn optional_settings_section(app: &App) -> Element<'_, Message> {
    let expanded = app.optional_settings_expanded;
    let arrow = if expanded { "▾" } else { "▸" };

    let header = button(
        row![
            text(arrow).size(13)
                .color(Color::from_rgb(0.45, 0.47, 0.52)),
            text(t!("opening.optional_section").to_string())
                .size(13)
                .color(Color::from_rgb(0.30, 0.32, 0.38)),
        ]
        .spacing(8)
        .align_y(Center)
    )
    .on_press(Message::ToggleOptionalSettings)
    .style(iced::widget::button::text)
    .padding(Padding::from([6.0, 4.0]));

    let hint = text(t!("opening.optional_hint").to_string())
        .size(11)
        .color(Color::from_rgb(0.55, 0.55, 0.60));

    if !expanded {
        return column![header, hint].spacing(2).into();
    }

    let def_row = file_picker_row(
        t!("opening.definition_label").to_string(),
        &app.definition_path,
        Message::PickDefinitionFile,
        Message::DefinitionPathChanged,
    );
    let ignore_row = file_picker_row(
        t!("opening.ignore_label").to_string(),
        &app.ignore_path,
        Message::PickIgnoreFile,
        Message::IgnorePathChanged,
    );

    column![
        header,
        hint,
        space().height(Length::Fixed(8.0)),
        def_row,
        space().height(Length::Fixed(6.0)),
        ignore_row,
    ]
    .spacing(2)
    .into()
}

fn file_picker_row<'a, F>(
    label: String,
    current: &'a str,
    pick_msg: Message,
    on_text_change: F,
) -> Element<'a, Message>
where
    F: 'a + Fn(String) -> Message,
{
    let label_text = text(label).size(12)
        .color(Color::from_rgb(0.35, 0.37, 0.42));
    let input = text_input("", current)
        .on_input(on_text_change)
        .padding(Padding::from([8.0, 10.0]))
        .size(12);
    let pick_btn = button(text(t!("opening.pick_file").to_string()).size(12))
        .on_press(pick_msg)
        .padding(Padding::from([8.0, 14.0]));

    column![
        label_text,
        row![input, pick_btn].spacing(8).align_y(Center),
    ]
    .spacing(4)
    .into()
}

fn recent_projects_section(app: &App) -> Element<'_, Message> {
    let profiles = &app.profiles.profiles;
    if profiles.is_empty() {
        return space().height(Length::Fixed(0.0)).into();
    }

    let header = text(format!("─── {} ───", t!("opening.recent_section")))
        .size(12)
        .color(Color::from_rgb(0.55, 0.55, 0.60));

    let mut col = column![header, space().height(Length::Fixed(6.0))].spacing(4);

    for (idx, prof) in profiles.iter().take(5).enumerate() {
        let label = text(format!("▸ {}", prof.name)).size(13);
        let detail = text(format!(
            "  before: {}  →  after: {}",
            prof.before,
            prof.after
        ))
        .size(11)
        .color(Color::from_rgb(0.55, 0.55, 0.60))
        .font(iced::Font::MONOSPACE);
        let open_btn = button(text(t!("opening.open_recent").to_string()).size(12))
            .on_press(Message::LoadProfile(idx))
            .padding(Padding::from([8.0, 14.0]));

        let row_el = container(
            row![
                column![label, detail].spacing(2),
                space().width(Length::Fill),
                open_btn,
            ]
            .spacing(8)
            .align_y(Center)
            .width(Length::Fill),
        )
        .padding(Padding::from([8.0, 12.0]))
        .width(Length::Fill)
        .style(card_style);

        col = col.push(row_el);
    }

    col.into()
}
