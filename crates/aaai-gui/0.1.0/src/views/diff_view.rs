//! Side-by-side diff viewer (panel 2).

use iced::{
    Color, Element, Length, Padding,
    widget::{column, container, row, scrollable, text},
};
use similar::{ChangeTag, TextDiff};

use aaai_core::DiffType;
use aaai_core::diff::entry::DiffEntry;
use crate::app::Message;

pub fn view(diff: &DiffEntry) -> Element<'static, Message> {
    if diff.is_dir {
        return placeholder("(directory)");
    }
    match diff.diff_type {
        DiffType::Unchanged => placeholder("Files are identical."),
        DiffType::Unreadable | DiffType::Incomparable => {
            let msg = diff.error_detail.as_deref().unwrap_or("File cannot be compared.");
            placeholder(msg)
        }
        _ => side_by_side(diff),
    }
}

fn placeholder(msg: &str) -> Element<'static, Message> {
    let msg = msg.to_string();
    container(text(msg).size(13).color(Color::from_rgb(0.5, 0.5, 0.5)))
        .padding(16)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// One highlighted diff line for display.
#[derive(Clone)]
struct DiffLine {
    content: String,
    added: bool,   // green
    removed: bool, // red
}

fn collect_lines(before: &str, after: &str) -> (Vec<DiffLine>, Vec<DiffLine>) {
    let td = TextDiff::from_lines(before, after);
    let mut before_lines = Vec::new();
    let mut after_lines = Vec::new();

    for change in td.iter_all_changes() {
        let content = change.value().trim_end_matches('\n').to_string();
        match change.tag() {
            ChangeTag::Delete => {
                before_lines.push(DiffLine { content, added: false, removed: true });
            }
            ChangeTag::Insert => {
                after_lines.push(DiffLine { content, added: true, removed: false });
            }
            ChangeTag::Equal => {
                let line = DiffLine { content, added: false, removed: false };
                before_lines.push(line.clone());
                after_lines.push(line);
            }
        }
    }
    (before_lines, after_lines)
}

fn render_lines<Message: 'static>(lines: Vec<DiffLine>) -> Element<'static, Message> {
    let items: Vec<Element<'static, Message>> = lines.into_iter().map(|line| {
        let t = text(line.content)
            .size(12)
            .font(iced::Font::MONOSPACE);

        let bg = if line.added {
            Some(Color::from_rgba(0.20, 0.75, 0.35, 0.18))
        } else if line.removed {
            Some(Color::from_rgba(0.95, 0.25, 0.25, 0.18))
        } else {
            None
        };

        if let Some(color) = bg {
            container(t)
                .width(Length::Fill)
                .padding(Padding::from([1.0, 6.0]))
                .style(move |_| iced::widget::container::Style {
                    background: Some(iced::Background::Color(color)),
                    ..Default::default()
                })
                .into()
        } else {
            container(t)
                .width(Length::Fill)
                .padding(Padding::from([1.0, 6.0]))
                .into()
        }
    }).collect();

    scrollable(column(items).spacing(0).width(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn side_by_side(diff: &DiffEntry) -> Element<'static, Message> {
    let before_str = diff.before_text.as_deref().unwrap_or("");
    let after_str = diff.after_text.as_deref().unwrap_or("");

    let (before_lines, after_lines) = collect_lines(before_str, after_str);

    let header_style = |_: &iced::Theme| -> iced::widget::container::Style {
        iced::widget::container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.91, 0.91, 0.93))),
            ..Default::default()
        }
    };

    let before_header: Element<'static, Message> = container(
        text("Before").size(12).font(iced::Font {
            weight: iced::font::Weight::Semibold,
            ..Default::default()
        }),
    )
    .padding(Padding::from([4.0, 8.0]))
    .width(Length::Fill)
    .style(header_style)
    .into();

    let after_header: Element<'static, Message> = container(
        text("After").size(12).font(iced::Font {
            weight: iced::font::Weight::Semibold,
            ..Default::default()
        }),
    )
    .padding(Padding::from([4.0, 8.0]))
    .width(Length::Fill)
    .style(header_style)
    .into();

    let before_col: Element<'static, Message> = column![
        before_header,
        render_lines(before_lines),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into();

    let after_col: Element<'static, Message> = column![
        after_header,
        render_lines(after_lines),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into();

    row![before_col, after_col]
        .spacing(1)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
