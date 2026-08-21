//! Inspector panel (Phase 3: ticket, approved_by, expires_at, template picker).

use iced::{
    Color, Element, Length, Padding,
    widget::{button, column, container, pick_list, row, scrollable, space, text, text_input},
};
use rust_i18n::t;

use aaai_core::{
    AuditStatus, FileAuditResult,
    config::definition::{AuditStrategy, LineAction, RegexTarget},
    templates::library as tmpl,
};
use crate::app::{App, InspectorState, Message};
use crate::theme;

const STRATEGIES: &[&str] = &["None", "Checksum", "LineMatch", "Regex", "Exact"];

pub fn view<'a>(app: &'a App, far: &'a FileAuditResult) -> Element<'a, Message> {
    let ins = &app.inspector;

    // ── Header ────────────────────────────────────────────────────────
    let status_color = match far.status {
        AuditStatus::Ok      => theme::OK_COLOR,
        AuditStatus::Pending => theme::PENDING_COLOR,
        AuditStatus::Failed  => theme::FAILED_COLOR,
        AuditStatus::Ignored => theme::IGNORED_COLOR,
        AuditStatus::Error   => theme::ERROR_COLOR,
    };
    let status_label = t!(match far.status {
        AuditStatus::Ok      => "status.ok",
        AuditStatus::Pending => "status.pending",
        AuditStatus::Failed  => "status.failed",
        AuditStatus::Ignored => "status.ignored",
        AuditStatus::Error   => "status.error",
    });
    let status_badge = container(text(status_label.to_string()).size(11).color(Color::WHITE))
        .padding(Padding::from([2.0, 8.0]))
        .style(move |_| iced::widget::container::Style {
            background: Some(iced::Background::Color(status_color)),
            border: iced::Border { radius: 4.0.into(), ..Default::default() },
            ..Default::default()
        });

    // Expiry badge
    let expiry_badge: Option<Element<'_, Message>> =
        far.entry.as_ref().and_then(|e| {
            if e.is_expired() {
                Some(colored_badge(t!("expiry.expired_badge").to_string(), iced::Color::from_rgb(0.80, 0.15, 0.15)))
            } else if e.expires_soon(30) {
                Some(colored_badge(t!("expiry.soon_badge").to_string(), iced::Color::from_rgb(0.85, 0.55, 0.05)))
            } else {
                None
            }
        });

    // ── Section: path + status ────────────────────────────────────────
    let mut badge_row = row![].spacing(4).align_y(iced::Alignment::Center);
    if let Some(b) = expiry_badge {
        badge_row = badge_row.push(b);
    }
    badge_row = badge_row.push(status_badge);
    let header_row = row![
        column![
            text(far.diff.path.clone()).size(12).font(iced::Font::MONOSPACE),
            text(far.diff.diff_type.to_string()).size(11).color(Color::from_rgb(0.5,0.5,0.5)),
        ].spacing(2),
        space().width(Length::Fill),
        badge_row,
    ]
    .align_y(iced::Alignment::Start)
    .spacing(4);

    let divider = iced::widget::rule::horizontal(1);

    // ── Section: reason ──────────────────────────────────────────────
    // RFC 009 + RFC 014: multi-line text_editor for the reason field
    let reason_label = semibold_text(
        format!("{} *", t!("inspector.reason_label")),  // * = required marker
        13.0,
    );
    let reason_input = iced::widget::text_editor(&ins.reason_content)
        .on_action(Message::ReasonAction)
        .height(Length::Fixed(72.0))
        .padding(Padding::from([8.0, 10.0]));

    // ── Section: Phase 3 traceability ────────────────────────────────
    let ticket_label = semibold_text(t!("inspector.ticket_label").to_string(), 12.0);
    let ticket_input = text_input(&t!("inspector.ticket_placeholder"), &ins.ticket)
        .on_input(Message::TicketChanged)
        .padding(6);

    let approved_by_label = semibold_text(t!("inspector.approved_by_label").to_string(), 12.0);
    let approved_by_input = text_input(&t!("inspector.approved_by_placeholder"), &ins.approved_by)
        .on_input(Message::ApprovedByChanged)
        .padding(6);

    let expires_label = semibold_text(t!("inspector.expires_at_label").to_string(), 12.0);
    let expires_input = text_input(&t!("inspector.expires_at_placeholder"), &ins.expires_at_str)
        .on_input(Message::ExpiresAtChanged)
        .padding(6);

    // ── Section: strategy ────────────────────────────────────────────
    let strategy_label = semibold_text(t!("inspector.strategy_label").to_string(), 13.0);
    let strategy_desc = text(ins.strategy.description().to_string()).size(11)
        .color(Color::from_rgb(0.45, 0.45, 0.45));
    let strategy_pick = pick_list(
        STRATEGIES,
        Some(ins.strategy_label.as_str()),
        |s: &str| Message::StrategySelected(s.to_string()),
    ).padding(6);

    // Template picker
    let tmpl_label = text(t!("inspector.template_label").to_string()).size(12);
    let mut tmpl_opts: Vec<String> = vec![t!("inspector.template_none").to_string()];
    tmpl_opts.extend(tmpl::TEMPLATES.iter().map(|t| t.id.to_string()));
    let tmpl_pick = pick_list(
        tmpl_opts,
        Some(t!("inspector.template_none").to_string()),
        |s: String| {
            if s == t!("inspector.template_none").to_string() {
                Message::ApplyTemplate(String::new())
            } else {
                Message::ApplyTemplate(s)
            }
        },
    ).padding(4).text_size(11);

    let strategy_form = build_strategy_form(ins);

    // ── Section: note ─────────────────────────────────────────────────
    let note_label = text(t!("inspector.note_label").to_string()).size(12);
    let note_input = text_input(&t!("inspector.note_placeholder"), &ins.note)
        .on_input(Message::NoteChanged)
        .padding(6);

    // ── Section: AuditWarnings ───────────────────────────────────────
    let warning_section: Option<Element<'a, Message>> = if !far.warnings.is_empty() {
        let warn_items: Vec<Element<'a, Message>> = far.warnings.iter().map(|w| {
            let (icon, color) = match w.kind() {
                "large-file"  => ("⚠", iced::Color::from_rgb(0.85, 0.55, 0.05)),
                "no-strategy" => ("ℹ", iced::Color::from_rgb(0.30, 0.55, 0.90)),
                "no-approver" => ("ℹ", iced::Color::from_rgb(0.55, 0.55, 0.60)),
                _             => ("⚠", iced::Color::from_rgb(0.85, 0.55, 0.05)),
            };
            row![
                text(icon).size(12).color(color),
                text(w.message()).size(11).color(color),
            ]
            .spacing(4)
            .align_y(iced::Alignment::Center)
            .into()
        }).collect();

        Some(
            container(column(warn_items).spacing(3))
                .width(Length::Fill)
                .padding(Padding::from([6.0, 8.0]))
                .style(|_| iced::widget::container::Style {
                    background: Some(iced::Background::Color(
                        iced::Color::from_rgba(0.95, 0.85, 0.20, 0.12)
                    )),
                    border: iced::Border {
                        color: iced::Color::from_rgba(0.85, 0.70, 0.10, 0.40),
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                })
                .into()
        )
    } else {
        None
    };

    // ── Column assembly ───────────────────────────────────────────────
    // ── Validation + approve button ───────────────────────────────────
    // RFC 002: collect all validation errors for display
    let all_errors: Vec<String> = {
        let mut errs = Vec::new();
        if let Some(e) = &ins.validation.reason_error { errs.push(e.clone()); }
        for fe in &ins.validation.strategy_errors { errs.push(fe.message.clone()); }
        if let Some(e) = &ins.validation.expires_at_error { errs.push(e.clone()); }
        errs
    };
    let val_err: Option<Element<'_, Message>> = if all_errors.is_empty() {
        None
    } else {
        let msg = all_errors.join(" · ");
        Some(text(msg).size(11).color(Color::from_rgb(0.78, 0.10, 0.10)).into())
    };
    // RFC 008: approve button moved to bottom action bar
    // can_approve is still computed for validation state reference (RFC 002)
    let _can_approve = ins.validation.can_approve();

    // Collect all column children into a Vec so lifetimes are uniform.
    let mut children: Vec<Element<'_, Message>> = vec![
        header_row.into(),
        divider.into(),
    ];

    // Warnings block (optional) immediately after the divider.
    if let Some(ws) = warning_section {
        children.push(ws);
    }

    children.extend([
        reason_label.into(), reason_input.into(),
        iced::widget::rule::horizontal(1).into(),
        ticket_label.into(), ticket_input.into(),
        approved_by_label.into(), approved_by_input.into(),
        expires_label.into(), expires_input.into(),
        iced::widget::rule::horizontal(1).into(),
        strategy_label.into(),
        row![strategy_pick, space().width(Length::Fill)].spacing(4).into(),
        strategy_desc.into(),
        tmpl_label.into(), tmpl_pick.into(),
        strategy_form,
        note_label.into(), note_input.into(),
    ]);

    if let Some(err) = val_err {
        children.push(err);
    }

    let col = column(children)
        .spacing(8)
        .padding(Padding::from([14.0, 14.0]));

    scrollable(container(col).width(Length::Fill))
        .width(Length::Fixed(300.0))
        .height(Length::Fill)
        .into()
}

fn semibold_text(s: String, size: f32) -> iced::widget::Text<'static> {
    text(s).size(size).font(iced::Font { weight: iced::font::Weight::Semibold, ..Default::default() })
}

fn colored_badge(label: String, color: Color) -> Element<'static, Message> {
    container(text(label).size(10).color(Color::WHITE))
        .padding(Padding::from([2.0, 6.0]))
        .style(move |_| iced::widget::container::Style {
            background: Some(iced::Background::Color(color)),
            border: iced::Border { radius: 4.0.into(), ..Default::default() },
            ..Default::default()
        })
        .into()
}

fn build_strategy_form<'a>(ins: &'a InspectorState) -> Element<'a, Message> {
    match &ins.strategy {
        AuditStrategy::None => {
            text("No content inspection.").size(12).color(Color::from_rgb(0.5,0.5,0.5)).into()
        }
        AuditStrategy::Checksum { expected_sha256 } => {
            column![
                text(t!("inspector.checksum_label").to_string()).size(12),
                text_input(&t!("inspector.checksum_placeholder"), expected_sha256)
                    .on_input(Message::ChecksumChanged).padding(6).font(iced::Font::MONOSPACE),
            ].spacing(4).into()
        }
        AuditStrategy::LineMatch { rules } => {
            let editing_rule = ins.editing_rule;
            let mut col = column![text(t!("inspector.linematch_label").to_string()).size(12)].spacing(5);

            for (i, rule) in rules.iter().enumerate() {
                let is_editing = editing_rule == Some(i);

                if is_editing {
                    // ── Edit form (expanded on click) ───────────────
                    let action_pick = pick_list(
                        &["Added", "Removed"][..],
                        Some(if rule.action == LineAction::Added { "Added" } else { "Removed" }),
                        move |s: &str| Message::LineRuleActionChanged(i, s.to_string()),
                    ).padding(4);
                    let line_input = text_input("line content", &rule.line)
                        .on_input(move |s| Message::LineRuleLineChanged(i, s))
                        .padding(6).font(iced::Font::MONOSPACE);
                    let del = button(text("✕").size(11))
                        .on_press(Message::RemoveLineRule(i)).padding(4);
                    let done = button(text("✓").size(11))
                        .on_press(Message::EditRule(i)).padding(4);
                    let edit_row = container(
                        column![
                            row![action_pick, done, del].spacing(4).align_y(iced::Alignment::Center),
                            line_input,
                        ].spacing(4)
                    )
                    .padding(Padding::from([6.0, 8.0]))
                    .width(Length::Fill)
                    .style(|_| iced::widget::container::Style {
                        border: iced::Border {
                            color: iced::Color::from_rgb(0.70, 0.75, 0.85),
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    });
                    col = col.push(edit_row);
                } else {
                    // ── Display block (RFC 012: colour coded) ────────
                    let (bg, label_color, action_label) = if rule.action == LineAction::Removed {
                        (
                            iced::Color::from_rgba(0.85, 0.20, 0.20, 0.10),
                            iced::Color::from_rgb(0.70, 0.10, 0.10),
                            "Removed",
                        )
                    } else {
                        (
                            iced::Color::from_rgba(0.10, 0.65, 0.30, 0.10),
                            iced::Color::from_rgb(0.08, 0.50, 0.20),
                            "Added",
                        )
                    };
                    let block = button(
                        container(
                            column![
                                text(format!("- action: {action_label}")).size(11)
                                    .font(iced::Font::MONOSPACE).color(label_color),
                                text(format!("  line: {:?}", rule.line)).size(11)
                                    .font(iced::Font::MONOSPACE)
                                    .color(iced::Color::from_rgb(0.20, 0.22, 0.28)),
                            ]
                            .spacing(1)
                        )
                        .padding(Padding::from([6.0, 10.0]))
                        .width(Length::Fill)
                        .style(move |_| iced::widget::container::Style {
                            background: Some(iced::Background::Color(bg)),
                            border: iced::Border { radius: 4.0.into(), ..Default::default() },
                            ..Default::default()
                        })
                    )
                    .on_press(Message::EditRule(i))
                    .style(iced::widget::button::text)
                    .width(Length::Fill);
                    col = col.push(block);
                }
            }

            col = col.push(
                button(text(t!("inspector.add_rule").to_string()).size(12))
                    .on_press(Message::AddLineRule)
                    .padding(Padding::from([4.0, 8.0]))
            );
            col.into()
        }
        AuditStrategy::Regex { pattern, target } => {
            let target_opts: &[&str] = &["Added lines", "Removed lines", "All changed lines"];
            let target_sel = match target {
                RegexTarget::AddedLines     => "Added lines",
                RegexTarget::RemovedLines   => "Removed lines",
                RegexTarget::AllChangedLines => "All changed lines",
            };
            column![
                text(t!("inspector.regex_pattern_label").to_string()).size(12),
                text_input("regular expression", pattern)
                    .on_input(Message::RegexPatternChanged).padding(6).font(iced::Font::MONOSPACE),
                text(t!("inspector.regex_target_label").to_string()).size(12),
                pick_list(target_opts, Some(target_sel),
                    |s: &str| Message::RegexTargetChanged(s.to_string())).padding(4),
            ].spacing(4).into()
        }
        AuditStrategy::Exact { expected_content } => {
            column![
                text(t!("inspector.exact_label").to_string()).size(12),
                text_input("exact file content...", expected_content)
                    .on_input(Message::ExactContentChanged).padding(6).font(iced::Font::MONOSPACE),
            ].spacing(4).into()
        }
    }
}
