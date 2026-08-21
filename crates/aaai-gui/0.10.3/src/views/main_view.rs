//! Main 3-pane view — Phase 10: PaneGrid (resizable) + directory collapse.

use iced::{
    Color, Element, Length, Padding,
    widget::{
        button, checkbox, column, container, pane_grid,
        row, scrollable, space, text, text_input,
    },
};
use rust_i18n::t;

use aaai_core::{AuditStatus, DiffType, FileAuditResult};
use crate::app::{App, FilterMode, Message, PaneKind};
use crate::theme;
use crate::views::{dashboard, diff_view, inspector};

// ── Top-level view ───────────────────────────────────────────────────────────

pub fn view(app: &App) -> Element<'_, Message> {
    let toolbar  = build_toolbar(app);
    let filter_bar = build_filter_bar(app);
    let search_bar = build_search_bar(app);
    // ── PaneGrid ──────────────────────────────────────────────────────────
    let pg = pane_grid::PaneGrid::new(&app.panes, |_pane, kind, _is_maximized| {
        let content: Element<'_, Message> = match kind {
            PaneKind::FileTree => build_file_tree(app),
            PaneKind::Diff     => build_diff_panel(app),
            PaneKind::Inspector => build_inspector_panel(app),
        };
        pane_grid::Content::new(content)
    })
    .width(Length::Fill)
    .height(Length::Fill)
    .spacing(2)
    .on_resize(6, Message::PaneResized);

    column![
        toolbar,
        filter_bar,
        search_bar,
        pg,
    ]
    .spacing(0)
    .into()
}

// ── Toolbar ──────────────────────────────────────────────────────────────────

fn build_toolbar<'a>(app: &'a App) -> Element<'a, Message> {
    use crate::style::panel_style;

    let save_btn = button(text(t!("toolbar.save").to_string()).size(12))
        .on_press(Message::SaveDefinition)
        .padding(Padding::from([4.0, 10.0]));
    let rerun_btn = button(text(t!("toolbar.rerun").to_string()).size(12))
        .on_press(Message::RerunAudit)
        .padding(Padding::from([4.0, 10.0]));
    let batch_btn = button(text(t!("toolbar.batch_approve").to_string()).size(12))
        .on_press(Message::OpenBatchSheet)
        .padding(Padding::from([4.0, 10.0]));

    let verdict_section: Element<'_, Message> = if let Some(result) = &app.audit_result {
        let s = &result.summary;
        let verdict_str = if s.is_passing() {
            t!("toolbar.passed").to_string()
        } else {
            t!("toolbar.failed").to_string()
        };
        let verdict_color = if s.is_passing() { theme::OK_COLOR } else { theme::FAILED_COLOR };
        let warn_str = if s.warning_count > 0 {
            format!("  ⚠ {}", s.warning_count)
        } else {
            String::new()
        };
        row![
            colored_badge(verdict_str, verdict_color),
            text(format!(
                "  OK: {}  Pending: {}  Failed: {}  Error: {}{}",
                s.ok, s.pending, s.failed, s.error, warn_str
            )).size(12),
        ]
        .align_y(iced::Alignment::Center)
        .spacing(4)
        .into()
    } else {
        space().width(Length::Fill).into()
    };

    let report_md  = button(text("Export MD").size(11))
        .on_press(Message::ExportReport("markdown".into()))
        .padding(Padding::from([3.0, 7.0]));
    let report_json = button(text("Export JSON").size(11))
        .on_press(Message::ExportReport("json".into()))
        .padding(Padding::from([3.0, 7.0]));

    container(
        row![
            save_btn, rerun_btn, batch_btn,
            space().width(Length::Fixed(8.0)),
            verdict_section,
            space().width(Length::Fill),
            report_md, report_json,
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center)
        .padding(Padding::from([4.0, 8.0])),
    )
    .width(Length::Fill)
    .style(panel_style)
    .into()
}

// ── Filter bar ───────────────────────────────────────────────────────────────

fn build_filter_bar<'a>(app: &'a App) -> Element<'a, Message> {
    use crate::style::panel_style;

    let make_btn = |label: &'static str, mode: FilterMode| {
        let active = app.filter_mode == mode;
        let btn = button(text(t!(label).to_string()).size(11))
            .on_press(Message::SetFilter(mode))
            .padding(Padding::from([3.0, 8.0]));
        if active {
            container(btn)
                .style(|_| iced::widget::container::Style {
                    background: Some(iced::Background::Color(
                        Color::from_rgba(0.20, 0.45, 0.85, 0.18)
                    )),
                    border: iced::Border { radius: 4.0.into(), ..Default::default() },
                    ..Default::default()
                })
        } else {
            container(btn)
        }
    };

    let undo_btn = button(text(t!("toolbar.undo").to_string()).size(11))
        .on_press(Message::UndoApproval)
        .padding(Padding::from([3.0, 8.0]));

    container(
        row![
            make_btn("filter.all",           FilterMode::All),
            make_btn("filter.changed",        FilterMode::ChangedOnly),
            make_btn("filter.pending",        FilterMode::PendingOnly),
            make_btn("filter.errors",         FilterMode::FailedAndError),
            space().width(Length::Fill),
            undo_btn,
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center)
        .padding(Padding::from([3.0, 8.0])),
    )
    .width(Length::Fill)
    .style(panel_style)
    .into()
}

// ── Search bar ───────────────────────────────────────────────────────────────

fn build_search_bar<'a>(app: &'a App) -> Element<'a, Message> {
    if app.audit_result.is_none() {
        return space().height(0).into();
    }
    container(
        row![
            text("🔍").size(12),
            text_input("Search paths…", &app.search_query)
                .on_input(Message::SearchQueryChanged)
                .padding(Padding::from([3.0, 6.0]))
                .size(12)
                .width(Length::Fixed(220.0)),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
    )
    .padding(Padding::from([2.0, 8.0]))
    .width(Length::Fill)
    .style(|_| iced::widget::container::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.95, 0.96, 0.97))),
        ..Default::default()
    })
    .into()
}

// ── File tree pane ───────────────────────────────────────────────────────────

fn build_file_tree<'a>(app: &'a App) -> Element<'a, Message> {
    let result = match &app.audit_result {
        Some(r) => r,
        None    => return container(
            text("No audit result. Press \"Re-run\".").size(12)
                .color(Color::from_rgb(0.5, 0.5, 0.5))
        ).padding(12).into(),
    };

    let q = app.search_query.to_lowercase();

    // Collect visible entries with directory collapse support
    let mut items: Vec<Element<'_, Message>> = Vec::new();
    let mut prev_dir = String::new();

    for (idx, far) in result.results.iter().enumerate() {
        // Filter
        if !app.filter_mode.passes(far) { continue; }
        if far.diff.diff_type == DiffType::Unchanged { continue; }
        if !q.is_empty() && !far.diff.path.to_lowercase().contains(&q) { continue; }

        // Directory collapse
        let parts: Vec<&str> = far.diff.path.split('/').collect();
        let short = parts.last().copied().unwrap_or(&far.diff.path);
        let dir   = if parts.len() > 1 {
            parts[..parts.len()-1].join("/")
        } else {
            String::new()
        };
        let indent = (parts.len().saturating_sub(1)) as f32 * 14.0;

        // Insert directory header when dir changes
        if !dir.is_empty() && dir != prev_dir {
            let is_collapsed = app.collapsed_dirs.contains(&dir);
            let icon = if is_collapsed { "▶" } else { "▼" };
            let dir_clone = dir.clone();
            let dir_btn = button(
                row![
                    space().width(Length::Fixed((parts.len().saturating_sub(1)) as f32 * 14.0)),
                    text(format!("{icon} {}", parts[parts.len()-2])).size(11)
                        .color(Color::from_rgb(0.5, 0.5, 0.55))
                        .font(iced::Font { weight: iced::font::Weight::Semibold, ..Default::default() }),
                ]
                .spacing(4)
                .align_y(iced::Alignment::Center),
            )
            .on_press(Message::ToggleDir(dir_clone))
            .width(Length::Fill)
            .padding(Padding::from([2.0, 6.0]))
            .style(iced::widget::button::text);
            items.push(dir_btn.into());
            prev_dir = dir.clone();
        }

        // Skip collapsed children
        if !dir.is_empty() && app.collapsed_dirs.contains(&dir) { continue; }

        // Entry row
        items.push(build_file_row(app, idx, far, short, indent));
    }

    if items.is_empty() {
        return container(
            text("No entries match the current filter.").size(12)
                .color(Color::from_rgb(0.55, 0.55, 0.58))
        ).padding(12).into();
    }

    scrollable(
        column(items).spacing(0).width(Length::Fill)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn build_file_row<'a>(
    app: &'a App,
    idx: usize,
    far: &'a FileAuditResult,
    short: &'a str,
    indent: f32,
) -> Element<'a, Message> {
    let is_selected = app.selected_index == Some(idx);
    let is_batch    = app.batch.selected.contains(&idx);

    let status_color = match far.status {
        AuditStatus::Ok      => theme::OK_COLOR,
        AuditStatus::Pending => theme::PENDING_COLOR,
        AuditStatus::Failed  => theme::FAILED_COLOR,
        AuditStatus::Ignored => theme::IGNORED_COLOR,
        AuditStatus::Error   => theme::ERROR_COLOR,
    };
    let diff_icon = match far.diff.diff_type {
        DiffType::Added        => "+",
        DiffType::Removed      => "−",
        DiffType::Modified     => "~",
        DiffType::TypeChanged  => "T",
        DiffType::Unreadable   => "!",
        DiffType::Incomparable => "?",
        DiffType::Unchanged    => " ",
    };

    let status_badge = colored_badge(diff_icon.to_string(), status_color);

    // Warning badge
    let warn_badge: Option<Element<'_, Message>> = if !far.warnings.is_empty() {
        Some(
            container(
                text(format!("⚠{}", far.warnings.len())).size(9)
                    .color(Color::from_rgb(0.60, 0.40, 0.00))
            )
            .padding(Padding::from([1.0, 3.0]))
            .style(|_| iced::widget::container::Style {
                background: Some(iced::Background::Color(
                    Color::from_rgba(0.95, 0.85, 0.20, 0.25)
                )),
                border: iced::Border {
                    color: Color::from_rgba(0.85, 0.65, 0.10, 0.50),
                    width: 1.0,
                    radius: 3.0.into(),
                },
                ..Default::default()
            })
            .into()
        )
    } else { None };

    let batch_cb = checkbox(is_batch)
        .on_toggle(move |_| Message::ToggleBatchSelect(idx))
        .size(14);

    let mut name_row = row![
        space().width(Length::Fixed(indent)),
        status_badge,
        text(short).size(12).font(iced::Font::MONOSPACE),
    ]
    .spacing(4)
    .align_y(iced::Alignment::Center);
    if let Some(wb) = warn_badge {
        name_row = name_row.push(wb);
    }

    let full_row = row![batch_cb, name_row].spacing(4).align_y(iced::Alignment::Center);

    let bg = move |_: &iced::Theme| iced::widget::container::Style {
        background: if is_selected {
            Some(iced::Background::Color(Color::from_rgba(0.15, 0.45, 0.85, 0.18)))
        } else { None },
        ..Default::default()
    };

    button(
        container(full_row)
            .width(Length::Fill)
            .padding(Padding::from([3.0, 6.0]))
            .style(bg),
    )
    .on_press(Message::SelectEntry(idx))
    .width(Length::Fill)
    .padding(0)
    .style(iced::widget::button::text)
    .into()
}

// ── Diff pane ────────────────────────────────────────────────────────────────

fn build_diff_panel<'a>(app: &'a App) -> Element<'a, Message> {
    match app.selected_index {
        Some(idx) => {
            if let Some(result) = &app.audit_result {
                if let Some(far) = result.results.get(idx) {
                    return diff_view::view(&far.diff);
                }
            }
        }
        None => {}
    }
    match &app.audit_result {
        Some(r) => dashboard::view(r),
        None    => container(
            text("No data. Start an audit from the Opening screen.")
                .size(13).color(Color::from_rgb(0.5, 0.5, 0.5))
        ).padding(20).into(),
    }
}

// ── Inspector pane ───────────────────────────────────────────────────────────

fn build_inspector_panel<'a>(app: &'a App) -> Element<'a, Message> {
    match app.selected_index {
        Some(idx) => {
            if let Some(result) = &app.audit_result {
                if let Some(far) = result.results.get(idx) {
                    return inspector::view(app, far);
                }
            }
        }
        None => {}
    }
    container(
        text("Select a file to inspect.").size(12)
            .color(Color::from_rgb(0.55, 0.55, 0.60))
    )
    .padding(16)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn colored_badge(label: String, color: Color) -> Element<'static, Message> {
    container(
        text(label).size(11)
            .font(iced::Font { weight: iced::font::Weight::Semibold, ..Default::default() })
            .color(Color::WHITE)
    )
    .padding(Padding::from([2.0, 5.0]))
    .style(move |_| iced::widget::container::Style {
        background: Some(iced::Background::Color(color)),
        border: iced::Border { radius: 4.0.into(), ..Default::default() },
        ..Default::default()
    })
    .into()
}
