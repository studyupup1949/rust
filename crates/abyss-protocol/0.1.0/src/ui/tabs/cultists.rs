use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::App;
use crate::game::cultist::{CultistState, CultistTier, UnlockCondition};
use crate::game::production;
use crate::ui::format::format_number;
use crate::ui::theme::Theme;

/// 渲染信徒页面
///
/// 布局：
/// - 左侧 65%：信徒列表（等级符号、名称、数量、单个产出、招募价格），支持 ↑↓ 导航高亮
/// - 右侧 35%：合成面板（所需数量、当前拥有、合成结果）
pub fn render_cultists(f: &mut Frame, area: Rect, app: &App) {
    let theme = Theme::default();

    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(area);

    render_cultist_list(f, h_chunks[0], app, &theme);
    render_fusion_panel(f, h_chunks[1], app, &theme);
}

/// 检查信徒等级是否已解锁
fn is_tier_unlocked(condition: &UnlockCondition, app: &App) -> bool {
    match condition {
        UnlockCondition::Always => true,
        UnlockCondition::TotalWhispers(required) => {
            app.game_state.stats.total_whispers_earned >= *required
        }
        UnlockCondition::Dimension(dim) => {
            app.game_state.stats.max_dimension_unlocked >= *dim
        }
        UnlockCondition::DimensionAndRebirths(dim, rebirths) => {
            app.game_state.stats.max_dimension_unlocked >= *dim
                && app.game_state.stats.total_rebirths >= *rebirths
        }
        UnlockCondition::DimensionAndTruths(dim, truths) => {
            app.game_state.stats.max_dimension_unlocked >= *dim
                && app.game_state.currency.forbidden_truths >= *truths
        }
    }
}

/// 根据信徒等级返回对应颜色
fn tier_color(tier: &CultistTier, theme: &Theme) -> ratatui::style::Color {
    match tier {
        CultistTier::T1 => theme.locked,
        CultistTier::T2 => theme.neon_green,
        CultistTier::T3 => theme.ghost_blue,
        CultistTier::T4 => theme.toxic_purple,
        CultistTier::T5 | CultistTier::T7 | CultistTier::T9 => theme.blood_red,
        CultistTier::T6 | CultistTier::T8 | CultistTier::T10 => theme.deep_gold,
    }
}

/// 左侧：信徒列表
fn render_cultist_list(f: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let title = app.locale.t("tabs.cultists");
    let block = Block::default()
        .title(title.to_string())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.bg));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.width < 4 || inner.height < 1 {
        return;
    }

    let mut lines: Vec<Line> = Vec::new();

    for (i, tier) in CultistTier::ALL.iter().enumerate() {
        let data = tier.data();
        let unlocked = is_tier_unlocked(&data.unlock_condition, app);
        let is_selected = i == app.cultist_selected;
        let color = tier_color(tier, theme);

        if unlocked {
            let name = app.locale.t(data.name_key);
            let count = app.game_state.cultists.counts[tier.index()];
            let per_cultist = app
                .last_tick_result
                .as_ref()
                .map(|r| r.production.per_cultist[tier.index()])
                .unwrap_or(data.base_production);
            let price = production::cultist_recruit_price(
                *tier,
                count,
                &app.game_state.shop,
            );
            let price_str = format_number(price);
            let recruit_label =
                app.locale.tf("tab_labels.cultist_recruit_price", &[&price_str]);

            let row_style = if is_selected {
                Style::default().bg(theme.border).add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };

            let spans = vec![
                Span::styled(
                    format!(" {} ", data.symbol),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{:<12}", name),
                    Style::default().fg(theme.text),
                ),
                Span::styled(
                    format!("×{:<6}", count),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("+{}/s  ", format_number(per_cultist)),
                    Style::default().fg(theme.neon_green),
                ),
                Span::styled(
                    recruit_label,
                    Style::default().fg(theme.ghost_blue),
                ),
            ];

            lines.push(Line::from(spans).style(row_style));
        } else {
            // 锁定状态
            let locked_label = app.locale.t("tab_labels.cultist_locked");
            let condition_text = unlock_condition_text(&data.unlock_condition, app);

            let row_style = if is_selected {
                Style::default().bg(theme.border).add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };

            let spans = vec![
                Span::styled(
                    format!(" {} ", data.symbol),
                    Style::default().fg(theme.locked),
                ),
                Span::styled(
                    format!("{} ", locked_label),
                    Style::default().fg(theme.locked),
                ),
                Span::styled(
                    condition_text,
                    Style::default().fg(theme.locked),
                ),
            ];

            lines.push(Line::from(spans).style(row_style));
        }
    }

    let widget = Paragraph::new(lines);
    f.render_widget(widget, inner);
}

/// 生成解锁条件描述文本
fn unlock_condition_text(condition: &UnlockCondition, app: &App) -> String {
    match condition {
        UnlockCondition::Always => String::new(),
        UnlockCondition::TotalWhispers(w) => {
            format!("({}≥{})", app.locale.t("resources.whispers"), format_number(*w))
        }
        UnlockCondition::Dimension(d) => {
            app.locale.tf("tab_labels.node_dimension", &[&d.to_string()])
        }
        UnlockCondition::DimensionAndRebirths(d, r) => {
            let dim_text = app.locale.tf("tab_labels.node_dimension", &[&d.to_string()]);
            format!("{} + ×{}R", dim_text, r)
        }
        UnlockCondition::DimensionAndTruths(d, t) => {
            let dim_text = app.locale.tf("tab_labels.node_dimension", &[&d.to_string()]);
            format!("{} + {}FT", dim_text, t)
        }
    }
}

/// 右侧：合成面板
fn render_fusion_panel(f: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let title = app.locale.t("tab_labels.cultist_fuse_panel");
    let block = Block::default()
        .title(title.to_string())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.bg));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.width < 4 || inner.height < 1 {
        return;
    }

    let selected_tier = CultistTier::ALL[app.cultist_selected];
    let data = selected_tier.data();
    let unlocked = is_tier_unlocked(&data.unlock_condition, app);
    let color = tier_color(&selected_tier, theme);

    let mut lines: Vec<Line> = Vec::new();

    // 当前选中等级标题
    let name = app.locale.t(data.name_key);
    lines.push(Line::from(vec![
        Span::styled(
            format!(" {} ", data.symbol),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            name.to_string(),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(""));

    if !unlocked {
        let locked_label = app.locale.t("tab_labels.cultist_locked");
        lines.push(Line::from(Span::styled(
            format!(" {}", locked_label),
            Style::default().fg(theme.locked),
        )));
        let widget = Paragraph::new(lines);
        f.render_widget(widget, inner);
        return;
    }

    let fusion_cost = CultistState::fusion_cost(&app.game_state.shop);
    let owned = app.game_state.cultists.counts[selected_tier.index()];

    // 合成所需数量
    let cost_str = app
        .locale
        .tf("tab_labels.cultist_fuse_cost", &[&fusion_cost.to_string()]);
    lines.push(Line::from(Span::styled(
        format!(" {}", cost_str),
        Style::default().fg(theme.text),
    )));

    // 当前拥有数量
    lines.push(Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled(
            format!("{}: ", app.locale.t("resources.whispers")),
            Style::default().fg(theme.text),
        ),
        Span::styled(
            format!("{}", owned),
            Style::default()
                .fg(if owned >= fusion_cost {
                    theme.neon_green
                } else {
                    theme.blood_red
                })
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" / {}", fusion_cost),
            Style::default().fg(theme.locked),
        ),
    ]));

    lines.push(Line::from(""));

    // 合成结果
    if let Some(next) = selected_tier.next_tier() {
        let next_data = next.data();
        let next_name = app.locale.t(next_data.name_key);
        let result_str = app
            .locale
            .tf("tab_labels.cultist_fuse_result", &[next_name]);
        let next_color = tier_color(&next, theme);

        lines.push(Line::from(vec![
            Span::styled(
                format!(" {}", result_str),
                Style::default().fg(next_color).add_modifier(Modifier::BOLD),
            ),
        ]));

        // 可合成状态指示
        lines.push(Line::from(""));
        if owned >= fusion_cost {
            lines.push(Line::from(Span::styled(
                " [F] ▸ ✓",
                Style::default()
                    .fg(theme.neon_green)
                    .add_modifier(Modifier::BOLD),
            )));
        } else {
            let needed = fusion_cost - owned;
            lines.push(Line::from(Span::styled(
                format!(" -{}", needed),
                Style::default().fg(theme.blood_red),
            )));
        }
    } else {
        // T10 — 最高等级，无法合成
        lines.push(Line::from(Span::styled(
            " MAX TIER",
            Style::default()
                .fg(theme.deep_gold)
                .add_modifier(Modifier::BOLD),
        )));
    }

    let widget = Paragraph::new(lines);
    f.render_widget(widget, inner);
}
