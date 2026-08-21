//! Dashboard view (Phase 5) — summary cards shown before a file is selected.

use iced::{
    Color, Element, Length, Padding,
    widget::{column, container, row, space, text},
};
use rust_i18n::t;

use aaai_core::{AuditResult, AuditStatus};
use crate::app::Message;
use crate::theme;

#[allow(dead_code)]
pub fn view<'a>(result: &'a AuditResult) -> Element<'a, Message> {
    let s = &result.summary;

    // ── Stat cards ────────────────────────────────────────────────────
    let cards = row![
        stat_card(t!("status.ok").to_string(),      s.ok,      theme::OK_COLOR),
        stat_card(t!("status.pending").to_string(),  s.pending, theme::PENDING_COLOR),
        stat_card(t!("status.failed").to_string(),   s.failed,  theme::FAILED_COLOR),
        stat_card(t!("status.error").to_string(),    s.error,   theme::ERROR_COLOR),
        stat_card(t!("status.ignored").to_string(),  s.ignored, theme::IGNORED_COLOR),
    ]
    .spacing(12);

    // ── Verdict banner ────────────────────────────────────────────────
    let (verdict_text, verdict_color) = if s.is_passing() {
        (t!("status.passed").to_string(), theme::OK_COLOR)
    } else {
        (t!("status.result_failed").to_string(), theme::FAILED_COLOR)
    };

    let verdict_banner = container(
        text(verdict_text).size(22).font(iced::Font {
            weight: iced::font::Weight::Bold, ..Default::default()
        }).color(Color::WHITE)
    )
    .padding(Padding::from([12.0, 32.0]))
    .style(move |_| iced::widget::container::Style {
        background: Some(iced::Background::Color(verdict_color)),
        border: iced::Border { radius: 8.0.into(), ..Default::default() },
        ..Default::default()
    });

    // ── Attention list ────────────────────────────────────────────────
    let attention: Vec<_> = result.results.iter()
        .filter(|r| matches!(r.status, AuditStatus::Failed | AuditStatus::Pending | AuditStatus::Error)
                 && r.diff.diff_type != aaai_core::DiffType::Unchanged)
        .take(8)
        .collect();

    let attention_section: Element<'_, Message> = if !attention.is_empty() {
        let mut col = column![
            text("Needs attention").size(14).font(iced::Font {
                weight: iced::font::Weight::Semibold, ..Default::default()
            })
        ].spacing(6);

        for r in &attention {
            let badge_color = match r.status {
                AuditStatus::Failed  => theme::FAILED_COLOR,
                AuditStatus::Pending => theme::PENDING_COLOR,
                AuditStatus::Error   => theme::ERROR_COLOR,
                _                    => theme::IGNORED_COLOR,
            };
            let badge = container(
                text(r.status.to_string()).size(10).color(Color::WHITE)
            )
            .padding(Padding::from([2.0, 6.0]))
            .style(move |_| iced::widget::container::Style {
                background: Some(iced::Background::Color(badge_color)),
                border: iced::Border { radius: 3.0.into(), ..Default::default() },
                ..Default::default()
            });

            col = col.push(
                row![
                    badge,
                    text(r.diff.path.clone()).size(12).font(iced::Font::MONOSPACE),
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center)
            );
        }
        col.into()
    } else {
        container(
            text("All entries are in order.").size(13)
                .color(Color::from_rgb(0.3, 0.6, 0.3))
        )
        .into()
    };

    // ── Hint ──────────────────────────────────────────────────────────
    let hint = text("Select a file from the left panel to inspect it.")
        .size(12)
        .color(Color::from_rgb(0.55, 0.55, 0.60));

    container(
        column![
            verdict_banner,
            space().height(Length::Fixed(16.0)),
            cards,
            space().height(Length::Fixed(20.0)),
            attention_section,
            space().height(Length::Fill),
            hint,
        ]
        .spacing(0)
        .padding(Padding::from([24.0, 28.0])),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

#[allow(dead_code)]
fn stat_card<'a>(label: String, count: usize, color: Color) -> Element<'a, Message> {
    container(
        column![
            text(count.to_string()).size(32).font(iced::Font {
                weight: iced::font::Weight::Bold, ..Default::default()
            }).color(color),
            text(label).size(11).color(Color::from_rgb(0.5, 0.5, 0.5)),
        ]
        .spacing(2)
        .align_x(iced::Alignment::Center)
        .width(Length::Fill),
    )
    .padding(Padding::from([14.0, 16.0]))
    .width(Length::FillPortion(1))
    .style(|_| iced::widget::container::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.98, 0.98, 0.99))),
        border: iced::Border {
            color: Color::from_rgb(0.88, 0.88, 0.90),
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    })
    .into()
}
