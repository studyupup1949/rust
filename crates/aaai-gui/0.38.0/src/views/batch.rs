//! Batch-approval sheet (Phase 2).
//!
//! Shown as a snora `Sheet` anchored to the end (right) edge.
//! The user selects multiple Pending entries in the file tree,
//! types a shared reason, and approves them all at once.

use iced::{
    Color, Element, Length, Padding,
    widget::{button, column, container, pick_list, row,
             scrollable, space, text, text_input},
};

use aaai_core::AuditStatus;
use crate::app::{App, Message};
use crate::util::{LocalizedOption, StrategyKind};
use rust_i18n::t;

pub fn view<'a>(app: &'a App) -> Element<'a, Message> {
    let count = app.batch.selected.len();

    let title = text(t!("batch.title")).size(17).font(iced::Font {
        weight: iced::font::Weight::Bold,
        ..Default::default()
    });

    let selected_info = text(format!("{} {}", count, t!("batch.selected"))).size(13);

    // Selected path list (read-only preview)
    let path_list: Vec<Element<'_, Message>> = if let Some(result) = &app.audit_result {
        app.batch.selected.iter().copied()
            .filter_map(|i| result.results.get(i))
            .map(|far| {
                let status_color = match far.status {
                    AuditStatus::Pending => crate::theme::status_color(AuditStatus::Pending, &app.design_tokens, app.theme.is_high_contrast()),
                    AuditStatus::Failed  => crate::theme::status_color(AuditStatus::Failed, &app.design_tokens, app.theme.is_high_contrast()),
                    _                    => crate::theme::status_color(AuditStatus::Ignored, &app.design_tokens, app.theme.is_high_contrast()),
                };
                let dot = container(text("●").size(10).color(status_color))
                    .padding(Padding::from([0.0, 4.0]));
                row![
                    dot,
                    text(far.diff.path.as_str()).size(11).font(iced::Font::MONOSPACE),
                ]
                .spacing(4)
                .align_y(iced::Alignment::Center)
                .into()
            })
            .collect()
    } else {
        Vec::new()
    };

    let path_scroll = scrollable(
        container(column(path_list).spacing(2))
            .width(Length::Fill)
            .padding(Padding::from([4.0, 8.0])),
    )
    .height(Length::Fixed(160.0));

    // Shared reason
    let reason_label = text(t!("batch.reason_label")).size(13).font(iced::Font {
        weight: iced::font::Weight::Semibold,
        ..Default::default()
    });
    let reason_input = text_input(
        &t!("batch.reason_placeholder"),
        &app.batch.shared_reason,
    )
    .on_input(Message::BatchReasonChanged)
    .padding(8);

    // Strategy picker — RFC 035: LocalizedOption<StrategyKind> pattern
    let strategy_options: Vec<LocalizedOption<StrategyKind>> = [
        StrategyKind::None,
        StrategyKind::Checksum,
        StrategyKind::LineMatch,
        StrategyKind::Regex,
        StrategyKind::Exact,
    ]
    .into_iter()
    .map(|k| LocalizedOption { value: k, label: k.label() })
    .collect();
    let strategy_current_kind = StrategyKind::from_strategy(&app.batch.shared_strategy);
    let strategy_selected = strategy_options.iter()
        .find(|o| o.value == strategy_current_kind)
        .cloned();
    let strategy_pick = pick_list(
        strategy_options,
        strategy_selected,
        |o: LocalizedOption<StrategyKind>| Message::BatchStrategySelected(o.value),
    )
    .padding(6);

    // Validation error
    let val_err: Option<Element<'_, Message>> =
        app.batch.validation_error.as_ref().map(|e| {
            text(e.as_str()).size(12)
                .color(Color::from_rgb(0.78, 0.10, 0.10))
                .into()
        });

    let can_approve = count > 0 && !app.batch.shared_reason.trim().is_empty();

    let approve_btn = button(
        text(t!("batch.approve_button")).size(14).font(iced::Font {
            weight: iced::font::Weight::Semibold,
            ..Default::default()
        }),
    )
    .on_press_maybe(if can_approve { Some(Message::CommitBatchApprove) } else { None })
    .padding(Padding::from([9.0, 20.0]));

    let cancel_btn = button(text(t!("batch.cancel")).size(13))
        .on_press(Message::CloseBatchSheet)
        .padding(Padding::from([8.0, 16.0]));

    let mut col = column![
        title,
        selected_info,
        iced::widget::rule::horizontal(1),
        path_scroll,
        iced::widget::rule::horizontal(1),
        reason_label,
        reason_input,
        text(t!("batch.content_audit_strategy").to_string()).size(12),
        strategy_pick,
        space().height(Length::Fixed(8.0)),
        row![approve_btn, cancel_btn].spacing(8),
    ]
    .spacing(10)
    .padding(Padding::from([20.0, 20.0]));

    if let Some(err) = val_err {
        col = col.push(err);
    }

    container(col)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
