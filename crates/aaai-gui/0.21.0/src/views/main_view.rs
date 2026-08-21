//! Main 3-pane view — Phase 10: PaneGrid (resizable) + directory collapse.

use iced::{
    Color, Element, Length, Padding,
    widget::{
        button, column, container, pane_grid,
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
    let toolbar    = build_toolbar(app);
    let filter_bar = build_filter_bar(app);
    let search_bar = build_search_bar(app);
    let bottom_bar = build_bottom_bar(app);   // RFC 008
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
        bottom_bar,   // RFC 008: fixed bottom action bar
    ]
    .spacing(0)
    .into()
}

// ── Toolbar ──────────────────────────────────────────────────────────────────

fn build_toolbar<'a>(app: &'a App) -> Element<'a, Message> {
    // RFC 007 + RFC 014: Design-doc toolbar
    //  [ □ 開く ] [ □ 保存 ] [ ▶ 監査実行 ] [ ↑ レポート出力 ]   監査ステータス: XX
    use crate::style::panel_style;

    let toolbar_btn = |icon: String, label: String, msg: Message| -> Element<'_, Message> {
        button(
            row![
                text(icon).size(12),
                text(label).size(12),
            ]
            .spacing(4)
            .align_y(iced::Alignment::Center)
        )
        .on_press(msg)
        .padding(Padding::from([10.0, 16.0]))
        .into()
    };

    let open_btn   = toolbar_btn("□".into(), t!("toolbar.open").to_string(),          Message::BackToOpening);
    let save_btn   = toolbar_btn("□".into(), t!("toolbar.save").to_string(),          Message::SaveDefinition);
    let run_btn    = toolbar_btn("▶".into(), t!("toolbar.run_audit").to_string(),     Message::RerunAudit);
    let report_btn = toolbar_btn("↑".into(), t!("toolbar.report_output").to_string(), Message::ExportReport("markdown".into()));

    // RFC 021 §2.3 — small "✓ Saved Nm ago" / "✓ Reported Nm ago" marks
    // next to Save and Report buttons when those operations have happened
    // at least once. These persist (no auto-dismiss like toasts) so the
    // user can always tell at a glance how fresh the on-disk state is.
    // The relative time updates via the 30-second tick subscription.
    let save_with_mark: Element<'_, Message> = if let Some(t_saved) = app.last_saved_at {
        row![
            save_btn,
            text(format!("✓ {} {}",
                t!("banner.saved_label"),
                crate::util::humanize_since(t_saved),
            ))
            .size(10)
            .color(Color::from_rgb(0.40, 0.55, 0.40)),
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center)
        .into()
    } else {
        save_btn
    };
    let report_with_mark: Element<'_, Message> = if let Some(t_rep) = app.last_reported_at {
        row![
            report_btn,
            text(format!("✓ {} {}",
                t!("banner.reported_label"),
                crate::util::humanize_since(t_rep),
            ))
            .size(10)
            .color(Color::from_rgb(0.40, 0.55, 0.40)),
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center)
        .into()
    } else {
        report_btn
    };

    // Audit status label — right-aligned simple text
    let status_label: Element<'_, Message> = if let Some(result) = &app.audit_result {
        let s = &result.summary;
        let (label, color) = if s.is_passing() {
            (t!("toolbar.passed").to_string(), theme::OK_COLOR)
        } else {
            (t!("toolbar.failed").to_string(), theme::FAILED_COLOR)
        };
        text(format!("{}: {}", t!("toolbar.audit_status"), label))
            .size(13).color(color)
            .into()
    } else {
        space().width(Length::Fill).into()
    };

    container(
        row![
            open_btn, save_with_mark, run_btn, report_with_mark,
            space().width(Length::Fill),
            status_label,
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center)
        .padding(Padding::from([4.0, 10.0])),
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
            .padding(Padding::from([10.0, 14.0]));  // ABDD ≥44px
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
        .padding(Padding::from([10.0, 14.0]));  // ABDD ≥44px

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
    // RFC 005: focus management is handled by iced's `id`/`focus` plumbing,
    // not by per-state placeholder text. RFC 032: i18n migration. Both
    // branches of the previous `if focus == Search` produced the same
    // string, so the conditional is collapsed.
    let search_placeholder = t!("main.search_placeholder").to_string();
    container(
        row![
            text("🔍").size(12),
            text_input(&search_placeholder, &app.search_query)
                .on_input(Message::SearchQueryChanged)
                .padding(Padding::from([3.0, 6.0]))
                .size(12)
                .width(Length::Fixed(220.0)),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
    )
    .padding(Padding::from([10.0, 14.0]))  // ABDD ≥44px
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
        None    => return empty_state_file_tree(),
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
            .padding(Padding::from([4.0, 6.0]))  // ABDD minimum
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
            text(t!("empty_state.no_entries_match_filter").to_string()).size(12)
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
    let _is_batch   = app.batch.selected.contains(&idx);  // batch UI removed from toolbar (RFC 007)

    // RFC 013: row layout — left: status_icon, middle: path, right: diff_type_tag
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

    let sicon = status_icon(far.status);
    let dtype_tag = diff_type_tag(far.diff.diff_type);
    let mut name_row = row![
        space().width(Length::Fixed(indent)),
        sicon,
        text(short).size(12).font(iced::Font::MONOSPACE),
    ]
    .spacing(5)
    .align_y(iced::Alignment::Center);
    if let Some(wb) = warn_badge {
        name_row = name_row.push(wb);
    }

    let full_row = row![
        name_row,
        space().width(Length::Fill),
        dtype_tag,
    ]
    .spacing(4)
    .align_y(iced::Alignment::Center);

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
                    return diff_view::view(&far.diff, app.diff_view_mode);
                }
            }
        }
        None => {}
    }
    match &app.audit_result {
        Some(r) => dashboard::view(r),
        None    => empty_state_diff_panel(),
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
    empty_state_inspector()
}

// ── Helpers ──────────────────────────────────────────────────────────────────


// RFC 013: single status icon — symbol + colour only, no text label.
fn status_icon(status: AuditStatus) -> Element<'static, Message> {
    let (sym, color) = match status {
        AuditStatus::Ok      => ("✓", theme::OK_COLOR),
        AuditStatus::Pending => ("⚠", theme::PENDING_COLOR),
        AuditStatus::Failed  => ("✗", theme::FAILED_COLOR),
        AuditStatus::Error   => ("!", theme::ERROR_COLOR),
        AuditStatus::Ignored => ("—", iced::Color::from_rgb(0.65, 0.65, 0.68)),
    };
    text(sym).size(13).color(color).into()
}

// RFC 013: diff-type tag — right-aligned subtle grey symbol.
fn diff_type_tag(dtype: DiffType) -> Element<'static, Message> {
    let sym = match dtype {
        DiffType::Added        => "+",
        DiffType::Removed      => "−",
        DiffType::Modified     => "~",
        DiffType::TypeChanged  => "T",
        DiffType::Unchanged    => " ",
        DiffType::Unreadable   => "!",
        DiffType::Incomparable => "?",
    };
    text(sym).size(11)
        .color(iced::Color::from_rgb(0.60, 0.62, 0.66))
        .into()
}



// ── Bottom action bar (RFC 008) ───────────────────────────────────────────────

fn build_bottom_bar<'a>(app: &'a App) -> Element<'a, Message> {
    use crate::style::panel_style;

    // "承認して保存" button — enabled only when an entry is selected and valid
    let can_approve = app.selected_index.is_some()
        && app.inspector.validation.can_approve();

    let approve_btn = button(
        text(t!("bottombar.approve_and_save").to_string())
            .size(13)
            .font(iced::Font {
                weight: iced::font::Weight::Semibold,
                ..Default::default()
            }),
    )
    .on_press_maybe(if can_approve { Some(Message::ApproveAndSave) } else { None })
    .padding(Padding::from([10.0, 20.0]));  // ABDD ≥44px

    // Selected file label
    let selected_label: Element<'_, Message> = if let Some(idx) = app.selected_index {
        if let Some(r) = app.audit_result.as_ref().and_then(|r| r.results.get(idx)) {
            text(format!("{}  {}", t!("bottombar.selected"), r.diff.path))
                .size(12)
                .color(iced::Color::from_rgb(0.40, 0.42, 0.48))
                .into()
        } else {
            space().width(Length::Fill).into()
        }
    } else {
        space().width(Length::Fill).into()
    };

    // Unresolved count label
    let count_label: Element<'_, Message> = if let Some(s) =
        app.audit_result.as_ref().map(|r| &r.summary)
    {
        let unresolved = s.failed + s.pending + s.error;
        let color = if unresolved > 0 {
            iced::Color::from_rgb(0.75, 0.30, 0.10)
        } else {
            iced::Color::from_rgb(0.15, 0.60, 0.25)
        };
        text(format!("{}件の差分中 {}件が未解決", s.total, unresolved))
            .size(12)
            .color(color)
            .into()
    } else {
        space().width(Length::Fill).into()
    };

    container(
        row![
            approve_btn,
            space().width(Length::Fixed(16.0)),
            selected_label,
            space().width(Length::Fill),
            count_label,
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center)
        .padding(Padding::from([5.0, 12.0])),
    )
    .width(Length::Fill)
    .style(panel_style)
    .into()
}

// ── RFC 022: empty-state panels ──────────────────────────────────────────────
//
// Used when `audit_result` is None (file_tree / diff_panel) or when nothing
// is selected (inspector). All three follow the same visual contract:
// transparent background, soft 1-px border, mid-grey text. The guidance
// text is i18n-driven via the `empty_state.*` namespace so en/ja produce
// equivalent prose.

fn empty_state_file_tree<'a>() -> Element<'a, Message> {
    use crate::style::empty_state_panel_style;
    let body = column![
        text(t!("empty_state.file_tree_no_result_title").to_string())
            .size(13)
            .color(Color::from_rgb(0.40, 0.42, 0.48)),
        space().height(Length::Fixed(6.0)),
        text(t!("empty_state.file_tree_no_result_hint").to_string())
            .size(11)
            .color(Color::from_rgb(0.55, 0.55, 0.60)),
    ]
    .spacing(0)
    .align_x(iced::Alignment::Center)
    .width(Length::Fill);
    container(body)
        .padding(Padding::from([24.0, 16.0]))
        .width(Length::Fill)
        .center_x(Length::Fill)
        .style(empty_state_panel_style)
        .into()
}

fn empty_state_diff_panel<'a>() -> Element<'a, Message> {
    use crate::style::empty_state_panel_style;
    let body = column![
        text(t!("empty_state.diff_no_audit_title").to_string())
            .size(14)
            .color(Color::from_rgb(0.40, 0.42, 0.48)),
        space().height(Length::Fixed(10.0)),
        // Two-step guidance. Stepping is implicit in the order; we keep
        // the symbols inline with each line so ABDD's "no colour
        // dependence" rule is met (the bullet character itself carries
        // the meaning, not styling).
        text(format!("①  {}", t!("empty_state.diff_no_audit_step1")))
            .size(12)
            .color(Color::from_rgb(0.50, 0.52, 0.58)),
        space().height(Length::Fixed(4.0)),
        text(format!("②  {}", t!("empty_state.diff_no_audit_step2")))
            .size(12)
            .color(Color::from_rgb(0.50, 0.52, 0.58)),
    ]
    .spacing(0)
    .align_x(iced::Alignment::Center)
    .width(Length::Fill);
    container(body)
        .padding(Padding::from([32.0, 24.0]))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(empty_state_panel_style)
        .into()
}

fn empty_state_inspector<'a>() -> Element<'a, Message> {
    use crate::style::empty_state_panel_style;
    let body = column![
        text(t!("empty_state.inspector_no_selection").to_string())
            .size(13)
            .color(Color::from_rgb(0.40, 0.42, 0.48)),
        space().height(Length::Fixed(6.0)),
        text(format!("←  {}", t!("empty_state.inspector_no_selection_hint")))
            .size(11)
            .color(Color::from_rgb(0.55, 0.55, 0.60)),
    ]
    .spacing(0)
    .align_x(iced::Alignment::Center)
    .width(Length::Fill);
    container(body)
        .padding(Padding::from([24.0, 16.0]))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(empty_state_panel_style)
        .into()
}
