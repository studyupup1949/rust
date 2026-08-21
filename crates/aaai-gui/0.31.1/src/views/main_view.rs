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
    // RFC 071 — search bar is now inside build_file_tree, not a top-level row.
    let bottom_bar = build_bottom_bar(app);
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
        pg,
        bottom_bar,   // RFC 008: fixed bottom action bar
    ]
    .spacing(0)
    .into()
}

// ── Toolbar ──────────────────────────────────────────────────────────────────

fn build_toolbar<'a>(app: &'a App) -> Element<'a, Message> {
    // RFC 007 + RFC 014 + RFC 070 (layout stability, Undo relocation)
    //
    // Layout:  [← Open]  [↓ Save]  [▶ Run]  [↑ Export]  [↩ Undo]  ─────  [● STATUS]
    //
    // RFC 070 changes from the previous layout:
    //  • "✓ saved Nm ago" marks moved BELOW their buttons (not inline) so the
    //    button row never shifts width when the marks appear/disappear.
    //  • Undo moved here from the filter bar (where it was semantically wrong).
    //  • Icon glyphs clarified: Save = ↓, Open = ←, Export = ↑, Undo = ↩
    use crate::style::panel_style;

    let toolbar_btn = |icon: &'a str, label: String, msg: Message| -> Element<'a, Message> {
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

    let open_btn   = toolbar_btn("←", t!("toolbar.open").to_string(),          Message::BackToOpening);
    let save_btn   = toolbar_btn("↓", t!("toolbar.save").to_string(),          Message::SaveDefinition);
    let run_btn    = toolbar_btn("▶", t!("toolbar.run_audit").to_string(),     Message::RerunAudit);
    let report_btn = toolbar_btn("↑", t!("toolbar.report_output").to_string(), Message::ExportReport);
    let undo_btn   = toolbar_btn("↩", t!("toolbar.undo").to_string(),          Message::UndoApproval);

    let save_mark_text = app.last_saved_at.map(|t| format!("✓ {}",
        crate::util::humanize_since(t)));
    let report_mark_text = app.last_reported_at.map(|t| format!("✓ {}",
        crate::util::humanize_since(t)));

    // RFC 070 — "✓ saved Nm ago" marks stack BELOW their button in a fixed-height
    // sub-column so the row width is stable regardless of mark presence.
    let save_mark: Element<'_, Message> = match save_mark_text {
        Some(m) => text(m).size(9).color(Color::from_rgb(0.35, 0.55, 0.35)).into(),
        None    => space().height(Length::Fixed(13.0)).into(),
    };
    let save_col = column![save_btn, save_mark]
        .spacing(1)
        .align_x(iced::Alignment::Center);

    let report_mark: Element<'_, Message> = match report_mark_text {
        Some(m) => text(m).size(9).color(Color::from_rgb(0.35, 0.55, 0.35)).into(),
        None    => space().height(Length::Fixed(13.0)).into(),
    };
    let report_col = column![report_btn, report_mark]
        .spacing(1)
        .align_x(iced::Alignment::Center);

    // Audit status — compact colored pill: "● PASSED" / "● FAILED"
    let status_element: Element<'_, Message> = if app.audit_dirty && app.is_loading {
        text(format!("○ {}", t!("toolbar.rerunning")))
            .size(12).color(theme::PENDING_COLOR).into()
    } else if let Some(result) = &app.audit_result {
        let s = &result.summary;
        let (label, color) = if s.is_passing() {
            (t!("toolbar.passed").to_string(), theme::OK_COLOR)
        } else {
            (t!("toolbar.failed").to_string(), theme::FAILED_COLOR)
        };
        text(format!("● {}", label))
            .size(12).color(color).into()
    } else {
        space().width(Length::Fixed(1.0)).into()
    };

    container(
        row![
            open_btn, save_col, run_btn, report_col, undo_btn,
            space().width(Length::Fill),
            status_element,
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center)
        .padding(Padding::from([3.0, 10.0])),
    )
    .width(Length::Fill)
    .style(panel_style)
    .into()
}

// ── Filter bar ───────────────────────────────────────────────────────────────

fn build_filter_bar<'a>(app: &'a App) -> Element<'a, Message> {
    use crate::style::panel_style;

    // RFC 043 — pre-compute per-filter counts when audit is available.
    let counts = app.audit_result.as_ref().map(|r| {
        let s = &r.summary;
        // "Changed only" passes non-unchanged diffs regardless of status;
        // count = total - items with DiffType::Unchanged that also passed.
        // Simplest approximation: total - OK-and-unchanged entries.
        // We count the result list directly for accuracy.
        let changed_n = r.results.iter()
            .filter(|far| FilterMode::ChangedOnly.passes(far))
            .count();
        (s.total, changed_n, s.pending, s.failed + s.error)
    });

    // Build a button label: "Label (N)" when counts are available,
    // "Label" otherwise.
    let make_btn = |base_key: &'static str, mode: FilterMode, count: Option<usize>| {
        let label = match count {
            Some(n) => format!("{} ({})", t!(base_key), n),
            None    => t!(base_key).to_string(),
        };
        let active = app.filter_mode == mode;
        let btn = button(text(label).size(11))
            .on_press(Message::SetFilter(mode))
            .padding(Padding::from([10.0, 14.0]));
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

    let (all_n, changed_n, pending_n, errors_n) = match counts {
        Some((a, c, p, e)) => (Some(a), Some(c), Some(p), Some(e)),
        None               => (None, None, None, None),
    };

    // RFC 076 — status legend ? button at the right of the filter bar
    let legend_btn = button(text("?").size(11))
        .on_press(Message::ToggleStatusLegend)
        .padding(Padding::from([8.0, 12.0]))
        .style(iced::widget::button::text);

    let filter_row = row![
        make_btn("filter.all",     FilterMode::All,           all_n),
        make_btn("filter.changed", FilterMode::ChangedOnly,   changed_n),
        make_btn("filter.pending", FilterMode::PendingOnly,   pending_n),
        make_btn("filter.errors",  FilterMode::FailedAndError, errors_n),
        space().width(Length::Fill),
        legend_btn,
    ]
    .spacing(4)
    .align_y(iced::Alignment::Center)
    .padding(Padding::from([3.0, 8.0]));

    // RFC 076 — status legend popover (shown inline below the filter bar
    // so it stays close to the ? button and the status badges it explains)
    let legend_popup: Element<'_, Message> = if app.status_legend_open {
        let line = |key: &str| -> Element<'_, Message> {
            text(t!(key).to_string())
                .size(11)
                .color(Color::from_rgb(0.30, 0.32, 0.38))
                .into()
        };
        container(
            column![
                text(t!("main.status_legend_title").to_string())
                    .size(12)
                    .color(Color::from_rgb(0.20, 0.22, 0.28))
                    .font(iced::Font { weight: iced::font::Weight::Semibold, ..Default::default() }),
                space().height(Length::Fixed(6.0)),
                line("main.status_legend_pending"),
                line("main.status_legend_ok"),
                line("main.status_legend_failed"),
                line("main.status_legend_error"),
            ]
            .spacing(3)
        )
        .padding(Padding::from([10.0, 14.0]))
        .width(Length::Fill)
        .style(|_| iced::widget::container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.96, 0.97, 0.99))),
            border: iced::Border {
                color: Color::from_rgb(0.80, 0.82, 0.88),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .into()
    } else {
        space().height(0).into()
    };

    container(
        column![filter_row, legend_popup].spacing(0)
    )
    .width(Length::Fill)
    .style(panel_style)
    .into()
}

// ── Search bar ───────────────────────────────────────────────────────────────
// RFC 071 — search bar now lives at the top of the file tree pane, not
// as a standalone row above the entire pane grid. This function is kept
// as a building block called from build_file_tree.
fn build_search_bar<'a>(app: &'a App) -> Element<'a, Message> {
    if app.audit_result.is_none() {
        return space().height(0).into();
    }
    let search_placeholder = t!("main.search_placeholder").to_string();
    container(
        row![
            text("🔍").size(12),
            text_input(&search_placeholder, &app.search_query)
                .on_input(Message::SearchQueryChanged)
                .padding(Padding::from([3.0, 6.0]))
                .size(12)
                .width(Length::Fill),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
    )
    .padding(Padding::from([6.0, 10.0]))
    .width(Length::Fill)
    .style(|_| iced::widget::container::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.95, 0.96, 0.97))),
        ..Default::default()
    })
    .into()
}

// Kept for backward compatibility with existing call sites; unused variable suppressed.
#[allow(dead_code)]
fn _build_search_bar_unused<'a>(_app: &'a App) -> Element<'a, Message> {
    space().height(0).into()
}

// ── File tree pane ───────────────────────────────────────────────────────────

fn build_file_tree<'a>(app: &'a App) -> Element<'a, Message> {
    let result = match &app.audit_result {
        Some(r) => r,
        None    => return empty_state_file_tree(),
    };

    // RFC 071 — search bar lives at the top of this pane, not above the grid.
    let search = build_search_bar(app);

    // RFC 077 — first-audit coach line: shown once per session above the
    // file tree after the first audit completes. Gives newcomers a one-line
    // explanation of what they're looking at without interrupting experts
    // (they dismiss it once and it stays gone for the session).
    let coach_line: Option<Element<'_, Message>> =
        if !app.coach_dismissed {
            let dismiss_btn = button(text(t!("main.coach_dismiss").to_string()).size(10))
                .on_press(Message::DismissCoach)
                .padding(Padding::from([3.0, 8.0]))
                .style(iced::widget::button::text);
            Some(
                container(
                    row![
                        text(t!("main.coach_line").to_string())
                            .size(11)
                            .color(Color::from_rgb(0.28, 0.38, 0.56)),
                        space().width(Length::Fill),
                        dismiss_btn,
                    ]
                    .align_y(iced::Alignment::Center)
                    .spacing(4),
                )
                .padding(Padding::from([6.0, 10.0]))
                .width(Length::Fill)
                .style(|_| iced::widget::container::Style {
                    background: Some(iced::Background::Color(
                        Color::from_rgb(0.93, 0.96, 1.00)
                    )),
                    ..Default::default()
                })
                .into()
            )
        } else {
            None
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
        let mut col = column![search];
        if let Some(coach) = coach_line { col = col.push(coach); }
        return col.push(
            container(
                text(t!("empty_state.no_entries_match_filter").to_string()).size(12)
                    .color(Color::from_rgb(0.55, 0.55, 0.58))
            ).padding(12)
        ).spacing(0).width(Length::Fill).height(Length::Fill).into();
    }

    let tree_scroll = scrollable(
        column(items).spacing(0).width(Length::Fill)
    )
    .width(Length::Fill)
    .height(Length::Fill);

    let mut col = column![search];
    if let Some(coach) = coach_line { col = col.push(coach); }
    col.push(tree_scroll)
        .spacing(0)
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

    // RFC 073 — hide entirely when no file is selected: the bar implies
    // there is something actionable, which is misleading when the user is
    // looking at the dashboard or has just opened the screen.
    if app.selected_index.is_none() {
        return space().height(0).into();
    }

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
        // RFC 043 — i18n'd; was hardcoded Japanese.
        text(t!("filter.count_summary",
                total = s.total.to_string(),
                unresolved = unresolved.to_string()).to_string())
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
