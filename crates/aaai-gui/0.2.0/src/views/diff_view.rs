//! Side-by-side diff viewer with line numbers (Phase 2).

use iced::{
    Color, Element, Length, Padding,
    widget::{column, container, row, scrollable, text},
};
use rust_i18n::t;
use similar::{ChangeTag, TextDiff};

use aaai_core::{DiffType, diff::entry::DiffEntry};
use crate::app::Message;

pub fn view<'a>(diff: &'a DiffEntry) -> Element<'a, Message> {
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
        _ => side_by_side(diff),
    }
}

fn placeholder(msg: String) -> Element<'static, Message> {
    container(text(msg).size(13).color(Color::from_rgb(0.5, 0.5, 0.5)))
        .padding(16)
        .width(Length::Fill)
        .height(Length::Fill)
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
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into();

    let after_col: Element<'_, Message> = scrollable(
        column(after_lines).spacing(0).width(Length::Fill)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into();

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
    column![header, body].spacing(0).width(Length::Fill).height(Length::Fill).into()
}

enum LineKind { Equal, Added, Removed }

fn diff_line(num: usize, content: String, kind: LineKind) -> Element<'static, Message> {
    let bg = match kind {
        LineKind::Equal   => None,
        LineKind::Added   => Some(Color::from_rgba(0.20, 0.75, 0.35, 0.15)),
        LineKind::Removed => Some(Color::from_rgba(0.95, 0.25, 0.25, 0.15)),
    };
    let num_str = num.to_string();
    let line_num = container(
        text(num_str).size(11).font(iced::Font::MONOSPACE)
            .color(Color::from_rgb(0.60, 0.60, 0.65))
    )
    .padding(Padding::from([1.0, 4.0]))
    .width(Length::Fixed(36.0));

    let inner = row![line_num, text(content).size(12).font(iced::Font::MONOSPACE)]
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
