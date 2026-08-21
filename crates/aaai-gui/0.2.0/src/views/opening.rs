//! Opening / project-selection screen (Phase 2: i18n).

use iced::{
    Alignment::Center, Element, Length, Padding,
    widget::{button, column, container, row, space, text, text_input},
};
use rust_i18n::t;

use crate::app::{App, Message};
use crate::style::card_style;

pub fn view(app: &App) -> Element<'_, Message> {
    let title = text(t!("opening.title").to_string()).size(48).font(iced::Font {
        weight: iced::font::Weight::Bold, ..Default::default()
    });
    let subtitle = text(t!("opening.subtitle").to_string()).size(16);

    let before_row = labeled_input(
        t!("opening.before_label").to_string(),
        t!("opening.before_placeholder").to_string(),
        &app.before_path,
        Message::BeforePathChanged,
    );
    let after_row = labeled_input(
        t!("opening.after_label").to_string(),
        t!("opening.after_placeholder").to_string(),
        &app.after_path,
        Message::AfterPathChanged,
    );
    let def_row = labeled_input(
        t!("opening.definition_label").to_string(),
        t!("opening.definition_placeholder").to_string(),
        &app.definition_path,
        Message::DefinitionPathChanged,
    );

    let can_start = !app.before_path.trim().is_empty()
        && !app.after_path.trim().is_empty();

    let start_btn = button(
        text(t!("opening.start_button").to_string()).size(15).font(iced::Font {
            weight: iced::font::Weight::Semibold, ..Default::default()
        }),
    )
    .on_press_maybe(if can_start { Some(Message::StartAudit) } else { None })
    .padding(Padding::from([10.0, 32.0]));

    let mut form_col = column![
        before_row,
        after_row,
        def_row,
        space().height(Length::Fixed(8.0)),
        row![start_btn].spacing(12),
    ]
    .spacing(16)
    .width(Length::Fixed(560.0));

    if let Some(err) = &app.open_error {
        form_col = form_col.push(
            text(err.clone()).size(13)
                .color(iced::Color::from_rgb(0.78, 0.10, 0.10)),
        );
    }

    let card = container(
        column![
            title,
            subtitle,
            space().height(Length::Fixed(24.0)),
            form_col,
        ]
        .spacing(6)
        .align_x(Center),
    )
    .padding(Padding::from([40.0, 48.0]))
    .style(card_style);

    container(card)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

fn labeled_input<'a, F>(
    label: String,
    placeholder: String,
    value: &'a str,
    on_change: F,
) -> Element<'a, Message>
where
    F: Fn(String) -> Message + 'a,
{
    column![
        text(label).size(13).font(iced::Font {
            weight: iced::font::Weight::Semibold, ..Default::default()
        }),
        text_input(placeholder.as_str(), value)
            .on_input(on_change)
            .padding(8),
    ]
    .spacing(4)
    .into()
}
