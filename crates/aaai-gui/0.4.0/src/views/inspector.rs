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
    let reason_label = semibold_text(t!("inspector.reason_label").to_string(), 13.0);
    let reason_input = text_input(&t!("inspector.reason_placeholder"), &ins.reason)
        .on_input(Message::ReasonChanged)
        .padding(8);

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

    // ── Validation + approve button ───────────────────────────────────
    let val_err: Option<Element<'_, Message>> = ins.validation_error.as_ref().map(|e| {
        text(e.clone()).size(12).color(Color::from_rgb(0.78, 0.10, 0.10)).into()
    });
    let can_approve = !ins.reason.trim().is_empty() && ins.validation_error.is_none();
    let approve_btn = button(
        text(t!("inspector.approve_button").to_string()).size(14)
            .font(iced::Font { weight: iced::font::Weight::Semibold, ..Default::default() }),
    )
    .on_press_maybe(if can_approve { Some(Message::ApproveEntry) } else { None })
    .padding(Padding::from([9.0, 20.0]));

    let mut col = column![
        header_row,
        divider,
        reason_label, reason_input,
        iced::widget::rule::horizontal(1),
        ticket_label, ticket_input,
        approved_by_label, approved_by_input,
        expires_label, expires_input,
        iced::widget::rule::horizontal(1),
        strategy_label,
        row![strategy_pick, space().width(Length::Fill)].spacing(4),
        strategy_desc,
        tmpl_label, tmpl_pick,
        strategy_form,
        note_label, note_input,
        space().height(Length::Fixed(4.0)),
        approve_btn,
    ]
    .spacing(8)
    .padding(Padding::from([14.0, 14.0]));

    if let Some(err) = val_err {
        col = col.push(err);
    }

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
            let mut col = column![text(t!("inspector.linematch_label").to_string()).size(12)].spacing(6);
            for (i, rule) in rules.iter().enumerate() {
                let action_pick = pick_list(
                    &["Added", "Removed"][..],
                    Some(if rule.action == LineAction::Added { "Added" } else { "Removed" }),
                    move |s: &str| Message::LineRuleActionChanged(i, s.to_string()),
                ).padding(4);
                let line_input = text_input("line content", &rule.line)
                    .on_input(move |s| Message::LineRuleLineChanged(i, s))
                    .padding(6).font(iced::Font::MONOSPACE);
                let del = button(text("✕").size(11)).on_press(Message::RemoveLineRule(i)).padding(4);
                col = col.push(row![action_pick, line_input, del].spacing(4).align_y(iced::Alignment::Center));
            }
            col = col.push(
                button(text(t!("inspector.add_rule").to_string()).size(12))
                    .on_press(Message::AddLineRule).padding(Padding::from([4.0, 8.0]))
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
