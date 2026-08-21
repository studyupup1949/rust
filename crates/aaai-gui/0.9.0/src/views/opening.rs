//! Opening screen (Phase 3: profiles, ignore path).

use iced::{
    Alignment::Center, Color, Element, Length, Padding,
    widget::{button, column, container, row, scrollable, space, text, text_input},
};
use rust_i18n::t;

use crate::app::{App, Message};
use crate::style::card_style;

pub fn view(app: &App) -> Element<'_, Message> {
    let title    = text(t!("opening.title").to_string()).size(48).font(iced::Font {
        weight: iced::font::Weight::Bold, ..Default::default()
    });
    let subtitle = text(t!("opening.subtitle").to_string()).size(16);

    let before_row = labeled_input(t!("opening.before_label").to_string(),
        t!("opening.before_placeholder").to_string(), &app.before_path, Message::BeforePathChanged);
    let after_row = labeled_input(t!("opening.after_label").to_string(),
        t!("opening.after_placeholder").to_string(), &app.after_path, Message::AfterPathChanged);
    let def_row = labeled_input(t!("opening.definition_label").to_string(),
        t!("opening.definition_placeholder").to_string(), &app.definition_path, Message::DefinitionPathChanged);
    let ignore_row = labeled_input(
        ".aaaiignore file (省略可)".to_string(),
        "除外パターンファイルのパス（省略時: Before/.aaaiignore を自動検索）".to_string(),
        &app.ignore_path, Message::IgnorePathChanged);

    let can_start = !app.before_path.trim().is_empty() && !app.after_path.trim().is_empty();

    let start_btn = button(
        text(t!("opening.start_button").to_string()).size(15)
            .font(iced::Font { weight: iced::font::Weight::Semibold, ..Default::default() }),
    )
    .on_press_maybe(if can_start { Some(Message::StartAudit) } else { None })
    .padding(Padding::from([10.0, 32.0]));

    // Profile save row
    let profile_row = row![
        text_input("Profile name", &app.profile_name_input)
            .on_input(Message::ProfileNameChanged)
            .padding(6)
            .width(Length::Fixed(160.0)),
        button(text(t!("profile.save_current").to_string()).size(12))
            .on_press(Message::SaveProfile)
            .padding(Padding::from([6.0, 12.0])),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let mut form_col = column![
        before_row, after_row, def_row, ignore_row,
        space().height(Length::Fixed(8.0)),
        row![start_btn].spacing(12),
        space().height(Length::Fixed(4.0)),
        profile_row,
    ]
    .spacing(14)
    .width(Length::Fixed(560.0));

    if let Some(err) = &app.open_error {
        form_col = form_col.push(
            text(err.clone()).size(13).color(Color::from_rgb(0.78, 0.10, 0.10)),
        );
    }

    // Saved profiles list
    let profiles_section: Element<'_, Message> = if !app.profiles.profiles.is_empty() {
        let mut p_col = column![
            text(t!("profile.title").to_string()).size(14)
                .font(iced::Font { weight: iced::font::Weight::Semibold, ..Default::default() }),
        ].spacing(6);

        for (i, profile) in app.profiles.profiles.iter().enumerate() {
            let load_btn = button(text(t!("profile.load").to_string()).size(12))
                .on_press(Message::LoadProfile(i))
                .padding(Padding::from([4.0, 10.0]));
            let del_btn = button(text(t!("profile.delete").to_string()).size(11))
                .on_press(Message::DeleteProfile(i))
                .padding(Padding::from([4.0, 8.0]));

            p_col = p_col.push(
                row![
                    column![
                        text(profile.name.clone()).size(13)
                            .font(iced::Font { weight: iced::font::Weight::Semibold, ..Default::default() }),
                        text(profile.before.clone()).size(11).color(Color::from_rgb(0.5,0.5,0.5)),
                    ].spacing(2).width(Length::Fill),
                    load_btn, del_btn,
                ]
                .spacing(6)
                .align_y(iced::Alignment::Center),
            );
        }

        scrollable(container(p_col).width(Length::Fixed(560.0)).padding(8.0))
            .height(Length::Fixed(140.0))
            .into()
    } else {
        space().height(Length::Fixed(0.0)).into()
    };

    let card = container(
        column![
            title, subtitle,
            space().height(Length::Fixed(20.0)),
            form_col,
            space().height(Length::Fixed(8.0)),
            profiles_section,
        ]
        .spacing(6)
        .align_x(Center),
    )
    .padding(Padding::from([36.0, 48.0]))
    .style(card_style);

    // Phase 8: show loading overlay when diff is in progress
    if app.is_loading {
        let progress_msg = app.load_progress.as_deref().unwrap_or("フォルダを比較中…");
        let before_short = std::path::Path::new(&app.before_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| app.before_path.clone());
        let after_short = std::path::Path::new(&app.after_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| app.after_path.clone());

        let spinner = container(
            column![
                text("⟳").size(40).color(iced::Color::from_rgb(0.3, 0.5, 0.9)),
                text(progress_msg.to_owned()).size(15)
                    .color(iced::Color::from_rgb(0.3, 0.3, 0.4)),
                text(format!("{} → {}", before_short, after_short)).size(12)
                    .color(iced::Color::from_rgb(0.55, 0.55, 0.60)),
            ]
            .spacing(10)
            .align_x(iced::Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill);
        return spinner.into();
    }

    container(card)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

fn labeled_input<'a, F>(
    label: String, placeholder: String, value: &'a str, on_change: F,
) -> Element<'a, Message>
where F: Fn(String) -> Message + 'a {
    column![
        text(label).size(13).font(iced::Font { weight: iced::font::Weight::Semibold, ..Default::default() }),
        text_input(placeholder.as_str(), value).on_input(on_change).padding(8),
    ].spacing(4).into()
}
