//! Dashboard view (Phase 5) — summary cards shown before a file is selected.

use iced::{
    Color, Element, Length, Padding,
    widget::{button, column, container, row, space, text},
};
use rust_i18n::t;

use aaai::{AuditResult, AuditStatus};
use crate::app::Message;

pub fn view<'a>(result: &'a AuditResult, tokens: &'a snora::design::Tokens, is_hc: bool) -> Element<'a, Message> {
    let s = &result.summary;

    // ── Stat cards ────────────────────────────────────────────────────
    let cards = row![
        stat_card(t!("status.ok").to_string(),      s.ok,      crate::theme::status_color(aaai::AuditStatus::Ok, tokens, is_hc), tokens),
        stat_card(t!("status.pending").to_string(),  s.pending, crate::theme::status_color(aaai::AuditStatus::Pending, tokens, is_hc), tokens),
        stat_card(t!("status.failed").to_string(),   s.failed,  crate::theme::status_color(aaai::AuditStatus::Failed, tokens, is_hc), tokens),
        stat_card(t!("status.error").to_string(),    s.error,   crate::theme::status_color(aaai::AuditStatus::Error, tokens, is_hc), tokens),
        stat_card(t!("status.ignored").to_string(),  s.ignored, crate::theme::status_color(aaai::AuditStatus::Ignored, tokens, is_hc), tokens),
    ]
    .spacing(tokens.spacing.md);

    // ── Verdict banner ────────────────────────────────────────────────
    let (verdict_text, verdict_color) = if s.is_passing() {
        (t!("status.passed").to_string(), crate::theme::status_color(aaai::AuditStatus::Ok, tokens, is_hc))
    } else {
        (t!("status.result_failed").to_string(), crate::theme::status_color(aaai::AuditStatus::Failed, tokens, is_hc))
    };

    let verdict_banner = container(
        text(verdict_text)
            .size(tokens.typography.heading.size)
            .line_height(tokens.typography.heading.line_height)
            .font(iced::Font {
                weight: iced::font::Weight::Bold, ..Default::default()
            }).color(Color::WHITE)
    )
    .padding(Padding::from([tokens.spacing.md, tokens.spacing.xxl]))
    .style(move |_| iced::widget::container::Style {
        background: Some(iced::Background::Color(verdict_color)),
        border: iced::Border { radius: 8.0.into(), ..Default::default() },
        ..Default::default()
    });

    // ── Attention list ────────────────────────────────────────────────
    let attention: Vec<_> = result.results.iter()
        .filter(|r| matches!(r.status, AuditStatus::Failed | AuditStatus::Pending | AuditStatus::Error)
                 && r.diff.diff_type != aaai::DiffType::Unchanged)
        .take(8)
        .collect();

    let attention_section: Element<'_, Message> = if !attention.is_empty() {
        let mut col = column![
            text(t!("dashboard.needs_attention").to_string())
                .size(tokens.typography.title.size)
                .line_height(tokens.typography.title.line_height)
                .font(iced::Font {
                    weight: iced::font::Weight::Semibold, ..Default::default()
                })
        ].spacing(tokens.spacing.sm);

        for r in &attention {
            let badge_color = match r.status {
                AuditStatus::Failed  => crate::theme::status_color(aaai::AuditStatus::Failed, tokens, is_hc),
                AuditStatus::Pending => crate::theme::status_color(aaai::AuditStatus::Pending, tokens, is_hc),
                AuditStatus::Error   => crate::theme::status_color(aaai::AuditStatus::Error, tokens, is_hc),
                _                    => crate::theme::status_color(aaai::AuditStatus::Ignored, tokens, is_hc),
            };
            let badge = container(
                text(r.status.to_string())
                    .size(tokens.typography.label.size)
                    .line_height(tokens.typography.label.line_height)
                    .color(Color::WHITE)
            )
            .padding(Padding::from([tokens.spacing.xs, tokens.spacing.sm]))
            .style(move |_| iced::widget::container::Style {
                background: Some(iced::Background::Color(badge_color)),
                border: iced::Border { radius: 3.0.into(), ..Default::default() },
                ..Default::default()
            });

            col = col.push(
                row![
                    badge,
                    text(r.diff.path.clone())
                        .size(tokens.typography.body_small.size)
                        .line_height(tokens.typography.body_small.line_height)
                        .font(iced::Font::MONOSPACE),
                ]
                .spacing(tokens.spacing.sm)
                .align_y(iced::Alignment::Center)
            );
        }
        col.into()
    } else {
        // RFC 053 — all-clear state: show CTA buttons instead of static text.
        let all_clear_label = text(t!("empty_state.dashboard_all_clear").to_string())
            .size(tokens.typography.body.size)
            .line_height(tokens.typography.body.line_height)
            .color(crate::style::to_iced(tokens.palette.success));

        let export_btn = button(
            text(t!("toolbar.report_output").to_string())
                .size(tokens.typography.label.size)
                .line_height(tokens.typography.label.line_height)
        )
        .on_press(crate::app::Message::ExportReport)
        .padding(Padding::from([tokens.spacing.sm, tokens.spacing.xl]));

        let new_audit_btn = button(
            text(t!("dashboard.new_audit").to_string())
                .size(tokens.typography.label.size)
                .line_height(tokens.typography.label.line_height)
        )
        .on_press(crate::app::Message::BackToOpening)
        .padding(Padding::from([tokens.spacing.sm, tokens.spacing.xl]))
        .style({ let t = tokens.clone(); move |_th, s| crate::style::btn_secondary(&t, s) });

        column![
            all_clear_label,
            space().height(Length::Fixed(16.0)),
            row![export_btn, new_audit_btn].spacing(tokens.spacing.md),
        ]
        .spacing(0)
        .into()
    };

    // Hint — only shown when there ARE items needing attention.
    // When all-clear, the CTA buttons above replace the hint.
    let hint_visible = !attention.is_empty();

    let mut main_col = column![
        verdict_banner,
        space().height(Length::Fixed(16.0)),
        cards,
        space().height(Length::Fixed(20.0)),
        attention_section,
        space().height(Length::Fill),
    ]
    .spacing(0)
    .padding(Padding::from([tokens.spacing.xl, tokens.spacing.xxl]));

    if hint_visible {
        main_col = main_col.push(
            text(t!("empty_state.dashboard_select_file").to_string())
                .size(tokens.typography.body_small.size)
                .line_height(tokens.typography.body_small.line_height)
                .color(crate::style::to_iced(tokens.palette.text_secondary))
        );
    }

    container(main_col)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn stat_card<'a>(label: String, count: usize, color: Color, tokens: &snora::design::Tokens) -> Element<'a, Message> {
    let label_color = crate::style::to_iced(tokens.palette.text_secondary);
    let card_bg = crate::style::to_iced(tokens.palette.surface);
    let card_border = crate::style::to_iced(tokens.palette.border);
    container(
        column![
            text(count.to_string())
                .size(tokens.typography.display.size)
                .line_height(tokens.typography.display.line_height)
                .font(iced::Font {
                    weight: iced::font::Weight::Bold, ..Default::default()
                }).color(color),
            text(label)
                .size(tokens.typography.body_small.size)
                .line_height(tokens.typography.body_small.line_height)
                .color(label_color),
        ]
        .spacing(tokens.spacing.xs)
        .align_x(iced::Alignment::Center)
        .width(Length::Fill),
    )
    .padding(Padding::from([tokens.spacing.lg, tokens.spacing.lg]))
    .width(Length::FillPortion(1))
    .style(move |_| iced::widget::container::Style {
        background: Some(iced::Background::Color(card_bg)),
        border: iced::Border {
            color: card_border,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    })
    .into()
}
