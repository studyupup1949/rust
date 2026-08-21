//! Inspector panel (Phase 3: ticket, approved_by, expires_at, template picker).

use iced::{
    Color, Element, Length, Padding,
    widget::{button, column, container, pick_list, row, scrollable, space, text, text_input},
};
use rust_i18n::t;

use aaai::{
    AuditStatus, FileAuditResult, DiffType,
    config::definition::{AuditStrategy, LineAction, RegexTarget},
    templates::library as tmpl,
};
use crate::app::{App, InspectorState, Message};
use crate::util::{LocalizedOption, StrategyKind};

pub fn view<'a>(app: &'a App, far: &'a FileAuditResult) -> Element<'a, Message> {
    let ins = &app.inspector;

    // ── Header ────────────────────────────────────────────────────────
    let status_color = match far.status {
        s => crate::theme::status_color(s, &app.design_tokens, app.theme.is_high_contrast()),
    };
    let status_label = t!(match far.status {
        AuditStatus::Ok      => "status.ok",
        AuditStatus::Pending => "status.pending",
        AuditStatus::Failed  => "status.failed",
        AuditStatus::Ignored => "status.ignored",
        AuditStatus::Error   => "status.error",
    });
    let status_badge = container(
        text(status_label.to_string())
            .size(app.design_tokens.typography.label.size)
            .line_height(app.design_tokens.typography.label.line_height)
            .color(Color::WHITE),
    )
        .padding(Padding::from([app.design_tokens.spacing.xs, app.design_tokens.spacing.sm]))
        .style(move |_| iced::widget::container::Style {
            background: Some(iced::Background::Color(status_color)),
            border: iced::Border { radius: 4.0.into(), ..Default::default() },
            ..Default::default()
        });

    // Expiry badge
    let expiry_badge: Option<Element<'_, Message>> =
        far.entry.as_ref().and_then(|e| {
            if e.is_expired() {
                Some(colored_badge(t!("expiry.expired_badge").to_string(), crate::style::to_iced(app.design_tokens.palette.danger), &app.design_tokens))
            } else if e.expires_soon(30) {
                Some(colored_badge(t!("expiry.soon_badge").to_string(), crate::style::to_iced(app.design_tokens.palette.warning), &app.design_tokens))
            } else {
                None
            }
        });

    // ── Section: path + status ────────────────────────────────────────
    let mut badge_row = row![].spacing(app.design_tokens.spacing.xs).align_y(iced::Alignment::Center);
    if let Some(b) = expiry_badge {
        badge_row = badge_row.push(b);
    }
    badge_row = badge_row.push(status_badge);
    let header_row = row![
        column![
            text(far.diff.path.clone())
                .size(app.design_tokens.typography.body_small.size)
                .line_height(app.design_tokens.typography.body_small.line_height)
                .font(iced::Font::MONOSPACE),
            text(far.diff.diff_type.to_string())
                .size(app.design_tokens.typography.body_small.size)
                .line_height(app.design_tokens.typography.body_small.line_height)
                .color(crate::style::to_iced(app.design_tokens.palette.text_secondary)),
        ].spacing(app.design_tokens.spacing.xs),
        space().width(Length::Fill),
        badge_row,
    ]
    .align_y(iced::Alignment::Start)
    .spacing(app.design_tokens.spacing.xs);

    let divider = iced::widget::rule::horizontal(1);

    // ── Section: reason ──────────────────────────────────────────────
    // RFC 009 + RFC 014: multi-line text_editor for the reason field
    let reason_label = semibold_text(
        format!("{} *", t!("inspector.reason_label")),  // * = required marker
        &app.design_tokens,
    );
    let reason_input = iced::widget::text_editor(&ins.reason_content)
        .placeholder(t!("inspector.reason_placeholder").to_string())
        .on_action(Message::ReasonAction)
        .height(Length::Fixed(72.0))
        .padding(Padding::from([app.design_tokens.spacing.sm, app.design_tokens.spacing.md]));

    // RFC 074 — diff-type-aware example line, shown only while the reason is
    // empty. Teaches a newcomer the shape of a good justification without
    // adding clutter for experts (it disappears the moment they type).
    let reason_example: Option<Element<'_, Message>> = if ins.reason.trim().is_empty() {
        let key = match far.diff.diff_type {
            DiffType::Added       => "inspector.reason_example_added",
            DiffType::Removed     => "inspector.reason_example_removed",
            DiffType::Modified    => "inspector.reason_example_modified",
            _                     => "inspector.reason_example_generic",
        };
        Some(
            text(t!(key).to_string())
                .size(app.design_tokens.typography.body_small.size)
                .line_height(app.design_tokens.typography.body_small.line_height)
                .color(crate::style::to_iced(app.design_tokens.palette.text_muted))
                .into()
        )
    } else {
        None
    };

    // ── RFC 054: glob-pattern toggle ─────────────────────────────────
    let pattern_arrow = if ins.use_pattern { "▾" } else { "▸" };
    let pattern_toggle = button(
        text(format!("{} {}", pattern_arrow, t!("inspector.use_pattern")))
            .size(app.design_tokens.typography.label.size)
            .line_height(app.design_tokens.typography.label.line_height)
            .color(crate::style::to_iced(app.design_tokens.palette.text_secondary))
    )
    .on_press(Message::ToggleUsePattern)
    .style({ let t = app.design_tokens.clone(); move |_th, s| crate::style::btn_ghost(&t, s) })
    .padding(iced::Padding::from([app.design_tokens.spacing.xs, 0.0]));

    let pattern_section: Option<iced::Element<'_, Message>> = if ins.use_pattern {
        let pattern_input = text_input(
            &t!("inspector.pattern_placeholder"),
            &ins.pattern_path,
        )
        .on_input(Message::PatternChanged)
        .padding(Padding::from([app.design_tokens.spacing.sm, app.design_tokens.spacing.sm]))
        .size(app.design_tokens.typography.body_small.size)
        .line_height(app.design_tokens.typography.body_small.line_height);

        let pat_status: iced::Element<'_, Message> = match &ins.validation.pattern_error {
            Some(err) => text(format!("✗ {}", err))
                .size(app.design_tokens.typography.body_small.size)
                .line_height(app.design_tokens.typography.body_small.line_height)
                .color(crate::style::to_iced(app.design_tokens.palette.danger))
                .into(),
            None if !ins.pattern_path.trim().is_empty() =>
                text("✓")
                    .size(app.design_tokens.typography.body_small.size)
                    .line_height(app.design_tokens.typography.body_small.line_height)
                    .color(crate::style::to_iced(app.design_tokens.palette.success))
                    .into(),
            _ => iced::widget::space().height(0).into(),
        };

        // RFC 055 — suggestion chips
        let suggestions_row: Option<iced::Element<'_, Message>> =
            if !ins.pattern_suggestions.is_empty() {
                let label = text(t!("inspector.pattern_suggestions").to_string())
                    .size(app.design_tokens.typography.body_small.size)
                    .line_height(app.design_tokens.typography.body_small.line_height)
                    .color(crate::style::to_iced(app.design_tokens.palette.text_muted));
                let chips: Vec<iced::Element<'_, Message>> = ins.pattern_suggestions
                    .iter()
                    .map(|s| {
                        let s2 = s.clone();
                        button(
                            text(s)
                                .size(app.design_tokens.typography.label.size)
                                .line_height(app.design_tokens.typography.label.line_height)
                                .font(iced::Font::MONOSPACE),
                        )
                            .on_press(Message::ApplyPatternSuggestion(s2))
                            .padding(iced::Padding::from([app.design_tokens.spacing.xs, app.design_tokens.spacing.sm]))
                            .style({ let t = app.design_tokens.clone(); move |_th, s| crate::style::btn_secondary(&t, s) })
                            .into()
                    })
                    .collect();
                let chip_row = iced::widget::row(chips).spacing(app.design_tokens.spacing.xs);
                Some(column![label, chip_row].spacing(app.design_tokens.spacing.xs).into())
            } else {
                None
            };

        Some(column(
            [
                Some(semibold_text(t!("inspector.pattern_label").to_string(), &app.design_tokens).into()),
                Some(pattern_input.into()),
                Some(pat_status),
                suggestions_row,
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
        ).spacing(app.design_tokens.spacing.xs).into())
    } else {
        None
    };

    // ── Section: Phase 3 traceability ────────────────────────────────
    let ticket_label = semibold_text(t!("inspector.ticket_label").to_string(), &app.design_tokens);
    let ticket_input = text_input(&t!("inspector.ticket_placeholder"), &ins.ticket)
        .on_input(Message::TicketChanged)
        .padding(app.design_tokens.spacing.sm);

    let approved_by_label = semibold_text(t!("inspector.approved_by_label").to_string(), &app.design_tokens);
    let approved_by_input = text_input(&t!("inspector.approved_by_placeholder"), &ins.approved_by)
        .on_input(Message::ApprovedByChanged)
        .padding(app.design_tokens.spacing.sm);

    let expires_label = semibold_text(t!("inspector.expires_at_label").to_string(), &app.design_tokens);
    let expires_input = text_input(&t!("inspector.expires_at_placeholder"), &ins.expires_at_str)
        .on_input(Message::ExpiresAtChanged)
        .padding(app.design_tokens.spacing.sm);

    // ── Section: strategy ────────────────────────────────────────────
    let strategy_label = semibold_text(t!("inspector.strategy_label").to_string(), &app.design_tokens);
    let strategy_desc = text(ins.strategy.description().to_string())
        .size(app.design_tokens.typography.body_small.size)
        .line_height(app.design_tokens.typography.body_small.line_height)
        .color(crate::style::to_iced(app.design_tokens.palette.text_secondary));
    // RFC 035 — LocalizedOption<StrategyKind> pattern
    // RFC 075 — mark the recommended option with "(recommended)" for newcomers.
    let recommended_kind = crate::app::App::recommended_strategy(far.diff.diff_type);
    let rec_label = format!(" ({})", t!("inspector.strategy_recommended"));
    let strategy_options: Vec<LocalizedOption<StrategyKind>> = [
        StrategyKind::None,
        StrategyKind::Checksum,
        StrategyKind::LineMatch,
        StrategyKind::Regex,
        StrategyKind::Exact,
    ]
    .into_iter()
    .map(|k| {
        let base = k.label();
        let label = if k == recommended_kind {
            format!("{}{}", base, rec_label).into()
        } else {
            base
        };
        LocalizedOption { value: k, label }
    })
    .collect();
    let strategy_selected = strategy_options.iter()
        .find(|o| o.value == ins.strategy_kind)
        .cloned();
    let strategy_pick = pick_list(
        strategy_options,
        strategy_selected,
        |o: LocalizedOption<StrategyKind>| Message::StrategySelected(o.value),
    ).padding(app.design_tokens.spacing.sm);

    // Template picker
    let tmpl_label = text(t!("inspector.template_label").to_string())
        .size(app.design_tokens.typography.label.size)
        .line_height(app.design_tokens.typography.label.line_height);
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
    ).padding(app.design_tokens.spacing.xs).text_size(11);

    let strategy_form = build_strategy_form(ins, &app.design_tokens);

    // ── Section: note ─────────────────────────────────────────────────
    let note_label = text(t!("inspector.note_label").to_string())
        .size(app.design_tokens.typography.label.size)
        .line_height(app.design_tokens.typography.label.line_height);
    let note_input = text_input(&t!("inspector.note_placeholder"), &ins.note)
        .on_input(Message::NoteChanged)
        .padding(app.design_tokens.spacing.sm);

    // ── Section: AuditWarnings ───────────────────────────────────────
    let warning_section: Option<Element<'a, Message>> = if !far.warnings.is_empty() {
        let warn_items: Vec<Element<'a, Message>> = far.warnings.iter().map(|w| {
            let (icon, color) = match w.kind() {
                "large-file"  => ("⚠", crate::style::to_iced(app.design_tokens.palette.warning)),
                "no-strategy" => ("ℹ", crate::style::to_iced(app.design_tokens.palette.info)),
                "no-approver" => ("ℹ", crate::style::to_iced(app.design_tokens.palette.info)),
                _             => ("⚠", crate::style::to_iced(app.design_tokens.palette.warning)),
            };
            row![
                text(icon)
                    .size(app.design_tokens.typography.label.size)
                    .line_height(app.design_tokens.typography.label.line_height)
                    .color(color),
                text(w.message())
                    .size(app.design_tokens.typography.body_small.size)
                    .line_height(app.design_tokens.typography.body_small.line_height)
                    .color(color),
            ]
            .spacing(app.design_tokens.spacing.xs)
            .align_y(iced::Alignment::Center)
            .into()
        }).collect();

        Some(
            container(column(warn_items).spacing(app.design_tokens.spacing.xs))
                .width(Length::Fill)
                .padding(Padding::from([app.design_tokens.spacing.sm, app.design_tokens.spacing.sm]))
                .style({
                    let warning = crate::style::to_iced(app.design_tokens.palette.warning);
                    move |_| iced::widget::container::Style {
                        background: Some(iced::Background::Color(
                            Color { a: 0.12, ..warning }
                        )),
                        border: iced::Border {
                            color: Color { a: 0.40, ..warning },
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    }
                })
                .into()
        )
    } else {
        None
    };

    // ── Column assembly ───────────────────────────────────────────────
    // ── Validation + approve button ───────────────────────────────────
    // RFC 002: collect all validation errors for display.
    // RFC 028: FieldError now carries an optional `hint`; when present,
    // it renders beneath the message in a muted style. Errors from
    // reason_error and expires_at_error don't carry hints yet (their
    // own i18n migration is a follow-up).
    let val_err: Option<Element<'_, Message>> = {
        let mut blocks: Vec<Element<'_, Message>> = Vec::new();
        let err_color = crate::style::to_iced(app.design_tokens.palette.danger);
        // Muted but readable — matches RFC 020's banner hint style and
        // RFC 026's toast hint.
        let hint_color = crate::style::to_iced(app.design_tokens.palette.text_muted);

        if let Some(e) = &ins.validation.reason_error {
            blocks.push(
                text(e.clone())
                    .size(app.design_tokens.typography.body_small.size)
                    .line_height(app.design_tokens.typography.body_small.line_height)
                    .color(err_color)
                    .into(),
            );
        }
        for fe in &ins.validation.strategy_errors {
            blocks.push(
                text(fe.message.clone())
                    .size(app.design_tokens.typography.body_small.size)
                    .line_height(app.design_tokens.typography.body_small.line_height)
                    .color(err_color)
                    .into(),
            );
            if let Some(h) = &fe.hint {
                blocks.push(
                    text(h.clone())
                        .size(app.design_tokens.typography.body_small.size)
                        .line_height(app.design_tokens.typography.body_small.line_height)
                        .color(hint_color)
                        .into(),
                );
            }
        }
        if let Some(e) = &ins.validation.expires_at_error {
            blocks.push(
                text(e.clone())
                    .size(app.design_tokens.typography.body_small.size)
                    .line_height(app.design_tokens.typography.body_small.line_height)
                    .color(err_color)
                    .into(),
            );
        }
        // pattern_error is shown inline in the pattern section when use_pattern is true;
        // no duplicate display needed here.

        if blocks.is_empty() {
            None
        } else {
            Some(iced::widget::column(blocks).spacing(app.design_tokens.spacing.xs).into())
        }
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
    ]);
    if let Some(ex) = reason_example {
        children.push(ex);
    }
    children.extend([
        iced::widget::rule::horizontal(1).into(),
        pattern_toggle.into(),
    ]);
    if let Some(ps) = pattern_section {
        children.push(ps);
    }
    children.extend([
        iced::widget::rule::horizontal(1).into(),
        strategy_label.into(),
        row![strategy_pick, space().width(Length::Fill)].spacing(app.design_tokens.spacing.xs).into(),
        strategy_desc.into(),
        strategy_form,
    ]);

    // RFC 048 — progressive disclosure: expert fields behind a toggle.
    // RFC 049 — if an advanced field has a validation error, force the
    // section open so the user can see the field and fix it.
    let effective_advanced_expanded = app.advanced_inspector_expanded
        || ins.validation.expires_at_error.is_some();
    let toggle_label = if effective_advanced_expanded {
        t!("inspector.advanced_toggle_hide").to_string()
    } else {
        t!("inspector.advanced_toggle_show").to_string()
    };
    let toggle_btn = button(
        text(format!("{} {}", if effective_advanced_expanded { "▾" } else { "▸" }, toggle_label))
            .size(app.design_tokens.typography.label.size)
            .line_height(app.design_tokens.typography.label.line_height)
            .color(crate::style::to_iced(app.design_tokens.palette.text_secondary))
    )
    .on_press(Message::ToggleAdvancedInspector)
    .style({ let t = app.design_tokens.clone(); move |_th, s| crate::style::btn_ghost(&t, s) })
    .padding(iced::Padding::from([app.design_tokens.spacing.xs, 0.0]));

    children.push(toggle_btn.into());

    if effective_advanced_expanded {
        children.extend([
            iced::widget::rule::horizontal(1).into(),
            ticket_label.into(), ticket_input.into(),
            approved_by_label.into(), approved_by_input.into(),
            expires_label.into(), expires_input.into(),
            iced::widget::rule::horizontal(1).into(),
            tmpl_label.into(), tmpl_pick.into(),
            note_label.into(), note_input.into(),
        ]);
    }

    if let Some(err) = val_err {
        children.push(err);
    }

    // RFC 039 — Revert to Pending button (only when entry is OK).
    // The approve button is in the bottom action bar (RFC 008); the revert
    // button lives here in the inspector so it's contextually near the
    // approval details, not mixed with the main save/rerun actions.
    if far.status == AuditStatus::Ok {
        let revert_btn = button(
            text(t!("inspector.revert_to_pending").to_string())
                .size(app.design_tokens.typography.label.size)
                .line_height(app.design_tokens.typography.label.line_height)
        )
        .on_press(Message::RevertSelectedEntry)
        .padding(iced::Padding::from([app.design_tokens.spacing.sm, app.design_tokens.spacing.md]))
        .style({ let t = app.design_tokens.clone(); move |_th, s| crate::style::btn_secondary(&t, s) });

        children.push(
            container(revert_btn)
                .padding(iced::Padding::from([app.design_tokens.spacing.xs, 0.0]))
                .into(),
        );
    }

    let col = column(children)
        .spacing(app.design_tokens.spacing.sm)
        .padding(Padding::from([app.design_tokens.spacing.lg, app.design_tokens.spacing.lg]));

    scrollable(container(col).width(Length::Fill))
        .width(Length::Fixed(300.0))
        .height(Length::Fill)
        .into()
}

fn semibold_text(s: String, tokens: &snora::design::Tokens) -> iced::widget::Text<'static> {
    text(s)
        .size(tokens.typography.label.size)
        .line_height(tokens.typography.label.line_height)
        .font(iced::Font { weight: iced::font::Weight::Semibold, ..Default::default() })
}

fn colored_badge(label: String, color: Color, tokens: &snora::design::Tokens) -> Element<'static, Message> {
    container(
        text(label)
            .size(tokens.typography.label.size)
            .line_height(tokens.typography.label.line_height)
            .color(Color::WHITE),
    )
        .padding(Padding::from([tokens.spacing.xs, tokens.spacing.sm]))
        .style(move |_| iced::widget::container::Style {
            background: Some(iced::Background::Color(color)),
            border: iced::Border { radius: 4.0.into(), ..Default::default() },
            ..Default::default()
        })
        .into()
}

fn build_strategy_form<'a>(ins: &'a InspectorState, tokens: &'a snora::design::Tokens) -> Element<'a, Message> {
    match &ins.strategy {
        AuditStrategy::None => {
            text(t!("inspector.no_content_inspection").to_string())
                .size(tokens.typography.body_small.size)
                .line_height(tokens.typography.body_small.line_height)
                .color(crate::style::to_iced(tokens.palette.text_secondary)).into()
        }
        AuditStrategy::Checksum { expected_sha256 } => {
            column![
                text(t!("inspector.checksum_label").to_string())
                    .size(tokens.typography.label.size)
                    .line_height(tokens.typography.label.line_height),
                text_input(&t!("inspector.checksum_placeholder"), expected_sha256)
                    .on_input(Message::ChecksumChanged).padding(tokens.spacing.sm).font(iced::Font::MONOSPACE),
                // RFC 080 — show how to obtain the hash value so the user
                // doesn't have to look it up. Greyed, small — expert users
                // will already know; newcomers get the command they need.
                text(t!("inspector.checksum_how_to").to_string())
                    .size(tokens.typography.body_small.size)
                    .line_height(tokens.typography.body_small.line_height)
                    .color(crate::style::to_iced(tokens.palette.text_muted)),
            ].spacing(tokens.spacing.xs).into()
        }
        AuditStrategy::LineMatch { rules } => {
            let editing_rule = ins.editing_rule;
            let mut col = column![
                text(t!("inspector.linematch_label").to_string())
                    .size(tokens.typography.label.size)
                    .line_height(tokens.typography.label.line_height)
            ].spacing(tokens.spacing.sm);

            for (i, rule) in rules.iter().enumerate() {
                let is_editing = editing_rule == Some(i);

                if is_editing {
                    // ── Edit form (expanded on click) ───────────────
                    // RFC 033 — pick_list display/value separation: localized
                    // labels paired with `LineAction` enum values.
                    let action_options: Vec<crate::util::LocalizedOption<LineAction>> = vec![
                        crate::util::LocalizedOption {
                            value: LineAction::Added,
                            label: t!("inspector.action_added").to_string(),
                        },
                        crate::util::LocalizedOption {
                            value: LineAction::Removed,
                            label: t!("inspector.action_removed").to_string(),
                        },
                    ];
                    let action_selected = action_options.iter()
                        .find(|o| o.value == rule.action)
                        .cloned();
                    let action_pick = pick_list(
                        action_options,
                        action_selected,
                        move |o: crate::util::LocalizedOption<LineAction>|
                            Message::LineRuleActionChanged(i, o.value),
                    ).padding(tokens.spacing.xs);
                    let line_input = text_input(&t!("inspector.linematch_line_placeholder").to_string(), &rule.line)
                        .on_input(move |s| Message::LineRuleLineChanged(i, s))
                        .padding(tokens.spacing.sm).font(iced::Font::MONOSPACE);
                    let del = button(
                        text("✕")
                            .size(tokens.typography.label.size)
                            .line_height(tokens.typography.label.line_height),
                    )
                        .on_press(Message::RemoveLineRule(i)).padding(tokens.spacing.xs);
                    let done = button(
                        text("✓")
                            .size(tokens.typography.label.size)
                            .line_height(tokens.typography.label.line_height),
                    )
                        .on_press(Message::EditRule(i)).padding(tokens.spacing.xs);
                    let edit_row = container(
                        column![
                            row![action_pick, done, del].spacing(tokens.spacing.xs).align_y(iced::Alignment::Center),
                            line_input,
                        ].spacing(tokens.spacing.xs)
                    )
                    .padding(Padding::from([tokens.spacing.sm, tokens.spacing.sm]))
                    .width(Length::Fill)
                    .style({
                        let accent = crate::style::to_iced(tokens.palette.accent);
                        move |_| iced::widget::container::Style {
                            border: iced::Border {
                                color: accent,
                                width: 1.0,
                                radius: 4.0.into(),
                            },
                            ..Default::default()
                        }
                    });
                    col = col.push(edit_row);
                } else {
                    // ── Display block (RFC 012: colour coded) ────────
                    let (bg, label_color, action_label) = if rule.action == LineAction::Removed {
                        let danger = crate::style::to_iced(tokens.palette.danger);
                        (
                            Color { a: 0.10, ..danger },
                            danger,
                            "Removed",
                        )
                    } else {
                        let success = crate::style::to_iced(tokens.palette.success);
                        (
                            Color { a: 0.10, ..success },
                            success,
                            "Added",
                        )
                    };
                    let block = button(
                        container(
                            column![
                                text(format!("- action: {action_label}"))
                                    .size(tokens.typography.body_small.size)
                                    .line_height(tokens.typography.body_small.line_height)
                                    .font(iced::Font::MONOSPACE).color(label_color),
                                text(format!("  line: {:?}", rule.line))
                                    .size(tokens.typography.body_small.size)
                                    .line_height(tokens.typography.body_small.line_height)
                                    .font(iced::Font::MONOSPACE)
                                    .color(crate::style::to_iced(tokens.palette.text_secondary)),
                            ]
                            .spacing(tokens.spacing.xs)
                        )
                        .padding(Padding::from([tokens.spacing.sm, tokens.spacing.md]))
                        .width(Length::Fill)
                        .style(move |_| iced::widget::container::Style {
                            background: Some(iced::Background::Color(bg)),
                            border: iced::Border { radius: 4.0.into(), ..Default::default() },
                            ..Default::default()
                        })
                    )
                    .on_press(Message::EditRule(i))
                    .style({ let t = tokens.clone(); move |_th, s| crate::style::btn_ghost(&t, s) })
                    .width(Length::Fill);
                    col = col.push(block);
                }
            }

            col = col.push(
                button(
                    text(t!("inspector.add_rule").to_string())
                        .size(tokens.typography.label.size)
                        .line_height(tokens.typography.label.line_height),
                )
                    .on_press(Message::AddLineRule)
                    .padding(Padding::from([tokens.spacing.xs, tokens.spacing.sm]))
            );
            col.into()
        }
        AuditStrategy::Regex { pattern, target } => {
            // RFC 033 — pick_list display/value separation: localized labels
            // paired with `RegexTarget` enum values.
            let target_options: Vec<crate::util::LocalizedOption<RegexTarget>> = vec![
                crate::util::LocalizedOption {
                    value: RegexTarget::AddedLines,
                    label: t!("inspector.target_added_lines").to_string(),
                },
                crate::util::LocalizedOption {
                    value: RegexTarget::RemovedLines,
                    label: t!("inspector.target_removed_lines").to_string(),
                },
                crate::util::LocalizedOption {
                    value: RegexTarget::AllChangedLines,
                    label: t!("inspector.target_all_changed_lines").to_string(),
                },
            ];
            let target_selected = target_options.iter()
                .find(|o| &o.value == target)
                .cloned();
            column![
                text(t!("inspector.regex_pattern_label").to_string())
                    .size(tokens.typography.label.size)
                    .line_height(tokens.typography.label.line_height),
                text_input(&t!("inspector.regex_pattern_placeholder").to_string(), pattern)
                    .on_input(Message::RegexPatternChanged).padding(tokens.spacing.sm).font(iced::Font::MONOSPACE),
                text(t!("inspector.regex_target_label").to_string())
                    .size(tokens.typography.label.size)
                    .line_height(tokens.typography.label.line_height),
                pick_list(target_options, target_selected,
                    |o: crate::util::LocalizedOption<RegexTarget>|
                        Message::RegexTargetChanged(o.value)).padding(tokens.spacing.xs),
            ].spacing(tokens.spacing.xs).into()
        }
        AuditStrategy::Exact { expected_content } => {
            column![
                text(t!("inspector.exact_label").to_string())
                    .size(tokens.typography.label.size)
                    .line_height(tokens.typography.label.line_height),
                text_input(&t!("inspector.exact_content_placeholder").to_string(), expected_content)
                    .on_input(Message::ExactContentChanged).padding(tokens.spacing.sm).font(iced::Font::MONOSPACE),
            ].spacing(tokens.spacing.xs).into()
        }
    }
}
