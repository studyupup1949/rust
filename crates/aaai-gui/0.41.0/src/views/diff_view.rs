//! Side-by-side diff viewer (Phase 4: binary file panel, stats bar, masking).

use iced::{
    Color, Element, Length, Padding,
    widget::{column, container, row, scrollable, text},
};
use rust_i18n::t;
use similar::{ChangeTag, TextDiff};

use aaai::{DiffType, diff::entry::DiffEntry};
use crate::app::Message;

/// RFC 069 — stable IDs for the side-by-side diff panes, used for scroll sync.
pub static DIFF_BEFORE_ID: std::sync::LazyLock<iced::widget::Id> =
    std::sync::LazyLock::new(|| iced::widget::Id::new("diff_before"));
pub static DIFF_AFTER_ID: std::sync::LazyLock<iced::widget::Id> =
    std::sync::LazyLock::new(|| iced::widget::Id::new("diff_after"));

pub fn view<'a>(diff: &'a DiffEntry, mode: crate::app::DiffViewMode, tokens: &'a snora::design::Tokens, _is_hc: bool) -> Element<'a, Message> {
    use crate::app::DiffViewMode;

    if diff.is_dir {
        return placeholder(t!("diff.directory").to_string(), tokens);
    }
    match diff.diff_type {
        DiffType::Unchanged => placeholder(t!("diff.identical").to_string(), tokens),
        DiffType::Unreadable | DiffType::Incomparable => {
            let msg = diff.error_detail.clone()
                .unwrap_or_else(|| t!("diff.unreadable").to_string());
            placeholder(msg, tokens)
        }
        _ if diff.is_binary => binary_panel(diff, tokens),
        _ => {
            // RFC 011: tab bar + selected view
            let has_text = diff.has_text_diff();
            let tab_bar = build_tab_bar(mode, has_text, tokens);
            let content = match mode {
                DiffViewMode::SideBySide  => side_by_side(diff, tokens),
                DiffViewMode::Unified     => unified_view(diff, tokens),
                DiffViewMode::ChangedOnly => changed_only_view(diff, tokens),
            };
            column![tab_bar, content]
                .spacing(0)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
    }
}

/// RFC 011: Tab selector for diff view modes.
fn build_tab_bar(mode: crate::app::DiffViewMode, _has_text: bool, tokens: &snora::design::Tokens) -> Element<'static, Message> {
    use crate::app::DiffViewMode;

    let tab = |label: String, target: DiffViewMode, active: bool| -> Element<'static, Message> {
        let fg = if active {
            crate::style::to_iced(tokens.palette.accent)
        } else {
            crate::style::to_iced(tokens.palette.text_secondary)
        };
        let border_bottom = if active {
            iced::Border {
                color: crate::style::to_iced(tokens.palette.accent),
                width: 2.0,
                radius: 0.0.into(),
            }
        } else {
            iced::Border::default()
        };
        iced::widget::button(
            iced::widget::container(
                text(label)
                    .size(tokens.typography.label.size)
                    .line_height(tokens.typography.label.line_height)
                    .color(fg)
                    .font(if active {
                        iced::Font { weight: iced::font::Weight::Semibold, ..Default::default() }
                    } else {
                        iced::Font::default()
                    })
            )
            .padding(Padding::from([tokens.spacing.sm, tokens.spacing.md]))
            .style(move |_| iced::widget::container::Style {
                border: border_bottom,
                ..Default::default()
            })
        )
        .on_press_maybe(if active { None } else { Some(Message::SetDiffViewMode(target)) })
        .style({ let t = tokens.clone(); move |_th, s| crate::style::btn_ghost(&t, s) })
        .into()
    };

    let tab_s1 = t!("diff.tab_side_by_side").to_string();
    let tab_s2 = t!("diff.tab_unified").to_string();
    let tab_s3 = t!("diff.tab_changed_only").to_string();
    let tab_items: Vec<Element<'static, Message>> = vec![
        tab(tab_s1, DiffViewMode::SideBySide,  mode == DiffViewMode::SideBySide),
        tab(tab_s2, DiffViewMode::Unified,     mode == DiffViewMode::Unified),
        tab(tab_s3, DiffViewMode::ChangedOnly, mode == DiffViewMode::ChangedOnly),
    ];
    let tab_bar_border = crate::style::to_iced(tokens.palette.border);
    let tab_bar_bg = crate::style::to_iced(tokens.palette.surface);
    // T8 (RFC 099 §6a) — at 800 logical width the three-pane grid gives this
    // bar roughly 265 px, which the three tab labels exceed. Horizontal
    // scroll makes `Changes only` reachable instead of unrendered.
    iced::widget::container(
        scrollable(
            iced::widget::row(tab_items)
            .spacing(0)
            .align_y(iced::Alignment::Center)
        )
        .direction(scrollable::Direction::Horizontal(scrollable::Scrollbar::default()))
    )
    .width(Length::Fill)
    .style(move |_| iced::widget::container::Style {
        border: iced::Border {
            color: tab_bar_border,
            width: 0.0,
            ..Default::default()
        },
        background: Some(iced::Background::Color(tab_bar_bg)),
        ..Default::default()
    })
    .into()
}

fn placeholder(msg: String, tokens: &snora::design::Tokens) -> Element<'static, Message> {
    container(
        text(msg)
            .size(tokens.typography.body.size)
            .line_height(tokens.typography.body.line_height)
            .color(crate::style::to_iced(tokens.palette.text_secondary)),
    )
        .padding(tokens.spacing.lg)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Panel shown for binary files.
fn binary_panel<'a>(diff: &'a DiffEntry, tokens: &'a snora::design::Tokens) -> Element<'a, Message> {
    let kind_label = match diff.diff_type {
        DiffType::Added    => t!("diff.binary_file_added").to_string(),
        DiffType::Removed  => t!("diff.binary_file_removed").to_string(),
        DiffType::Modified => t!("diff.binary_file_modified").to_string(),
        _                  => t!("diff.binary_file").to_string(),
    };

    let mut rows: Vec<Element<'_, Message>> = vec![
        text(kind_label)
            .size(tokens.typography.title.size)
            .line_height(tokens.typography.title.line_height)
            .font(iced::Font {
                weight: iced::font::Weight::Semibold, ..Default::default()
            }).into(),
    ];

    // Size change
    if let Some(label) = diff.size_change_label() {
        rows.push(
            row![
                text(t!("diff.size_label").to_string())
                    .size(tokens.typography.label.size)
                    .line_height(tokens.typography.label.line_height)
                    .color(crate::style::to_iced(tokens.palette.text_secondary)),
                text(label)
                    .size(tokens.typography.body_small.size)
                    .line_height(tokens.typography.body_small.line_height),
            ].spacing(tokens.spacing.sm).into()
        );
    }

    // Hashes
    if let Some(h) = &diff.before_sha256 {
        rows.push(
            row![
                text(t!("diff.before_sha256_label").to_string())
                    .size(tokens.typography.label.size)
                    .line_height(tokens.typography.label.line_height)
                    .color(crate::style::to_iced(tokens.palette.text_secondary)),
                text(h.clone())
                    .size(tokens.typography.body_small.size)
                    .line_height(tokens.typography.body_small.line_height)
                    .font(iced::Font::MONOSPACE),
            ].spacing(tokens.spacing.sm).into()
        );
    }
    if let Some(h) = &diff.after_sha256 {
        rows.push(
            row![
                text(t!("diff.after_sha256_label").to_string())
                    .size(tokens.typography.label.size)
                    .line_height(tokens.typography.label.line_height)
                    .color(crate::style::to_iced(tokens.palette.text_secondary)),
                text(h.clone())
                    .size(tokens.typography.body_small.size)
                    .line_height(tokens.typography.body_small.line_height)
                    .font(iced::Font::MONOSPACE),
            ].spacing(tokens.spacing.sm).into()
        );
    }

    if diff.before_sha256.as_ref() == diff.after_sha256.as_ref()
        && diff.before_sha256.is_some()
    {
        rows.push(
            text(t!("diff.hashes_match").to_string())
                .size(tokens.typography.body_small.size)
                .line_height(tokens.typography.body_small.line_height)
                .color(crate::theme::added_color(tokens)).into(),
        );
    } else if diff.before_sha256.is_some() && diff.after_sha256.is_some() {
        rows.push(
            text(t!("diff.hashes_differ").to_string())
                .size(tokens.typography.body_small.size)
                .line_height(tokens.typography.body_small.line_height)
                .color(crate::theme::removed_color(tokens)).into(),
        );
    }

    container(
        column(rows).spacing(tokens.spacing.md).padding(tokens.spacing.xl),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Stats bar shown above the side-by-side diff.
fn stats_bar<'a>(diff: &'a DiffEntry, tokens: &'a snora::design::Tokens) -> Element<'a, Message> {
    

    let mut parts: Vec<Element<'_, Message>> = Vec::new();

    if let Some(stats) = &diff.stats {
        parts.push(
            text(format!("+{} lines", stats.lines_added))
                .size(tokens.typography.body_small.size)
                .line_height(tokens.typography.body_small.line_height)
                .color(crate::theme::added_color(tokens))
                .into()
        );
        parts.push(
            text(format!("  −{} lines", stats.lines_removed))
                .size(tokens.typography.body_small.size)
                .line_height(tokens.typography.body_small.line_height)
                .color(crate::theme::removed_color(tokens))
                .into()
        );
    }

    if let Some(label) = diff.size_change_label() {
        parts.push(
            text(t!("diff.size_inline", value = label).to_string())
                .size(tokens.typography.body_small.size)
                .line_height(tokens.typography.body_small.line_height)
                .color(crate::style::to_iced(tokens.palette.text_muted))
                .into()
        );
    }

    if parts.is_empty() {
        return iced::widget::space().height(Length::Fixed(0.0)).into();
    }

    let stats_row = row(parts).spacing(0).align_y(iced::Alignment::Center);

    let stats_bg = crate::style::to_iced(tokens.palette.surface);
    container(stats_row)
        .width(Length::Fill)
        .padding(Padding::from([tokens.spacing.xs, tokens.spacing.sm]))
        .style(move |_| iced::widget::container::Style {
            background: Some(iced::Background::Color(stats_bg)),
            ..Default::default()
        })
        .into()
}

fn side_by_side<'a>(diff: &'a DiffEntry, tokens: &'a snora::design::Tokens) -> Element<'a, Message> {
    let before_str = diff.before_text.as_deref().unwrap_or("");
    let after_str  = diff.after_text.as_deref().unwrap_or("");
    let text_diff  = TextDiff::from_lines(before_str, after_str);

    let mut before_lines: Vec<Element<'_, Message>> = Vec::new();
    let mut after_lines: Vec<Element<'_, Message>> = Vec::new();
    let mut line_num_b: usize = 1;
    let mut line_num_a: usize = 1;

    for change in text_diff.iter_all_changes() {
        let content = change.value().trim_end_matches('\n').to_string();
        match change.tag() {
            ChangeTag::Delete => {
                before_lines.push(diff_line(line_num_b, content, LineKind::Removed, tokens));
                line_num_b += 1;
            }
            ChangeTag::Insert => {
                after_lines.push(diff_line(line_num_a, content, LineKind::Added, tokens));
                line_num_a += 1;
            }
            ChangeTag::Equal => {
                after_lines.push(diff_line(line_num_a, content.clone(), LineKind::Equal, tokens));
                before_lines.push(diff_line(line_num_b, content, LineKind::Equal, tokens));
                line_num_b += 1;
                line_num_a += 1;
            }
        }
    }

    let before_col: Element<'_, Message> = scrollable(
        column(before_lines).spacing(0).width(Length::Fill)
    )
    .id(DIFF_BEFORE_ID.clone())
    .on_scroll(Message::DiffBeforeScrolled)
    .width(Length::Fill).height(Length::Fill).into();

    let after_col: Element<'_, Message> = scrollable(
        column(after_lines).spacing(0).width(Length::Fill)
    )
    .id(DIFF_AFTER_ID.clone())
    .on_scroll(Message::DiffAfterScrolled)
    .width(Length::Fill).height(Length::Fill).into();

    let header = row![
        container(
            text(t!("diff.before").to_string())
                .size(tokens.typography.title.size)
                .line_height(tokens.typography.title.line_height)
                .font(iced::Font {
                    weight: iced::font::Weight::Semibold, ..Default::default()
                }),
        )
        .padding(Padding::from([tokens.spacing.xs, tokens.spacing.sm]))
        .width(Length::FillPortion(1)),
        container(
            text(t!("diff.after").to_string())
                .size(tokens.typography.title.size)
                .line_height(tokens.typography.title.line_height)
                .font(iced::Font {
                    weight: iced::font::Weight::Semibold, ..Default::default()
                }),
        )
        .padding(Padding::from([tokens.spacing.xs, tokens.spacing.sm]))
        .width(Length::FillPortion(1)),
    ]
    .spacing(tokens.spacing.xs);

    let body = row![before_col, after_col].spacing(tokens.spacing.xs).height(Length::Fill);

    column![
        header,
        stats_bar(diff, tokens),
        body,
        diff_legend(tokens),   // RFC 010: colour legend
    ]
    .spacing(0)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// RFC 010: Colour legend shown at the bottom of the diff view.
fn diff_legend(tokens: &snora::design::Tokens) -> Element<'static, Message> {
    use iced::Background;

    let removed = crate::theme::removed_color(tokens);
    let added = crate::theme::added_color(tokens);
    let muted = crate::style::to_iced(tokens.palette.text_muted);

    let removed_swatch: Element<'static, Message> =
        iced::widget::container(iced::widget::text("  "))
            .height(Length::Fixed(12.0))
            .style(move |_| iced::widget::container::Style {
                background: Some(Background::Color(
                    Color { a: 0.30, ..removed })),
                border: iced::Border { radius: 2.0.into(), ..Default::default() },
                ..Default::default()
            })
            .into();

    let added_swatch: Element<'static, Message> =
        iced::widget::container(iced::widget::text("  "))
            .height(Length::Fixed(12.0))
            .style(move |_| iced::widget::container::Style {
                background: Some(Background::Color(
                    Color { a: 0.30, ..added })),
                border: iced::Border { radius: 2.0.into(), ..Default::default() },
                ..Default::default()
            })
            .into();

    let legend_row = iced::widget::row(vec![
        text(t!("diff.legend_label").to_string())
            .size(tokens.typography.body_small.size)
            .line_height(tokens.typography.body_small.line_height)
            .color(muted).into(),
        removed_swatch,
        text(t!("diff.legend_removed").to_string())
            .size(tokens.typography.body_small.size)
            .line_height(tokens.typography.body_small.line_height)
            .color(muted).into(),
        added_swatch,
        text(t!("diff.legend_added").to_string())
            .size(tokens.typography.body_small.size)
            .line_height(tokens.typography.body_small.line_height)
            .color(muted).into(),
    ]).spacing(tokens.spacing.sm).align_y(iced::Alignment::Center);

    // T8 (RFC 099 §6a) — same overflow as the tab bar: at 800 logical width
    // `Added` does not fit and was not rendered at all.
    iced::widget::container(
        scrollable(legend_row)
            .direction(scrollable::Direction::Horizontal(scrollable::Scrollbar::default()))
    )
        .padding(Padding::from([tokens.spacing.xs, tokens.spacing.md]))
        .width(Length::Fill)
        .into()
}

enum LineKind { Equal, Added, Removed }

fn diff_line(num: usize, content: String, kind: LineKind, tokens: &snora::design::Tokens) -> Element<'static, Message> {
    let bg = match kind {
        LineKind::Equal   => None,
        LineKind::Added   => Some(Color { a: 0.15, ..crate::theme::added_color(tokens) }),
        LineKind::Removed => Some(Color { a: 0.15, ..crate::theme::removed_color(tokens) }),
    };
    let line_num = container(
        text(num.to_string())
            .size(tokens.typography.body_small.size)
            .line_height(tokens.typography.body_small.line_height)
            .font(iced::Font::MONOSPACE)
            .color(crate::style::to_iced(tokens.palette.text_muted))
    )
    .padding(Padding::from([tokens.spacing.xs, tokens.spacing.xs]))
    .width(Length::Fixed(36.0));

    let inner = row![
        line_num,
        text(content)
            .size(tokens.typography.body.size)
            .line_height(tokens.typography.body.line_height)
            .font(iced::Font::MONOSPACE),
    ]
    .align_y(iced::Alignment::Center);

    if let Some(color) = bg {
        container(inner)
            .width(Length::Fill)
            .padding(Padding::from([tokens.spacing.xs, tokens.spacing.xs]))
            .style(move |_| iced::widget::container::Style {
                background: Some(iced::Background::Color(color)),
                ..Default::default()
            })
            .into()
    } else {
        container(inner)
            .width(Length::Fill)
            .padding(Padding::from([tokens.spacing.xs, tokens.spacing.xs]))
            .into()
    }
}

// ── RFC 011: Unified view ──────────────────────────────────────────────────────

fn unified_view<'a>(diff: &'a DiffEntry, tokens: &'a snora::design::Tokens) -> Element<'a, Message> {
    let before = diff.before_text.as_deref().unwrap_or("");
    let after  = diff.after_text.as_deref().unwrap_or("");

    if before.is_empty() && after.is_empty() {
        return placeholder(t!("diff.no_text_content").to_string(), tokens);
    }

    let td = similar::TextDiff::from_lines(before, after);
    let mut rows: Vec<Element<'_, Message>> = Vec::new();

    // Collect into owned data first to avoid borrow conflicts
    let changes: Vec<(similar::ChangeTag, String)> = td.iter_all_changes()
        .map(|c| (c.tag(), c.value().to_owned()))
        .collect();
    drop(td);

    for (tag, value) in changes {
        let line_str: String = value.trim_end_matches('\n').to_string();
        let (prefix, bg) = match tag {
            similar::ChangeTag::Delete =>
                ("-", Color { a: 0.12, ..crate::theme::removed_color(tokens) }),
            similar::ChangeTag::Insert =>
                ("+", Color { a: 0.12, ..crate::theme::added_color(tokens) }),
            similar::ChangeTag::Equal  =>
                (" ", Color::TRANSPARENT),
        };
        rows.push(
            iced::widget::container(
                row![
                    text(prefix)
                        .size(tokens.typography.label.size)
                        .line_height(tokens.typography.label.line_height)
                        .font(iced::Font::MONOSPACE)
                        .color(if prefix == "-" {
                            crate::theme::removed_color(tokens)
                        } else if prefix == "+" {
                            crate::theme::added_color(tokens)
                        } else {
                            crate::style::to_iced(tokens.palette.text_muted)
                        })
                        .width(Length::Fixed(14.0)),
                    text(line_str)
                        .size(tokens.typography.body.size)
                        .line_height(tokens.typography.body.line_height)
                        .font(iced::Font::MONOSPACE),
                ]
                .spacing(tokens.spacing.xs)
                .padding(Padding::from([tokens.spacing.xs, tokens.spacing.sm]))
            )
            .width(Length::Fill)
            .style(move |_| iced::widget::container::Style {
                background: Some(iced::Background::Color(bg)),
                ..Default::default()
            })
            .into()
        );
    }

    column![
        stats_bar(diff, tokens),
        scrollable(column(rows).width(Length::Fill)).height(Length::Fill),
        diff_legend(tokens),
    ]
    .spacing(0)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

// ── RFC 011: Changed-only view ────────────────────────────────────────────────

fn changed_only_view<'a>(diff: &'a DiffEntry, tokens: &'a snora::design::Tokens) -> Element<'a, Message> {
    let before = diff.before_text.as_deref().unwrap_or("");
    let after  = diff.after_text.as_deref().unwrap_or("");

    if before.is_empty() && after.is_empty() {
        return placeholder(t!("diff.no_text_content").to_string(), tokens);
    }

    let td = similar::TextDiff::from_lines(before, after);
    let mut rows: Vec<Element<'_, Message>> = Vec::new();
    let mut last_was_equal = false;

    let co_changes: Vec<(similar::ChangeTag, String)> = td.iter_all_changes()
        .map(|c| (c.tag(), c.value().to_owned()))
        .collect();
    drop(td);

    for (tag, value) in co_changes {
        match tag {
            similar::ChangeTag::Equal => {
                if !last_was_equal {
                    rows.push(
                        iced::widget::container(
                            text("···")
                                .size(tokens.typography.label.size)
                                .line_height(tokens.typography.label.line_height)
                                .color(crate::style::to_iced(tokens.palette.text_muted))
                                .font(iced::Font::MONOSPACE)
                        )
                        .padding(Padding::from([tokens.spacing.xs, tokens.spacing.sm]))
                        .width(Length::Fill)
                        .into()
                    );
                }
                last_was_equal = true;
            }
            tag => {
                last_was_equal = false;
                let line_str: String = value.trim_end_matches('\n').to_string();
                let (prefix, bg, fg) = if tag == similar::ChangeTag::Delete {
                    let removed = crate::theme::removed_color(tokens);
                    ("-", Color { a: 0.12, ..removed }, removed)
                } else {
                    let added = crate::theme::added_color(tokens);
                    ("+", Color { a: 0.12, ..added }, added)
                };
                rows.push(
                    iced::widget::container(
                        row![
                            text(prefix)
                                .size(tokens.typography.label.size)
                                .line_height(tokens.typography.label.line_height)
                                .font(iced::Font::MONOSPACE)
                                .color(fg).width(Length::Fixed(14.0)),
                            text(line_str)
                                .size(tokens.typography.body.size)
                                .line_height(tokens.typography.body.line_height)
                                .font(iced::Font::MONOSPACE),
                        ]
                        .spacing(tokens.spacing.xs)
                        .padding(Padding::from([tokens.spacing.xs, tokens.spacing.sm]))
                    )
                    .width(Length::Fill)
                    .style(move |_| iced::widget::container::Style {
                        background: Some(iced::Background::Color(bg)),
                        ..Default::default()
                    })
                    .into()
                );
            }
        }
    }

    if rows.is_empty() {
        return placeholder(t!("diff.no_changes").to_string(), tokens);
    }

    column![
        stats_bar(diff, tokens),
        scrollable(column(rows).width(Length::Fill)).height(Length::Fill),
        diff_legend(tokens),
    ]
    .spacing(0)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
