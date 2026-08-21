//! Unsaved-changes navigation guard dialog (RFC 041, RFC 086).
//!
//! Default state shows only the two safe choices:
//!   [Stay here]  [Save and leave]
//!
//! "Discard and leave" loses unsaved work, so it is hidden behind a
//! "More choices" link (RFC 086) — the safest action stays the easiest.
//!
//! The modal overlay (backdrop + centering) is assembled in `App::view()`.

use iced::{
    Color, Element, Length, Padding,
    widget::{button, column, container, row, space, text},
};
use rust_i18n::t;

use crate::app::Message;
use crate::style::{btn_primary, btn_secondary, btn_ghost, btn_danger};
use snora::design::Tokens;

pub fn view<'a>(show_discard: bool, tokens: &'a Tokens) -> Element<'a, Message> {
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

    // RFC 086 — primary row shows only safe choices: Stay | Save and leave.
    let t1 = tokens.clone();
    let cancel_btn = button(text(t!("nav_guard.cancel").to_string()).size(13))
        .on_press(Message::NavGuardCancel)
        .padding(Padding::from([6.0, 14.0]))
        .style(move |_theme, s| btn_secondary(&t1, s));

    let t2 = tokens.clone();
    let save_btn = button(text(t!("nav_guard.save_and_leave").to_string()).size(13))
        .on_press(Message::NavGuardSaveAndLeave)
        .padding(Padding::from([6.0, 14.0]))
        .style(move |_theme, s| btn_primary(&t2, s));

    // The data-losing action: either a quiet "More choices" link (hidden
    // state) or the actual danger button once revealed.
    let secondary: Element<'_, Message> = if show_discard {
        let t3 = tokens.clone();
        button(text(t!("nav_guard.discard_and_leave").to_string()).size(13))
            .on_press(Message::NavGuardDiscardAndLeave)
            .padding(Padding::from([6.0, 14.0]))
            .style(move |_theme, s| btn_danger(&t3, s))
            .into()
    } else {
        let t4 = tokens.clone();
        button(text(t!("nav_guard.more_choices").to_string()).size(12))
            .on_press(Message::NavGuardRevealDiscard)
            .padding(Padding::from([6.0, 10.0]))
            .style(move |_theme, s| btn_ghost(&t4, s))
            .into()
    };

    let actions = row![
        secondary,
        space().width(Length::Fill),
        cancel_btn,
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
