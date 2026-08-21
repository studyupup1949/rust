//! Main 3-pane screen (Phase 2: filter bar, batch select, i18n).

use iced::{
    Color, Element, Length, Padding,
    widget::{button, checkbox, column, container,
             row, scrollable, space, text},
};

use aaai_core::{AuditStatus, DiffType, FileAuditResult};
use crate::app::{App, FilterMode, Message};
use crate::style::panel_style;
use crate::theme;
use crate::views::{dashboard, diff_view, inspector};
use rust_i18n::t;

pub fn view(app: &App) -> Element<'_, Message> {
    let toolbar = build_toolbar(app);
    let filter_bar = build_filter_bar(app);
    let file_tree = build_file_tree(app);

    // Search bar
    let search_bar = build_search_bar(app);

    let center_and_inspector: Element<'_, Message> =
        if let Some(idx) = app.selected_index {
            if let Some(far) = app.audit_result.as_ref()
                .and_then(|r| r.results.get(idx))
            {
                let diff_panel = container(diff_view::view(&far.diff))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(panel_style);
                let insp = inspector::view(app, far);
                row![diff_panel, insp]
                    .spacing(1)
                    .height(Length::Fill)
                    .into()
            } else { empty_center() }
        } else { empty_center() };

    let body = column![
        toolbar,
        filter_bar,
        search_bar,
        row![file_tree, center_and_inspector]
            .spacing(1)
            .height(Length::Fill),
    ]
    .spacing(0)
    .height(Length::Fill);

    container(body).width(Length::Fill).height(Length::Fill).into()
}

fn empty_center<'a>() -> Element<'a, Message> {
    container(
        text("Select a file from the left panel to inspect it.")
            .size(13)
            .color(Color::from_rgb(0.5, 0.5, 0.5)),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

fn build_toolbar<'a>(app: &'a App) -> Element<'a, Message> {
    let batch_count = app.batch.selected.len();

    let save_btn = button(text(t!("toolbar.save")).size(13))
        .on_press(Message::SaveDefinition)
        .padding(Padding::from([5.0, 12.0]));

    let rerun_btn = button(text(t!("toolbar.rerun")).size(13))
        .on_press(Message::RerunAudit)
        .padding(Padding::from([5.0, 12.0]));

    let report_md_btn = button(text(t!("toolbar.export_md")).size(13))
        .on_press(Message::ExportReport("markdown".into()))
        .padding(Padding::from([5.0, 12.0]));

    let report_json_btn = button(text(t!("toolbar.export_json")).size(13))
        .on_press(Message::ExportReport("json".into()))
        .padding(Padding::from([5.0, 12.0]));

    let batch_label = format!("{} ({})", t!("toolbar.batch_approve"), batch_count);
    let batch_btn = button(text(batch_label).size(13))
        .on_press_maybe(
            if batch_count > 0 { Some(Message::OpenBatchSheet) } else { None },
        )
        .padding(Padding::from([5.0, 12.0]));

    let summary_text: Element<'_, Message> =
        if let Some(r) = &app.audit_result {
            let s = &r.summary;
            let verdict_color = if s.is_passing() { theme::OK_COLOR } else { theme::FAILED_COLOR };
            let verdict_str: String = if s.is_passing() {
                t!("status.passed").to_string()
            } else {
                t!("status.result_failed").to_string()
            };
            row![
                colored_badge(verdict_str, verdict_color),
                text(format!(
                    "  OK: {}  Pending: {}  Failed: {}  Error: {}",
                    s.ok, s.pending, s.failed, s.error
                )).size(12),
            ]
            .align_y(iced::Alignment::Center)
            .spacing(4)
            .into()
        } else {
            text("").size(12).into()
        };

    container(
        row![
            summary_text,
            space().width(Length::Fill),
            batch_btn,
            save_btn,
            rerun_btn,
            report_md_btn,
            report_json_btn,
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .padding(Padding::from([6.0, 12.0]))
    .style(panel_style)
    .into()
}

fn build_filter_bar<'a>(app: &'a App) -> Element<'a, Message> {
    let filter_data: Vec<(FilterMode, String)> = vec![
        (FilterMode::ChangedOnly,    t!("filter.changed").to_string()),
        (FilterMode::All,            t!("filter.all").to_string()),
        (FilterMode::PendingOnly,    t!("filter.pending").to_string()),
        (FilterMode::FailedAndError, t!("filter.errors").to_string()),
    ];

    let mut btns = row![].spacing(4);
    for (mode, label) in &filter_data {
        let is_active = app.filter_mode == *mode;
        let btn = button(text(label.clone()).size(12))
            .on_press(Message::SetFilter(*mode))
            .padding(Padding::from([3.0, 10.0]))
            .style(if is_active {
                iced::widget::button::primary
            } else {
                iced::widget::button::secondary
            });
        btns = btns.push(btn);
    }

    container(btns)
        .width(Length::Fill)
        .padding(Padding::from([4.0, 12.0]))
        .style(|_| iced::widget::container::Style {
            background: Some(iced::Background::Color(
                Color::from_rgb(0.93, 0.94, 0.96),
            )),
            ..Default::default()
        })
        .into()
}

fn build_file_tree<'a>(app: &'a App) -> Element<'a, Message> {
    let results: &[FileAuditResult] = app.audit_result
        .as_ref()
        .map(|r| r.results.as_slice())
        .unwrap_or(&[]);

    let mut items: Vec<Element<'_, Message>> = Vec::new();

    for (idx, far) in results.iter().enumerate() {
        if !app.filter_mode.passes(far) {
            continue;
        }
        // Search filter
        if !app.search_query.is_empty() {
            let q = app.search_query.to_lowercase();
            if !far.diff.path.to_lowercase().contains(&q) {
                continue;
            }
        }

        let is_selected = app.selected_index == Some(idx);
        let is_batch_selected = app.batch.selected.contains(&idx);

        let status_color = match far.status {
            AuditStatus::Ok      => theme::OK_COLOR,
            AuditStatus::Pending => theme::PENDING_COLOR,
            AuditStatus::Failed  => theme::FAILED_COLOR,
            AuditStatus::Ignored => theme::IGNORED_COLOR,
            AuditStatus::Error   => theme::ERROR_COLOR,
        };

        let diff_icon = match far.diff.diff_type {
            DiffType::Added        => "+",
            DiffType::Removed      => "-",
            DiffType::Modified     => "~",
            DiffType::TypeChanged  => "T",
            DiffType::Unreadable   => "!",
            DiffType::Incomparable => "?",
            DiffType::Unchanged    => " ",
        };

        let status_badge = colored_badge(diff_icon.to_string(), status_color);

        let parts: Vec<&str> = far.diff.path.split('/').collect();
        let short = parts.last().copied().unwrap_or(&far.diff.path);
        let indent = (parts.len().saturating_sub(1)) as f32 * 12.0;

        // Batch checkbox
        let batch_cb = checkbox(is_batch_selected)
            .on_toggle(move |_| Message::ToggleBatchSelect(idx))
            .size(14);

        let name_row = row![
            space().width(Length::Fixed(indent)),
            status_badge,
            text(short).size(12).font(iced::Font::MONOSPACE),
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center);

        let full_row = row![batch_cb, name_row].spacing(4).align_y(iced::Alignment::Center);

        let row_bg = move |_: &iced::Theme| -> iced::widget::container::Style {
            iced::widget::container::Style {
                background: if is_selected {
                    Some(iced::Background::Color(Color::from_rgba(0.15, 0.45, 0.85, 0.18)))
                } else {
                    None
                },
                ..Default::default()
            }
        };

        let item = button(
            container(full_row)
                .width(Length::Fill)
                .padding(Padding::from([3.0, 6.0]))
                .style(row_bg),
        )
        .on_press(Message::SelectEntry(idx))
        .width(Length::Fill)
        .padding(0)
        .style(|_theme, _status| iced::widget::button::Style {
            background: None,
            ..Default::default()
        });

        items.push(item.into());
    }

    scrollable(
        container(column(items).spacing(0).width(Length::Fill))
            .width(Length::Fixed(260.0))
            .padding(Padding::from([4.0, 0.0])),
    )
    .width(Length::Fixed(260.0))
    .height(Length::Fill)
    .into()
}

fn build_search_bar<'a>(app: &'a crate::app::App) -> Element<'a, Message> {
    use iced::widget::{container, text_input};
    use crate::app::Message;
    if app.audit_result.is_none() {
        return space().height(Length::Fixed(0.0)).into();
    }
    container(
        row![
            text("🔍").size(12),
            text_input("Search paths…", &app.search_query)
                .on_input(Message::SearchQueryChanged)
                .padding(Padding::from([4.0, 8.0]))
                .size(12)
                .width(Length::Fixed(220.0)),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
    )
    .padding(Padding::from([3.0, 8.0]))
    .width(Length::Fill)
    .style(|_| iced::widget::container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgb(0.95, 0.96, 0.97))),
        ..Default::default()
    })
    .into()
}

fn colored_badge(label: String, color: Color) -> Element<'static, Message> {
    container(text(label).size(10).color(Color::WHITE))
        .padding(Padding::from([1.0, 4.0]))
        .style(move |_| iced::widget::container::Style {
            background: Some(iced::Background::Color(color)),
            border: iced::Border { radius: 3.0.into(), ..Default::default() },
            ..Default::default()
        })
        .into()
}
