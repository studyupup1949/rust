//! Side-by-side diff viewer (Phase 4: binary file panel, stats bar, masking).

use iced::{
    Color, Element, Length, Padding,
    widget::{column, container, row, scrollable, text},
};
use rust_i18n::t;
use similar::{ChangeTag, TextDiff};

use aaai_core::{DiffType, diff::entry::DiffEntry};
use crate::app::Message;

pub fn view<'a>(diff: &'a DiffEntry, mode: crate::app::DiffViewMode) -> Element<'a, Message> {
    use crate::app::DiffViewMode;

    if diff.is_dir {
        return placeholder(t!("diff.directory").to_string());
    }
    match diff.diff_type {
        DiffType::Unchanged => placeholder(t!("diff.identical").to_string()),
        DiffType::Unreadable | DiffType::Incomparable => {
            let msg = diff.error_detail.clone()
                .unwrap_or_else(|| t!("diff.unreadable").to_string());
            placeholder(msg)
        }
        _ if diff.is_binary => binary_panel(diff),
        _ => {
            // RFC 011: tab bar + selected view
            let has_text = diff.has_text_diff();
            let tab_bar = build_tab_bar(mode, has_text);
            let content = match mode {
                DiffViewMode::SideBySide  => side_by_side(diff),
                DiffViewMode::Unified     => unified_view(diff),
                DiffViewMode::ChangedOnly => changed_only_view(diff),
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
fn build_tab_bar(mode: crate::app::DiffViewMode, _has_text: bool) -> Element<'static, Message> {
    use crate::app::DiffViewMode;

    let tab = |label: String, target: DiffViewMode, active: bool| -> Element<'static, Message> {
        let fg = if active {
            iced::Color::from_rgb(0.10, 0.35, 0.80)
        } else {
            iced::Color::from_rgb(0.45, 0.47, 0.52)
        };
        let border_bottom = if active {
            iced::Border {
                color: iced::Color::from_rgb(0.10, 0.35, 0.80),
                width: 2.0,
                radius: 0.0.into(),
            }
        } else {
            iced::Border::default()
        };
        iced::widget::button(
            iced::widget::container(
                text(label).size(12).color(fg)
                    .font(if active {
                        iced::Font { weight: iced::font::Weight::Semibold, ..Default::default() }
                    } else {
                        iced::Font::default()
                    })
            )
            .padding(Padding::from([5.0, 12.0]))
            .style(move |_| iced::widget::container::Style {
                border: border_bottom,
                ..Default::default()
            })
        )
        .on_press_maybe(if active { None } else { Some(Message::SetDiffViewMode(target)) })
        .style(iced::widget::button::text)
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
    iced::widget::container(
        iced::widget::row(tab_items)
        .spacing(0)
        .align_y(iced::Alignment::Center)
    )
    .width(Length::Fill)
    .style(|_| iced::widget::container::Style {
        border: iced::Border {
            color: iced::Color::from_rgb(0.85, 0.86, 0.88),
            width: 0.0,
            ..Default::default()
        },
        background: Some(iced::Background::Color(iced::Color::from_rgb(0.96, 0.97, 0.98))),
        ..Default::default()
    })
    .into()
}

fn placeholder(msg: String) -> Element<'static, Message> {
    container(text(msg).size(13).color(Color::from_rgb(0.5, 0.5, 0.5)))
        .padding(16)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Panel shown for binary files.
fn binary_panel<'a>(diff: &'a DiffEntry) -> Element<'a, Message> {
    let kind_label = match diff.diff_type {
        DiffType::Added    => t!("diff.binary_file_added").to_string(),
        DiffType::Removed  => t!("diff.binary_file_removed").to_string(),
        DiffType::Modified => t!("diff.binary_file_modified").to_string(),
        _                  => t!("diff.binary_file").to_string(),
    };

    let mut rows: Vec<Element<'_, Message>> = vec![
        text(kind_label).size(14).font(iced::Font {
            weight: iced::font::Weight::Semibold, ..Default::default()
        }).into(),
    ];

    // Size change
    if let Some(label) = diff.size_change_label() {
        rows.push(
            row![
                text(t!("diff.size_label").to_string()).size(12).color(Color::from_rgb(0.5,0.5,0.5)),
                text(label).size(12),
            ].spacing(8).into()
        );
    }

    // Hashes
    if let Some(h) = &diff.before_sha256 {
        rows.push(
            row![
                text(t!("diff.before_sha256_label").to_string()).size(11).color(Color::from_rgb(0.5,0.5,0.5)),
                text(h.clone()).size(11).font(iced::Font::MONOSPACE),
            ].spacing(8).into()
        );
    }
    if let Some(h) = &diff.after_sha256 {
        rows.push(
            row![
                text(t!("diff.after_sha256_label").to_string()).size(11).color(Color::from_rgb(0.5,0.5,0.5)),
                text(h.clone()).size(11).font(iced::Font::MONOSPACE),
            ].spacing(8).into()
        );
    }

    if diff.before_sha256.as_ref() == diff.after_sha256.as_ref()
        && diff.before_sha256.is_some()
    {
        rows.push(text(t!("diff.hashes_match").to_string()).size(12).color(Color::from_rgb(0.18,0.65,0.32)).into());
    } else if diff.before_sha256.is_some() && diff.after_sha256.is_some() {
        rows.push(text(t!("diff.hashes_differ").to_string()).size(12).color(Color::from_rgb(0.82,0.18,0.18)).into());
    }

    container(
        column(rows).spacing(10).padding(20.0),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Stats bar shown above the side-by-side diff.
fn stats_bar<'a>(diff: &'a DiffEntry) -> Element<'a, Message> {
    

    let mut parts: Vec<Element<'_, Message>> = Vec::new();

    if let Some(stats) = &diff.stats {
        parts.push(
            text(format!("+{} lines", stats.lines_added)).size(11)
                .color(Color::from_rgb(0.18, 0.65, 0.32))
                .into()
        );
        parts.push(
            text(format!("  −{} lines", stats.lines_removed)).size(11)
                .color(Color::from_rgb(0.82, 0.18, 0.18))
                .into()
        );
    }

    if let Some(label) = diff.size_change_label() {
        parts.push(
            text(format!("  Size: {label}")).size(11)
                .color(Color::from_rgb(0.50, 0.50, 0.55))
                .into()
        );
    }

    if parts.is_empty() {
        return iced::widget::space().height(Length::Fixed(0.0)).into();
    }

    let stats_row = row(parts).spacing(0).align_y(iced::Alignment::Center);

    container(stats_row)
        .width(Length::Fill)
        .padding(Padding::from([3.0, 8.0]))
        .style(|_| iced::widget::container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.96, 0.97, 0.98))),
            ..Default::default()
        })
        .into()
}

fn side_by_side<'a>(diff: &'a DiffEntry) -> Element<'a, Message> {
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
                before_lines.push(diff_line(line_num_b, content, LineKind::Removed));
                line_num_b += 1;
            }
            ChangeTag::Insert => {
                after_lines.push(diff_line(line_num_a, content, LineKind::Added));
                line_num_a += 1;
            }
            ChangeTag::Equal => {
                after_lines.push(diff_line(line_num_a, content.clone(), LineKind::Equal));
                before_lines.push(diff_line(line_num_b, content, LineKind::Equal));
                line_num_b += 1;
                line_num_a += 1;
            }
        }
    }

    let before_col: Element<'_, Message> = scrollable(
        column(before_lines).spacing(0).width(Length::Fill)
    ).width(Length::Fill).height(Length::Fill).into();

    let after_col: Element<'_, Message> = scrollable(
        column(after_lines).spacing(0).width(Length::Fill)
    ).width(Length::Fill).height(Length::Fill).into();

    let header = row![
        container(text(t!("diff.before").to_string()).size(12).font(iced::Font {
            weight: iced::font::Weight::Semibold, ..Default::default()
        }))
        .padding(Padding::from([4.0, 8.0]))
        .width(Length::FillPortion(1)),
        container(text(t!("diff.after").to_string()).size(12).font(iced::Font {
            weight: iced::font::Weight::Semibold, ..Default::default()
        }))
        .padding(Padding::from([4.0, 8.0]))
        .width(Length::FillPortion(1)),
    ]
    .spacing(1);

    let body = row![before_col, after_col].spacing(1).height(Length::Fill);

    column![
        header,
        stats_bar(diff),
        body,
        diff_legend(),   // RFC 010: colour legend
    ]
    .spacing(0)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// RFC 010: Colour legend shown at the bottom of the diff view.
fn diff_legend() -> Element<'static, Message> {
    use iced::Background;

    let removed_swatch: Element<'static, Message> =
        iced::widget::container(iced::widget::text("  "))
            .height(Length::Fixed(12.0))
            .style(|_| iced::widget::container::Style {
                background: Some(Background::Color(
                    Color::from_rgba(0.85, 0.20, 0.20, 0.30))),
                border: iced::Border { radius: 2.0.into(), ..Default::default() },
                ..Default::default()
            })
            .into();

    let added_swatch: Element<'static, Message> =
        iced::widget::container(iced::widget::text("  "))
            .height(Length::Fixed(12.0))
            .style(|_| iced::widget::container::Style {
                background: Some(Background::Color(
                    Color::from_rgba(0.10, 0.65, 0.30, 0.30))),
                border: iced::Border { radius: 2.0.into(), ..Default::default() },
                ..Default::default()
            })
            .into();

    let legend_row = iced::widget::row(vec![
        text(t!("diff.legend_label").to_string()).size(11)
            .color(Color::from_rgb(0.55, 0.55, 0.60)).into(),
        removed_swatch,
        text(t!("diff.legend_removed").to_string()).size(11)
            .color(Color::from_rgb(0.55, 0.55, 0.60)).into(),
        added_swatch,
        text(t!("diff.legend_added").to_string()).size(11)
            .color(Color::from_rgb(0.55, 0.55, 0.60)).into(),
    ]).spacing(6).align_y(iced::Alignment::Center);

    iced::widget::container(legend_row)
        .padding(Padding::from([4.0, 12.0]))
        .width(Length::Fill)
        .into()
}

enum LineKind { Equal, Added, Removed }

fn diff_line(num: usize, content: String, kind: LineKind) -> Element<'static, Message> {
    let bg = match kind {
        LineKind::Equal   => None,
        LineKind::Added   => Some(Color::from_rgba(0.20, 0.75, 0.35, 0.15)),
        LineKind::Removed => Some(Color::from_rgba(0.95, 0.25, 0.25, 0.15)),
    };
    let line_num = container(
        text(num.to_string()).size(11).font(iced::Font::MONOSPACE)
            .color(Color::from_rgb(0.60, 0.60, 0.65))
    )
    .padding(Padding::from([1.0, 4.0]))
    .width(Length::Fixed(36.0));

    let inner = row![
        line_num,
        text(content).size(12).font(iced::Font::MONOSPACE),
    ]
    .align_y(iced::Alignment::Center);

    if let Some(color) = bg {
        container(inner)
            .width(Length::Fill)
            .padding(Padding::from([1.0, 2.0]))
            .style(move |_| iced::widget::container::Style {
                background: Some(iced::Background::Color(color)),
                ..Default::default()
            })
            .into()
    } else {
        container(inner)
            .width(Length::Fill)
            .padding(Padding::from([1.0, 2.0]))
            .into()
    }
}

// ── RFC 011: Unified view ──────────────────────────────────────────────────────

fn unified_view<'a>(diff: &'a DiffEntry) -> Element<'a, Message> {
    let before = diff.before_text.as_deref().unwrap_or("");
    let after  = diff.after_text.as_deref().unwrap_or("");

    if before.is_empty() && after.is_empty() {
        return placeholder(t!("diff.no_text_content").to_string());
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
                ("-", iced::Color::from_rgba(0.85, 0.20, 0.20, 0.12)),
            similar::ChangeTag::Insert =>
                ("+", iced::Color::from_rgba(0.10, 0.65, 0.30, 0.12)),
            similar::ChangeTag::Equal  =>
                (" ", iced::Color::from_rgba(0.0, 0.0, 0.0, 0.0)),
        };
        rows.push(
            iced::widget::container(
                row![
                    text(prefix).size(11)
                        .font(iced::Font::MONOSPACE)
                        .color(if prefix == "-" {
                            iced::Color::from_rgb(0.75, 0.15, 0.15)
                        } else if prefix == "+" {
                            iced::Color::from_rgb(0.10, 0.55, 0.25)
                        } else {
                            iced::Color::from_rgb(0.50, 0.50, 0.55)
                        })
                        .width(Length::Fixed(14.0)),
                    text(line_str).size(11).font(iced::Font::MONOSPACE),
                ]
                .spacing(4)
                .padding(Padding::from([1.0, 8.0]))
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
        stats_bar(diff),
        scrollable(column(rows).width(Length::Fill)).height(Length::Fill),
        diff_legend(),
    ]
    .spacing(0)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

// ── RFC 011: Changed-only view ────────────────────────────────────────────────

fn changed_only_view<'a>(diff: &'a DiffEntry) -> Element<'a, Message> {
    let before = diff.before_text.as_deref().unwrap_or("");
    let after  = diff.after_text.as_deref().unwrap_or("");

    if before.is_empty() && after.is_empty() {
        return placeholder(t!("diff.no_text_content").to_string());
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
                            text("···").size(10)
                                .color(iced::Color::from_rgb(0.65, 0.65, 0.70))
                                .font(iced::Font::MONOSPACE)
                        )
                        .padding(Padding::from([2.0, 8.0]))
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
                    ("-", iced::Color::from_rgba(0.85, 0.20, 0.20, 0.12),
                          iced::Color::from_rgb(0.75, 0.15, 0.15))
                } else {
                    ("+", iced::Color::from_rgba(0.10, 0.65, 0.30, 0.12),
                          iced::Color::from_rgb(0.10, 0.55, 0.25))
                };
                rows.push(
                    iced::widget::container(
                        row![
                            text(prefix).size(11).font(iced::Font::MONOSPACE)
                                .color(fg).width(Length::Fixed(14.0)),
                            text(line_str).size(11).font(iced::Font::MONOSPACE),
                        ]
                        .spacing(4)
                        .padding(Padding::from([1.0, 8.0]))
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
        return placeholder(t!("diff.no_changes").to_string());
    }

    column![
        stats_bar(diff),
        scrollable(column(rows).width(Length::Fill)).height(Length::Fill),
        diff_legend(),
    ]
    .spacing(0)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
