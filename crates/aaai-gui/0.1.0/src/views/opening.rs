//! Opening / project-selection screen.

use iced::{
    Alignment::Center,
    Element, Length, Padding,
    widget::{button, column, container, row, space, text, text_input},
};

use crate::app::{App, Message};
use crate::style::card_style;

pub fn view(app: &App) -> Element<'_, Message> {
    let title = text("aaai")
        .size(48)
        .font(iced::Font { weight: iced::font::Weight::Bold, ..Default::default() });

    let subtitle = text("audit for asset integrity").size(16);

    let before_row = labeled_input(
        "Before folder",
        "Path to the source / expected folder",
        &app.before_path,
        Message::BeforePathChanged,
    );

    let after_row = labeled_input(
        "After folder",
        "Path to the target / actual folder",
        &app.after_path,
        Message::AfterPathChanged,
    );

    let def_row = labeled_input(
        "Audit definition",
        "Path to audit.yaml (leave empty to create new)",
        &app.definition_path,
        Message::DefinitionPathChanged,
    );

    let can_start = !app.before_path.trim().is_empty()
        && !app.after_path.trim().is_empty();

    let start_btn = button(
        text("Start Audit").size(15).font(iced::Font {
            weight: iced::font::Weight::Semibold,
            ..Default::default()
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
        let err_text = text(err.as_str())
            .size(13)
            .color(iced::Color::from_rgb(0.78, 0.10, 0.10));
        form_col = form_col.push(err_text);
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
    label: &'a str,
    placeholder: &'a str,
    value: &'a str,
    on_change: F,
) -> Element<'a, Message>
where
    F: Fn(String) -> Message + 'a,
{
    column![
        text(label).size(13).font(iced::Font {
            weight: iced::font::Weight::Semibold,
            ..Default::default()
        }),
        text_input(placeholder, value)
            .on_input(on_change)
            .padding(8),
    ]
    .spacing(4)
    .into()
}
