//! Unsaved-changes navigation guard dialog (RFC 041).
//!
//! A compact 3-choice modal:
//!   [Cancel]  [Discard and Leave]  [Save and Leave]
//!
//! The modal overlay (backdrop + centering) is assembled in `App::view()`.

use iced::{
    Color, Element, Length, Padding,
    widget::{button, column, container, row, space, text},
};
use rust_i18n::t;

use crate::app::Message;

pub fn view<'a>() -> Element<'a, Message> {
    let title = text(t!("nav_guard.title").to_string())
        .size(16)
        .font(iced::Font { weight: iced::font::Weight::Bold, ..Default::default() });

    let msg = text(t!("nav_guard.message").to_string())
        .size(13)
        .color(Color::from_rgb(0.30, 0.30, 0.35));

    let separator = container(iced::widget::space().height(1))
        .width(Length::Fill)
        .height(Length::Fixed(1.0))
        .style(|_| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.88, 0.88, 0.90))),
            ..Default::default()
        });

    // Buttons — right-aligned: Cancel | Discard | Save
    let cancel_btn = button(text(t!("nav_guard.cancel").to_string()).size(13))
        .on_press(Message::NavGuardCancel)
        .padding(Padding::from([6.0, 14.0]))
        .style(iced::widget::button::secondary);

    let discard_btn = button(text(t!("nav_guard.discard_and_leave").to_string()).size(13))
        .on_press(Message::NavGuardDiscardAndLeave)
        .padding(Padding::from([6.0, 14.0]))
        .style(iced::widget::button::danger);

    let save_btn = button(text(t!("nav_guard.save_and_leave").to_string()).size(13))
        .on_press(Message::NavGuardSaveAndLeave)
        .padding(Padding::from([6.0, 14.0]));

    let actions = row![
        space().width(Length::Fill),
        cancel_btn,
        discard_btn,
        save_btn,
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let body = column![title, separator, msg, actions]
        .spacing(16)
        .width(Length::Fixed(420.0));

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
