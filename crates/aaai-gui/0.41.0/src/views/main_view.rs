//! Main 3-pane view — Phase 10: PaneGrid (resizable) + directory collapse.

use iced::{
    Color, Element, Length, Padding,
    widget::{
        button, column, container, pane_grid,
        row, scrollable, space, text, text_input,
    },
};
use rust_i18n::t;

use aaai::{AuditStatus, DiffType, FileAuditResult};
use crate::style::panel_style;
use crate::app::{App, FilterMode, Message, PaneKind};
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
    .spacing(app.design_tokens.spacing.xs)
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
    
    let toolbar_btn = |icon: &'a str, label: String, msg: Message| -> Element<'a, Message> {
        button(
            row![
                text(icon)
                    .size(app.design_tokens.typography.label.size)
                    .line_height(app.design_tokens.typography.label.line_height),
                text(label)
                    .size(app.design_tokens.typography.label.size)
                    .line_height(app.design_tokens.typography.label.line_height),
            ]
            .spacing(app.design_tokens.spacing.xs)
            .align_y(iced::Alignment::Center)
        )
        .on_press(msg)
        .padding(Padding::from([app.design_tokens.spacing.md, app.design_tokens.spacing.lg]))
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
        Some(m) => text(m)
            .size(app.design_tokens.typography.body_small.size)
            .line_height(app.design_tokens.typography.body_small.line_height)
            .color(crate::style::to_iced(app.design_tokens.palette.success)).into(),
        None    => space().height(Length::Fixed(13.0)).into(),
    };
    let save_col = column![save_btn, save_mark]
        .spacing(app.design_tokens.spacing.xs)
        .align_x(iced::Alignment::Center);

    let report_mark: Element<'_, Message> = match report_mark_text {
        Some(m) => text(m)
            .size(app.design_tokens.typography.body_small.size)
            .line_height(app.design_tokens.typography.body_small.line_height)
            .color(crate::style::to_iced(app.design_tokens.palette.success)).into(),
        None    => space().height(Length::Fixed(13.0)).into(),
    };
    let report_col = column![report_btn, report_mark]
        .spacing(app.design_tokens.spacing.xs)
        .align_x(iced::Alignment::Center);

    // Audit status — compact colored pill: "● PASSED" / "● FAILED"
    let status_element: Element<'_, Message> = if app.audit_dirty && app.is_loading {
        text(format!("○ {}", t!("toolbar.rerunning")))
            .size(app.design_tokens.typography.label.size)
            .line_height(app.design_tokens.typography.label.line_height)
            .color(crate::theme::status_color(aaai::AuditStatus::Pending, &app.design_tokens, app.theme.is_high_contrast())).into()
    } else if let Some(result) = &app.audit_result {
        let s = &result.summary;
        let (label, color) = if s.is_passing() {
            (t!("toolbar.passed").to_string(), crate::theme::status_color(aaai::AuditStatus::Ok, &app.design_tokens, app.theme.is_high_contrast()))
        } else {
            (t!("toolbar.failed").to_string(), crate::theme::status_color(aaai::AuditStatus::Failed, &app.design_tokens, app.theme.is_high_contrast()))
        };
        text(format!("● {}", label))
            .size(app.design_tokens.typography.label.size)
            .line_height(app.design_tokens.typography.label.line_height)
            .color(color).into()
    } else {
        space().width(Length::Fixed(1.0)).into()
    };

    container(
        row![
            open_btn, save_col, run_btn, report_col, undo_btn,
            space().width(Length::Fill),
            status_element,
        ]
        .spacing(app.design_tokens.spacing.xs)
        .align_y(iced::Alignment::Center)
        .padding(Padding::from([app.design_tokens.spacing.xs, app.design_tokens.spacing.md])),
    )
    .width(Length::Fill)
    .style(panel_style(app.design_tokens.clone()))
    .into()
}

// ── Filter bar ───────────────────────────────────────────────────────────────

fn build_filter_bar<'a>(app: &'a App) -> Element<'a, Message> {
    
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
        let btn = button(
            text(label)
                .size(app.design_tokens.typography.label.size)
                .line_height(app.design_tokens.typography.label.line_height),
        )
            .on_press(Message::SetFilter(mode))
            .padding(Padding::from([app.design_tokens.spacing.md, app.design_tokens.spacing.lg]));
        if active {
            let accent = crate::style::to_iced(app.design_tokens.palette.accent);
            container(btn)
                .style(move |_| iced::widget::container::Style {
                    background: Some(iced::Background::Color(
                        Color { a: 0.18, ..accent }
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
    let legend_btn = button(
        text("?")
            .size(app.design_tokens.typography.label.size)
            .line_height(app.design_tokens.typography.label.line_height),
    )
        .on_press(Message::ToggleStatusLegend)
        .padding(Padding::from([app.design_tokens.spacing.sm, app.design_tokens.spacing.md]))
        .style({ let t = app.design_tokens.clone(); move |_th, s| crate::style::btn_ghost(&t, s) });

    let filter_row = row![
        make_btn("filter.all",     FilterMode::All,           all_n),
        make_btn("filter.changed", FilterMode::ChangedOnly,   changed_n),
        make_btn("filter.pending", FilterMode::PendingOnly,   pending_n),
        make_btn("filter.errors",  FilterMode::FailedAndError, errors_n),
        space().width(Length::Fill),
        legend_btn,
    ]
    .spacing(app.design_tokens.spacing.xs)
    .align_y(iced::Alignment::Center)
    .padding(Padding::from([app.design_tokens.spacing.xs, app.design_tokens.spacing.sm]));

    // RFC 076 — status legend popover (shown inline below the filter bar
    // so it stays close to the ? button and the status badges it explains)
    let legend_popup: Element<'_, Message> = if app.status_legend_open {
        let line = |key: &str| -> Element<'_, Message> {
            text(t!(key).to_string())
                .size(app.design_tokens.typography.body_small.size)
                .line_height(app.design_tokens.typography.body_small.line_height)
                .color(crate::style::to_iced(app.design_tokens.palette.text_secondary))
                .into()
        };
        container(
            column![
                text(t!("main.status_legend_title").to_string())
                    .size(app.design_tokens.typography.title.size)
                    .line_height(app.design_tokens.typography.title.line_height)
                    .color(crate::style::to_iced(app.design_tokens.palette.text_secondary))
                    .font(iced::Font { weight: iced::font::Weight::Semibold, ..Default::default() }),
                space().height(Length::Fixed(6.0)),
                line("main.status_legend_pending"),
                line("main.status_legend_ok"),
                line("main.status_legend_failed"),
                line("main.status_legend_error"),
            ]
            .spacing(app.design_tokens.spacing.xs)
        )
        .padding(Padding::from([app.design_tokens.spacing.md, app.design_tokens.spacing.lg]))
        .width(Length::Fill)
        .style({
            let surface = crate::style::to_iced(app.design_tokens.palette.surface);
            let border = crate::style::to_iced(app.design_tokens.palette.border);
            move |_| iced::widget::container::Style {
                background: Some(iced::Background::Color(surface)),
                border: iced::Border {
                    color: border,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
    } else {
        space().height(0).into()
    };

    container(
        column![filter_row, legend_popup].spacing(0)
    )
    .width(Length::Fill)
    .style(panel_style(app.design_tokens.clone()))
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
            text("🔍")
                .size(app.design_tokens.typography.label.size)
                .line_height(app.design_tokens.typography.label.line_height),
            text_input(&search_placeholder, &app.search_query)
                .on_input(Message::SearchQueryChanged)
                .padding(Padding::from([app.design_tokens.spacing.xs, app.design_tokens.spacing.sm]))
                .size(app.design_tokens.typography.body_small.size)
                .line_height(app.design_tokens.typography.body_small.line_height)
                .width(Length::Fill),
        ]
        .spacing(app.design_tokens.spacing.sm)
        .align_y(iced::Alignment::Center),
    )
    .padding(Padding::from([app.design_tokens.spacing.sm, app.design_tokens.spacing.md]))
    .width(Length::Fill)
    .style({
        let surface = crate::style::to_iced(app.design_tokens.palette.surface);
        move |_| iced::widget::container::Style {
            background: Some(iced::Background::Color(surface)),
            ..Default::default()
        }
    })
    .into()
}


// ── File tree pane ───────────────────────────────────────────────────────────

fn build_file_tree<'a>(app: &'a App) -> Element<'a, Message> {
    let result = match &app.audit_result {
        Some(r) => r,
        None    => return empty_state_file_tree(app.design_tokens.clone()),
    };

    // RFC 071 — search bar lives at the top of this pane, not above the grid.
    let search = build_search_bar(app);

    // RFC 077 — first-audit coach line: shown once per session above the
    // file tree after the first audit completes. Gives newcomers a one-line
    // explanation of what they're looking at without interrupting experts
    // (they dismiss it once and it stays gone for the session).
    let coach_line: Option<Element<'_, Message>> =
        if !app.coach_dismissed {
            let dismiss_btn = button(
                text(t!("main.coach_dismiss").to_string())
                    .size(app.design_tokens.typography.label.size)
                    .line_height(app.design_tokens.typography.label.line_height),
            )
                .on_press(Message::DismissCoach)
                .padding(Padding::from([app.design_tokens.spacing.xs, app.design_tokens.spacing.sm]))
                .style({ let t = app.design_tokens.clone(); move |_th, s| crate::style::btn_ghost(&t, s) });
            Some(
                container(
                    row![
                        text(t!("main.coach_line").to_string())
                            .size(app.design_tokens.typography.body_small.size)
                            .line_height(app.design_tokens.typography.body_small.line_height)
                            .color(crate::style::to_iced(app.design_tokens.palette.accent)),
                        space().width(Length::Fill),
                        dismiss_btn,
                    ]
                    .align_y(iced::Alignment::Center)
                    .spacing(app.design_tokens.spacing.xs),
                )
                .padding(Padding::from([app.design_tokens.spacing.sm, app.design_tokens.spacing.md]))
                .width(Length::Fill)
                .style({
                    let accent = crate::style::to_iced(app.design_tokens.palette.accent);
                    move |_| iced::widget::container::Style {
                        background: Some(iced::Background::Color(
                            Color { a: 0.10, ..accent }
                        )),
                        ..Default::default()
                    }
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
                    text(format!("{icon} {}", parts[parts.len()-2]))
                        .size(app.design_tokens.typography.label.size)
                        .line_height(app.design_tokens.typography.label.line_height)
                        .color(crate::style::to_iced(app.design_tokens.palette.text_secondary))
                        .font(iced::Font { weight: iced::font::Weight::Semibold, ..Default::default() }),
                ]
                .spacing(app.design_tokens.spacing.xs)
                .align_y(iced::Alignment::Center),
            )
            .on_press(Message::ToggleDir(dir_clone))
            .width(Length::Fill)
            .padding(Padding::from([app.design_tokens.spacing.xs, app.design_tokens.spacing.sm]))  // ABDD minimum
            .style({ let t = app.design_tokens.clone(); move |_th, s| crate::style::btn_ghost(&t, s) });
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
                text(t!("empty_state.no_entries_match_filter").to_string())
                    .size(app.design_tokens.typography.body_small.size)
                    .line_height(app.design_tokens.typography.body_small.line_height)
                    .color(crate::style::to_iced(app.design_tokens.palette.text_muted))
            ).padding(app.design_tokens.spacing.md)
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
                text(format!("⚠{}", far.warnings.len()))
                    .size(app.design_tokens.typography.label.size)
                    .line_height(app.design_tokens.typography.label.line_height)
                    .color(crate::style::to_iced(app.design_tokens.palette.warning))
            )
            .padding(Padding::from([app.design_tokens.spacing.xs, app.design_tokens.spacing.xs]))
            .style({
                let warning = crate::style::to_iced(app.design_tokens.palette.warning);
                move |_| iced::widget::container::Style {
                    background: Some(iced::Background::Color(
                        Color { a: 0.25, ..warning }
                    )),
                    border: iced::Border {
                        color: Color { a: 0.50, ..warning },
                        width: 1.0,
                        radius: 3.0.into(),
                    },
                    ..Default::default()
                }
            })
            .into()
        )
    } else { None };

    let sicon = status_icon(far.status, &app.design_tokens, app.theme.is_high_contrast());
    let dtype_tag = diff_type_tag(far.diff.diff_type, &app.design_tokens);
    let mut name_row = row![
        space().width(Length::Fixed(indent)),
        sicon,
        text(short)
            .size(app.design_tokens.typography.body_small.size)
            .line_height(app.design_tokens.typography.body_small.line_height)
            .font(iced::Font::MONOSPACE),
    ]
    .spacing(app.design_tokens.spacing.sm)
    .align_y(iced::Alignment::Center);
    if let Some(wb) = warn_badge {
        name_row = name_row.push(wb);
    }

    let full_row = row![
        name_row,
        space().width(Length::Fill),
        dtype_tag,
    ]
    .spacing(app.design_tokens.spacing.xs)
    .align_y(iced::Alignment::Center);

    let selected_bg = crate::style::to_iced(app.design_tokens.palette.accent);
    let bg = move |_: &iced::Theme| iced::widget::container::Style {
        background: if is_selected {
            Some(iced::Background::Color(Color { a: 0.18, ..selected_bg }))
        } else { None },
        ..Default::default()
    };

    button(
        container(full_row)
            .width(Length::Fill)
            .padding(Padding::from([app.design_tokens.spacing.xs, app.design_tokens.spacing.sm]))
            .style(bg),
    )
    .on_press(Message::SelectEntry(idx))
    .width(Length::Fill)
    .padding(0)
    .style({ let t = app.design_tokens.clone(); move |_th, s| crate::style::btn_ghost(&t, s) })
    .into()
}

// ── Diff pane ────────────────────────────────────────────────────────────────

fn build_diff_panel<'a>(app: &'a App) -> Element<'a, Message> {
    match app.selected_index {
        Some(idx) => {
            if let Some(result) = &app.audit_result {
                if let Some(far) = result.results.get(idx) {
                    return diff_view::view(&far.diff, app.diff_view_mode, &app.design_tokens, app.theme.is_high_contrast());
                }
            }
        }
        None => {}
    }
    match &app.audit_result {
        Some(r) => dashboard::view(r, &app.design_tokens, app.theme.is_high_contrast()),
        None    => empty_state_diff_panel(app.design_tokens.clone()),
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
    empty_state_inspector(app.design_tokens.clone())
}

// ── Helpers ──────────────────────────────────────────────────────────────────


// RFC 013: single status icon — symbol + colour only, no text label.
fn status_icon(status: AuditStatus, tokens: &snora::design::Tokens, is_hc: bool) -> Element<'_, Message> {
    let (sym, color) = match status {
        AuditStatus::Ok      => ("✓", crate::theme::status_color(AuditStatus::Ok,      tokens, is_hc)),
        AuditStatus::Pending => ("⚠", crate::theme::status_color(AuditStatus::Pending, tokens, is_hc)),
        AuditStatus::Failed  => ("✗", crate::theme::status_color(AuditStatus::Failed,  tokens, is_hc)),
        AuditStatus::Error   => ("!", crate::theme::status_color(AuditStatus::Error,   tokens, is_hc)),
        AuditStatus::Ignored => ("—", crate::theme::status_color(AuditStatus::Ignored, tokens, is_hc)),
    };
    text(sym)
        .size(tokens.typography.label.size)
        .line_height(tokens.typography.label.line_height)
        .color(color).into()
}

// RFC 013: diff-type tag — right-aligned subtle grey symbol.
fn diff_type_tag(dtype: DiffType, tokens: &snora::design::Tokens) -> Element<'static, Message> {
    let sym = match dtype {
        DiffType::Added        => "+",
        DiffType::Removed      => "−",
        DiffType::Modified     => "~",
        DiffType::TypeChanged  => "T",
        DiffType::Unchanged    => " ",
        DiffType::Unreadable   => "!",
        DiffType::Incomparable => "?",
    };
    text(sym)
        .size(tokens.typography.label.size)
        .line_height(tokens.typography.label.line_height)
        .color(crate::style::to_iced(tokens.palette.text_muted))
        .into()
}



// ── Bottom action bar (RFC 008) ───────────────────────────────────────────────

fn build_bottom_bar<'a>(app: &'a App) -> Element<'a, Message> {
    
    // RFC 073 — hide entirely when no file is selected: the bar implies
    // there is something actionable, which is misleading when the user is
    // looking at the dashboard or has just opened the screen.
    if app.selected_index.is_none() {
        return space().height(0).into();
    }

    // "Save and continue" button — enabled only when an entry is selected and valid
    let can_approve = app.selected_index.is_some()
        && app.inspector.validation.can_approve();

    // RFC 087 — disabled state explanation.
    // The disabled reason drives a tooltip on the button so the user
    // knows exactly what to do before the action becomes available.
    let disabled_reason: Option<String> = if !can_approve {
        if app.selected_index.is_none() {
            Some(t!("bottombar.disabled_no_file").to_string())
        } else {
            Some(t!("bottombar.disabled_no_reason").to_string())
        }
    } else {
        None
    };

    let t_approve = app.design_tokens.clone();
    let approve_btn_inner = button(
        text(t!("bottombar.approve_and_save").to_string())
            .size(app.design_tokens.typography.label.size)
            .line_height(app.design_tokens.typography.label.line_height)
            .font(iced::Font {
                weight: iced::font::Weight::Semibold,
                ..Default::default()
            }),
    )
    .on_press_maybe(if can_approve { Some(Message::ApproveAndSave) } else { None })
    .padding(Padding::from([app.design_tokens.spacing.md, app.design_tokens.spacing.xl]))  // ABDD ≥44px
    .style(move |_theme, s| crate::style::btn_primary(&t_approve, s));

    let approve_btn: Element<'_, Message> = match disabled_reason {
        Some(reason) => {
            let hint = t!("bottombar.disabled_fix_hint").to_string();
            iced::widget::tooltip(
                approve_btn_inner,
                iced::widget::container(
                    iced::widget::column![
                        iced::widget::text(reason)
                            .size(app.design_tokens.typography.body_small.size)
                            .line_height(app.design_tokens.typography.body_small.line_height),
                        iced::widget::text(hint)
                            .size(app.design_tokens.typography.body_small.size)
                            .line_height(app.design_tokens.typography.body_small.line_height)
                            .color(crate::style::to_iced(app.design_tokens.palette.text_muted)),
                    ]
                    .spacing(app.design_tokens.spacing.xs)
                )
                .padding(iced::Padding::from([app.design_tokens.spacing.sm, app.design_tokens.spacing.md])),
                iced::widget::tooltip::Position::Top,
            )
            .into()
        }
        None => approve_btn_inner.into(),
    };

    // Selected file label
    let selected_label: Element<'_, Message> = if let Some(idx) = app.selected_index {
        if let Some(r) = app.audit_result.as_ref().and_then(|r| r.results.get(idx)) {
            text(format!("{}  {}", t!("bottombar.selected"), r.diff.path))
                .size(app.design_tokens.typography.body_small.size)
                .line_height(app.design_tokens.typography.body_small.line_height)
                .color(crate::style::to_iced(app.design_tokens.palette.text_secondary))
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
            crate::style::to_iced(app.design_tokens.palette.danger)
        } else {
            crate::style::to_iced(app.design_tokens.palette.success)
        };
        // RFC 043 — i18n'd; was hardcoded Japanese.
        text(t!("filter.count_summary",
                total = s.total.to_string(),
                unresolved = unresolved.to_string()).to_string())
            .size(app.design_tokens.typography.body_small.size)
            .line_height(app.design_tokens.typography.body_small.line_height)
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
        .spacing(app.design_tokens.spacing.sm)
        .align_y(iced::Alignment::Center)
        .padding(Padding::from([app.design_tokens.spacing.sm, app.design_tokens.spacing.md])),
    )
    .width(Length::Fill)
    .style(panel_style(app.design_tokens.clone()))
    .into()
}

// ── RFC 022: empty-state panels ──────────────────────────────────────────────
//
// Used when `audit_result` is None (file_tree / diff_panel) or when nothing
// is selected (inspector). All three follow the same visual contract:
// transparent background, soft 1-px border, mid-grey text. The guidance
// text is i18n-driven via the `empty_state.*` namespace so en/ja produce
// equivalent prose.

fn empty_state_file_tree<'a>(tokens: snora::design::Tokens) -> Element<'a, Message> {
    use crate::style::empty_state_panel_style;
    let body = column![
        text(t!("empty_state.file_tree_no_result_title").to_string())
            .size(tokens.typography.title.size)
            .line_height(tokens.typography.title.line_height)
            .color(crate::style::to_iced(tokens.palette.text_secondary)),
        space().height(Length::Fixed(6.0)),
        text(t!("empty_state.file_tree_no_result_hint").to_string())
            .size(tokens.typography.body_small.size)
            .line_height(tokens.typography.body_small.line_height)
            .color(crate::style::to_iced(tokens.palette.text_muted)),
    ]
    .spacing(0)
    .align_x(iced::Alignment::Center)
    .width(Length::Fill);
    container(body)
        .padding(Padding::from([tokens.spacing.xl, tokens.spacing.lg]))
        .width(Length::Fill)
        .center_x(Length::Fill)
        .style(empty_state_panel_style(tokens))
        .into()
}

fn empty_state_diff_panel<'a>(tokens: snora::design::Tokens) -> Element<'a, Message> {
    use crate::style::empty_state_panel_style;
    let body = column![
        text(t!("empty_state.diff_no_audit_title").to_string())
            .size(tokens.typography.title.size)
            .line_height(tokens.typography.title.line_height)
            .color(crate::style::to_iced(tokens.palette.text_secondary)),
        space().height(Length::Fixed(10.0)),
        // Two-step guidance. Stepping is implicit in the order; we keep
        // the symbols inline with each line so ABDD's "no colour
        // dependence" rule is met (the bullet character itself carries
        // the meaning, not styling).
        text(format!("①  {}", t!("empty_state.diff_no_audit_step1")))
            .size(tokens.typography.body_small.size)
            .line_height(tokens.typography.body_small.line_height)
            .color(crate::style::to_iced(tokens.palette.text_muted)),
        space().height(Length::Fixed(4.0)),
        text(format!("②  {}", t!("empty_state.diff_no_audit_step2")))
            .size(tokens.typography.body_small.size)
            .line_height(tokens.typography.body_small.line_height)
            .color(crate::style::to_iced(tokens.palette.text_muted)),
    ]
    .spacing(0)
    .align_x(iced::Alignment::Center)
    .width(Length::Fill);
    container(body)
        .padding(Padding::from([tokens.spacing.xxl, tokens.spacing.xl]))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(empty_state_panel_style(tokens))
        .into()
}

fn empty_state_inspector<'a>(tokens: snora::design::Tokens) -> Element<'a, Message> {
    use crate::style::empty_state_panel_style;
    let body = column![
        text(t!("empty_state.inspector_no_selection").to_string())
            .size(tokens.typography.title.size)
            .line_height(tokens.typography.title.line_height)
            .color(crate::style::to_iced(tokens.palette.text_secondary)),
        space().height(Length::Fixed(6.0)),
        text(format!("←  {}", t!("empty_state.inspector_no_selection_hint")))
            .size(tokens.typography.body_small.size)
            .line_height(tokens.typography.body_small.line_height)
            .color(crate::style::to_iced(tokens.palette.text_muted)),
    ]
    .spacing(0)
    .align_x(iced::Alignment::Center)
    .width(Length::Fill);
    container(body)
        .padding(Padding::from([tokens.spacing.xl, tokens.spacing.lg]))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(empty_state_panel_style(tokens))
        .into()
}
