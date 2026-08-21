//! Keyboard shortcuts help overlay (RFC 038).
//!
//! A static modal table listing all keyboard shortcuts active on the main
//! screen.  The overlay (backdrop + centering) is assembled in `App::view()`.

use iced::{
    Color, Element, Length, Padding,
    widget::{button, column, container, row, text},
};
use rust_i18n::t;

use crate::app::Message;
use snora::design::Tokens;

/// Build the help dialog box (without the backdrop overlay).
pub fn view<'a>(tokens: &'a Tokens) -> Element<'a, Message> {
    let title = text(t!("help.title").to_string())
        .size(tokens.typography.title.size)
        .line_height(tokens.typography.title.line_height)
        .font(iced::Font { weight: iced::font::Weight::Bold, ..Default::default() });

    // Column headers
    let header = row![
        text(t!("help.shortcut_label").to_string())
            .size(tokens.typography.label.size)
            .line_height(tokens.typography.label.line_height)
            .font(iced::Font { weight: iced::font::Weight::Semibold, ..Default::default() })
            .width(Length::Fixed(160.0)),
        text(t!("help.action_label").to_string())
            .size(tokens.typography.label.size)
            .line_height(tokens.typography.label.line_height)
            .font(iced::Font { weight: iced::font::Weight::Semibold, ..Default::default() }),
    ]
    .spacing(tokens.spacing.md);

    let separator_border = crate::style::to_iced(tokens.palette.border);
    let separator = container(iced::widget::space().height(1))
        .width(Length::Fill)
        .height(Length::Fixed(1.0))
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(separator_border)),
            ..Default::default()
        });

    // Shortcut rows — (key_label, action_key)
    let shortcuts: &[(&str, &str)] = &[
        ("Ctrl + S",          "help.save"),
        ("Ctrl + R",          "help.rerun"),
        ("Ctrl + Z",          "help.undo"),
        ("Ctrl + Shift + Z",  "help.revert"),
        ("Ctrl + Enter",      "help.approve_and_save"),
        ("Ctrl + E",          "help.export"),
        ("↑ / ↓",            "help.navigate"),
        ("Tab / Shift+Tab",   "help.cycle_pane"),
        ("Enter",             "help.approve"),
        ("/",                 "help.search"),
        ("?",                 "help.show_help"),
    ];

    let key_color = crate::style::to_iced(tokens.palette.text_primary);
    let action_color = crate::style::to_iced(tokens.palette.text_secondary);
    let rows: Vec<Element<'_, Message>> = shortcuts
        .iter()
        .map(|(key, action_key)| {
            row![
                text(*key)
                    .size(tokens.typography.label.size)
                    .line_height(tokens.typography.label.line_height)
                    .font(iced::Font::MONOSPACE)
                    .color(key_color)
                    .width(Length::Fixed(160.0)),
                text(t!(*action_key).to_string())
                    .size(tokens.typography.body_small.size)
                    .line_height(tokens.typography.body_small.line_height)
                    .color(action_color),
            ]
            .spacing(tokens.spacing.md)
            .into()
        })
        .collect();

    // Close button
    let close_btn = button(
        text(t!("help.close").to_string())
            .size(tokens.typography.label.size)
            .line_height(tokens.typography.label.line_height),
    )
        .on_press(Message::CloseHelp)
        .padding(Padding::from([tokens.spacing.sm, tokens.spacing.lg]));

    let body = column![
        title,
        separator,
        header,
    ]
    .extend(rows)
    .push(
        row![
            iced::widget::space().width(Length::Fill),
            close_btn,
        ]
        .align_y(iced::Alignment::Center),
    )
    .spacing(tokens.spacing.sm)
    .width(Length::Fixed(380.0));

    container(body)
        .padding(Padding::from([tokens.spacing.xl, tokens.spacing.xxl]))
        .style(move |_theme| dialog_style(tokens))
        .into()
}

fn dialog_style(tokens: &Tokens) -> container::Style {
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
