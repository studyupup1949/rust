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

/// Build the help dialog box (without the backdrop overlay).
pub fn view<'a>() -> Element<'a, Message> {
    let title = text(t!("help.title").to_string())
        .size(16)
        .font(iced::Font { weight: iced::font::Weight::Bold, ..Default::default() });

    // Column headers
    let header = row![
        text(t!("help.shortcut_label").to_string())
            .size(12)
            .font(iced::Font { weight: iced::font::Weight::Semibold, ..Default::default() })
            .width(Length::Fixed(160.0)),
        text(t!("help.action_label").to_string())
            .size(12)
            .font(iced::Font { weight: iced::font::Weight::Semibold, ..Default::default() }),
    ]
    .spacing(12);

    let separator = container(iced::widget::space().height(1))
        .width(Length::Fill)
        .height(Length::Fixed(1.0))
        .style(|_| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.88, 0.88, 0.90))),
            ..Default::default()
        });

    // Shortcut rows — (key_label, action_key)
    let shortcuts: &[(&str, &str)] = &[
        ("Ctrl + S",        "help.save"),
        ("Ctrl + R",        "help.rerun"),
        ("Ctrl + Z",        "help.undo"),
        ("Ctrl + Shift + Z","help.revert"),
        ("Ctrl + E",        "help.export"),
        ("↑ / ↓",          "help.navigate"),
        ("Tab / Shift+Tab", "help.cycle_pane"),
        ("Enter",           "help.approve"),
        ("/",               "help.search"),
        ("?",               "help.show_help"),
    ];

    let rows: Vec<Element<'_, Message>> = shortcuts
        .iter()
        .map(|(key, action_key)| {
            row![
                text(*key)
                    .size(12)
                    .font(iced::Font::MONOSPACE)
                    .color(Color::from_rgb(0.20, 0.20, 0.24))
                    .width(Length::Fixed(160.0)),
                text(t!(*action_key).to_string())
                    .size(12)
                    .color(Color::from_rgb(0.30, 0.30, 0.35)),
            ]
            .spacing(12)
            .into()
        })
        .collect();

    // Close button
    let close_btn = button(text(t!("help.close").to_string()).size(13))
        .on_press(Message::CloseHelp)
        .padding(Padding::from([6.0, 14.0]));

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
    .spacing(8)
    .width(Length::Fixed(380.0));

    container(body)
        .padding(Padding::from([24.0, 28.0]))
        .style(dialog_style)
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
