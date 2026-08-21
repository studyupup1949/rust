//! Inspector panel (panel 3) — edit reason, strategy, and approve.

use iced::{
    Color, Element, Length, Padding,
    widget::{button, column, container, pick_list, row, rule, scrollable, space, text, text_input},
};

use aaai_core::{
    FileAuditResult, AuditStatus,
    config::definition::{AuditStrategy, LineAction, RegexTarget},
};
use crate::app::{App, InspectorState, Message};
use crate::theme;

const STRATEGIES: &[&str] = &["None", "Checksum", "LineMatch", "Regex", "Exact"];

pub fn view<'a>(app: &'a App, far: &'a FileAuditResult) -> Element<'a, Message> {
    let ins = &app.inspector;

    let path_text = text(&far.diff.path).size(13).font(iced::Font::MONOSPACE);
    let diff_type_text = text(far.diff.diff_type.to_string()).size(12);

    let status_color = match far.status {
        AuditStatus::Ok      => theme::OK_COLOR,
        AuditStatus::Pending => theme::PENDING_COLOR,
        AuditStatus::Failed  => theme::FAILED_COLOR,
        AuditStatus::Ignored => theme::IGNORED_COLOR,
        AuditStatus::Error   => theme::ERROR_COLOR,
    };
    let status_badge = container(
        text(far.status.to_string()).size(12).color(Color::WHITE),
    )
    .padding(Padding::from([2.0, 8.0]))
    .style(move |_| iced::widget::container::Style {
        background: Some(iced::Background::Color(status_color)),
        border: iced::Border { radius: 4.0.into(), ..Default::default() },
        ..Default::default()
    });

    // Reason input
    let reason_label = text("Reason").size(13).font(iced::Font {
        weight: iced::font::Weight::Semibold,
        ..Default::default()
    });
    let reason_input = text_input(
        "Why is this change allowed? (required)",
        &ins.reason,
    )
    .on_input(Message::ReasonChanged)
    .padding(8);

    // Strategy picker
    let strategy_label = text("Content Audit Strategy").size(13).font(iced::Font {
        weight: iced::font::Weight::Semibold,
        ..Default::default()
    });
    let strategy_desc = text(ins.strategy.description()).size(11)
        .color(Color::from_rgb(0.45, 0.45, 0.45));

    let strategy_pick = pick_list(
        STRATEGIES,
        Some(ins.strategy_label.as_str()),
        |s: &str| Message::StrategySelected(s.to_string()),
    )
    .padding(6);

    // Strategy-specific form
    let strategy_form = build_strategy_form(ins);

    // Validation error
    let val_err: Option<Element<'_, Message>> = ins.validation_error.as_ref().map(|e| {
        text(e.as_str()).size(12).color(Color::from_rgb(0.78, 0.10, 0.10)).into()
    });

    // Note input
    let note_label = text("Note (optional)").size(12);
    let note_input = text_input("Additional context...", &ins.note)
        .on_input(Message::NoteChanged)
        .padding(6);

    // Approve button — only enabled when reason non-empty and strategy valid
    let can_approve = !ins.reason.trim().is_empty()
        && ins.validation_error.is_none();

    let approve_btn = button(
        text("Approve & Apply").size(14).font(iced::Font {
            weight: iced::font::Weight::Semibold,
            ..Default::default()
        }),
    )
    .on_press_maybe(if can_approve { Some(Message::ApproveEntry) } else { None })
    .padding(Padding::from([9.0, 20.0]));

    let mut col = column![
        row![
            column![path_text, diff_type_text].spacing(2),
            space().width(Length::Fill),
            status_badge,
        ]
        .spacing(8)
        .align_y(iced::Alignment::Start),
        rule::horizontal(1),
        reason_label,
        reason_input,
        strategy_label,
        strategy_pick,
        strategy_desc,
        strategy_form,
        note_label,
        note_input,
        space().height(Length::Fixed(4.0)),
        row![approve_btn].spacing(8),
    ]
    .spacing(10)
    .padding(Padding::from([16.0, 16.0]));

    if let Some(err) = val_err {
        col = col.push(err);
    }

    scrollable(
        container(col)
            .width(Length::Fill)
    )
    .width(Length::Fixed(280.0))
    .height(Length::Fill)
    .into()
}

fn build_strategy_form<'a>(ins: &'a InspectorState) -> Element<'a, Message> {
    match &ins.strategy {
        AuditStrategy::None => {
            text("No content inspection.").size(12)
                .color(Color::from_rgb(0.5, 0.5, 0.5))
                .into()
        }

        AuditStrategy::Checksum { expected_sha256 } => {
            column![
                text("Expected SHA-256").size(12),
                text_input("64-char hex digest", expected_sha256)
                    .on_input(Message::ChecksumChanged)
                    .padding(6)
                    .font(iced::Font::MONOSPACE),
            ]
            .spacing(4)
            .into()
        }

        AuditStrategy::LineMatch { rules } => {
            let mut col = column![
                text("Expected line changes").size(12),
            ]
            .spacing(6);

            for (i, rule) in rules.iter().enumerate() {
                let action_pick = pick_list(
                    &["Added", "Removed"][..],
                    Some(if rule.action == LineAction::Added { "Added" } else { "Removed" }),
                    move |s: &str| Message::LineRuleActionChanged(i, s.to_string()),
                )
                .padding(4);

                let line_input = text_input("line content", &rule.line)
                    .on_input(move |s| Message::LineRuleLineChanged(i, s))
                    .padding(6)
                    .font(iced::Font::MONOSPACE);

                let del_btn = button(text("✕").size(11))
                    .on_press(Message::RemoveLineRule(i))
                    .padding(4);

                col = col.push(
                    row![action_pick, line_input, del_btn]
                        .spacing(4)
                        .align_y(iced::Alignment::Center),
                );
            }

            col = col.push(
                button(text("+ Add rule").size(12))
                    .on_press(Message::AddLineRule)
                    .padding(Padding::from([4.0, 8.0])),
            );

            col.into()
        }

        AuditStrategy::Regex { pattern, target } => {
            let target_display: &'static str = match target {
                RegexTarget::AddedLines => "Added lines",
                RegexTarget::RemovedLines => "Removed lines",
                RegexTarget::AllChangedLines => "All changed lines",
            };
            column![
                text("Pattern").size(12),
                text_input("regular expression", pattern)
                    .on_input(Message::RegexPatternChanged)
                    .padding(6)
                    .font(iced::Font::MONOSPACE),
                text("Apply to").size(12),
                pick_list(
                    &["Added lines", "Removed lines", "All changed lines"][..],
                    Some(target_display),
                    |s: &str| Message::RegexTargetChanged(s.to_string()),
                )
                .padding(4),
            ]
            .spacing(4)
            .into()
        }

        AuditStrategy::Exact { expected_content } => {
            column![
                text("Expected full content").size(12),
                text_input("exact file content...", expected_content)
                    .on_input(Message::ExactContentChanged)
                    .padding(6)
                    .font(iced::Font::MONOSPACE),
            ]
            .spacing(4)
            .into()
        }
    }
}
