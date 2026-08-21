//! Main screen — 3-pane layout: file tree | diff view | inspector.

use iced::{
    Color, Element, Length, Padding,
    widget::{button, column, container, row, scrollable, space, text},
};

use aaai_core::{AuditStatus, DiffType, FileAuditResult};
use crate::app::{App, Message};
use crate::style::panel_style;
use crate::theme;
use crate::views::{diff_view, inspector};

pub fn view(app: &App) -> Element<'_, Message> {
    let toolbar = build_toolbar(app);
    let file_tree = build_file_tree(app);

    let center_and_inspector: Element<'_, Message> = if let Some(idx) = app.selected_index {
        if let Some(result) = &app.audit_result {
            if let Some(far) = result.results.get(idx) {
                let diff_panel = container(diff_view::view(&far.diff))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(panel_style);

                let inspector_panel = inspector::view(app, far);

                row![diff_panel, inspector_panel]
                    .spacing(1)
                    .height(Length::Fill)
                    .into()
            } else {
                empty_center()
            }
        } else {
            empty_center()
        }
    } else {
        empty_center()
    };

    let body = column![
        toolbar,
        row![
            file_tree,
            center_and_inspector,
        ]
        .spacing(1)
        .height(Length::Fill),
    ]
    .spacing(0)
    .height(Length::Fill);

    container(body)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
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
    let save_btn = button(text("Save").size(13))
        .on_press(Message::SaveDefinition)
        .padding(Padding::from([5.0, 12.0]));

    let rerun_btn = button(text("Re-run Audit").size(13))
        .on_press(Message::RerunAudit)
        .padding(Padding::from([5.0, 12.0]));

    let report_md_btn = button(text("Export MD").size(13))
        .on_press(Message::ExportReport("markdown".into()))
        .padding(Padding::from([5.0, 12.0]));

    let report_json_btn = button(text("Export JSON").size(13))
        .on_press(Message::ExportReport("json".into()))
        .padding(Padding::from([5.0, 12.0]));

    // Summary badge row
    let summary_text: Element<'_, Message> = if let Some(r) = &app.audit_result {
        let s = &r.summary;
        let verdict_color = if s.is_passing() {
            theme::OK_COLOR
        } else {
            theme::FAILED_COLOR
        };
        let verdict_str = if s.is_passing() { "PASSED" } else { "FAILED" };

        row![
            colored_badge(verdict_str, verdict_color),
            text(format!(
                "  OK: {}  Pending: {}  Failed: {}  Error: {}",
                s.ok, s.pending, s.failed, s.error
            ))
            .size(12),
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
            save_btn,
            rerun_btn,
            report_md_btn,
            report_json_btn,
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .padding(Padding::from([6.0, 12.0]))
    .style(panel_style)
    .into()
}

fn build_file_tree<'a>(app: &'a App) -> Element<'a, Message> {
    let results: &[FileAuditResult] = app
        .audit_result
        .as_ref()
        .map(|r| r.results.as_slice())
        .unwrap_or(&[]);

    let mut items: Vec<Element<'_, Message>> = Vec::new();

    for (idx, far) in results.iter().enumerate() {
        if far.diff.diff_type == DiffType::Unchanged {
            continue;
        }

        let is_selected = app.selected_index == Some(idx);

        let status_color = match far.status {
            AuditStatus::Ok => theme::OK_COLOR,
            AuditStatus::Pending => theme::PENDING_COLOR,
            AuditStatus::Failed => theme::FAILED_COLOR,
            AuditStatus::Ignored => theme::IGNORED_COLOR,
            AuditStatus::Error => theme::ERROR_COLOR,
        };

        let diff_icon = match far.diff.diff_type {
            DiffType::Added => "+",
            DiffType::Removed => "-",
            DiffType::Modified => "~",
            DiffType::TypeChanged => "T",
            DiffType::Unreadable => "!",
            DiffType::Incomparable => "?",
            DiffType::Unchanged => " ",
        };

        let status_dot = colored_badge(diff_icon, status_color);

        let name = {
            let parts: Vec<&str> = far.diff.path.split('/').collect();
            let short = parts.last().copied().unwrap_or(&far.diff.path);
            let indent = (parts.len().saturating_sub(1)) as f32 * 12.0;
            row![
                space().width(Length::Fixed(indent)),
                status_dot,
                text(short).size(12).font(iced::Font::MONOSPACE),
            ]
            .spacing(4)
            .align_y(iced::Alignment::Center)
        };

        let row_style = move |_: &iced::Theme| -> iced::widget::container::Style {
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
            container(name)
                .width(Length::Fill)
                .padding(Padding::from([3.0, 8.0]))
                .style(row_style),
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

    let tree_col = column(items).spacing(0).width(Length::Fill);

    scrollable(
        container(tree_col)
            .width(Length::Fixed(240.0))
            .padding(Padding::from([4.0, 0.0])),
    )
    .width(Length::Fixed(240.0))
    .height(Length::Fill)
    .into()
}

fn colored_badge<'a>(label: &'a str, color: Color) -> Element<'a, Message> {
    container(text(label).size(10).color(Color::WHITE))
        .padding(Padding::from([1.0, 4.0]))
        .style(move |_| iced::widget::container::Style {
            background: Some(iced::Background::Color(color)),
            border: iced::Border { radius: 3.0.into(), ..Default::default() },
            ..Default::default()
        })
        .into()
}
