use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::App;
use crate::game::cultist::UnlockCondition;
use crate::game::engine::BatchMode;
use crate::game::nodes::NodeId;
use crate::game::production;
use crate::game::synergy::SynergyId;
use crate::ui::format::format_number;
use crate::ui::theme::Theme;

/// 渲染节点页面
///
/// 布局：
/// - 左侧 65%：节点列表（维度分组、名称、数量、效果、价格），支持 ↑↓ 导航高亮
/// - 右侧 35%：已激活协同效应列表 + 维度信息 + 批量模式指示
pub fn render_nodes(f: &mut Frame, area: Rect, app: &App) {
    let theme = Theme::default();

    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(area);

    render_node_list(f, h_chunks[0], app, &theme);
    render_side_panel(f, h_chunks[1], app, &theme);
}

/// 检查节点解锁条件是否满足
fn is_node_unlocked(condition: &UnlockCondition, app: &App) -> bool {
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

/// 根据维度返回对应颜色
fn dimension_color(dimension: u32, theme: &Theme) -> ratatui::style::Color {
    match dimension {
        1 => theme.neon_green,
        2 => theme.ghost_blue,
        3 => theme.toxic_purple,
        _ => theme.text,
    }
}

/// 获取当前维度中可见的节点列表（仅已解锁维度的节点）
fn visible_nodes(app: &App) -> Vec<NodeId> {
    let max_dim = app.game_state.stats.max_dimension_unlocked;
    NodeId::ALL
        .iter()
        .filter(|id| id.data().dimension <= max_dim)
        .copied()
        .collect()
}

/// 左侧：节点列表（按维度分组）
fn render_node_list(f: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let title = app.locale.t("tabs.nodes");
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

    let nodes = visible_nodes(app);
    let mut lines: Vec<Line> = Vec::new();
    let mut current_dim: u32 = 0;
    let mut flat_index: usize = 0;

    for &node_id in &nodes {
        let data = node_id.data();

        // 维度分隔头
        if data.dimension != current_dim {
            if current_dim != 0 {
                lines.push(Line::from(""));
            }
            current_dim = data.dimension;
            let dim_color = dimension_color(current_dim, theme);
            let dim_label = app
                .locale
                .tf("tab_labels.node_dimension", &[&current_dim.to_string()]);
            lines.push(Line::from(Span::styled(
                format!("── {} ──", dim_label),
                Style::default()
                    .fg(dim_color)
                    .add_modifier(Modifier::BOLD),
            )));
        }

        let unlocked = is_node_unlocked(&data.unlock_condition, app);
        let is_selected = flat_index == app.node_selected;
        let color = dimension_color(data.dimension, theme);

        if unlocked {
            let name = app.locale.t(data.name_key);
            let owned = app.game_state.nodes.count(node_id);
            let desc = app.locale.t(data.desc_key);

            // 计算价格（考虑批量模式）
            let (price_str, batch_label) = format_batch_price(
                node_id, owned, app,
            );

            let row_style = if is_selected {
                Style::default()
                    .bg(theme.border)
                    .add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };

            // 第一行：名称 + 数量 + 价格
            let count_label = app
                .locale
                .tf("tab_labels.node_count", &[&owned.to_string()]);

            let mut spans = vec![
                Span::styled(
                    format!(" {:<16}", name),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{:<10}", count_label),
                    Style::default().fg(theme.text),
                ),
                Span::styled(
                    format!("{}{}", batch_label, price_str),
                    Style::default().fg(theme.ghost_blue),
                ),
            ];

            // 显示最大数量标记
            if let Some(max) = data.max_count {
                if owned >= max {
                    spans.push(Span::styled(
                        " MAX",
                        Style::default()
                            .fg(theme.deep_gold)
                            .add_modifier(Modifier::BOLD),
                    ));
                }
            }

            lines.push(Line::from(spans).style(row_style));

            // 第二行：效果描述（缩进）
            lines.push(
                Line::from(Span::styled(
                    format!("   {}", desc),
                    Style::default().fg(theme.locked),
                ))
                .style(row_style),
            );
        } else {
            // 锁定状态
            let locked_label = app.locale.t("tab_labels.node_locked");
            let condition_text = unlock_condition_text(&data.unlock_condition, app);

            let row_style = if is_selected {
                Style::default()
                    .bg(theme.border)
                    .add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };

            lines.push(
                Line::from(vec![
                    Span::styled(
                        format!(" {} ", locked_label),
                        Style::default().fg(theme.locked),
                    ),
                    Span::styled(condition_text, Style::default().fg(theme.locked)),
                ])
                .style(row_style),
            );
        }

        flat_index += 1;
    }

    // 滚动：让选中项保持在可视区域中间
    let scroll_offset = if app.node_selected > 2 {
        // 每个节点约占 2-3 行，粗略估算
        ((app.node_selected as u16).saturating_sub(2)) * 2
    } else {
        0
    };
    let widget = Paragraph::new(lines).scroll((scroll_offset, 0));
    f.render_widget(widget, inner);
}
fn format_batch_price(
    node_id: NodeId,
    owned: u32,
    app: &App,
) -> (String, String) {
    let data = node_id.data();
    let discount_level = app
        .game_state
        .shop
        .levels
        .get(&crate::game::shop::ShopUpgradeId::NodeDiscount)
        .copied()
        .unwrap_or(0);
    let discount_factor = (1.0 - 0.05 * discount_level as f64).max(0.0);

    match app.batch_mode {
        BatchMode::X1 => {
            let price = production::node_purchase_price(node_id, owned, &app.game_state.shop);
            (format_number(price), String::new())
        }
        BatchMode::X10 | BatchMode::X100 => {
            let count = app.batch_mode.count().unwrap();
            let total = production::batch_price(
                data.base_price,
                data.price_growth,
                owned,
                count,
                discount_factor,
            );
            let label = batch_mode_label(app);
            (format_number(total), format!("{} ", label))
        }
        BatchMode::XMax => {
            let budget = app.game_state.currency.whispers;
            let max_n = production::max_purchasable(
                data.base_price,
                data.price_growth,
                owned,
                budget,
                discount_factor,
            );
            let label = batch_mode_label(app);
            if max_n > 0 {
                let total = production::batch_price(
                    data.base_price,
                    data.price_growth,
                    owned,
                    max_n,
                    discount_factor,
                );
                (
                    format!("{}(×{})", format_number(total), max_n),
                    format!("{} ", label),
                )
            } else {
                (format_number(0.0), format!("{} ", label))
            }
        }
    }
}

/// 获取批量模式的 i18n 标签
fn batch_mode_label(app: &App) -> String {
    let key = match app.batch_mode {
        BatchMode::X1 => "batch.batch_x1",
        BatchMode::X10 => "batch.batch_x10",
        BatchMode::X100 => "batch.batch_x100",
        BatchMode::XMax => "batch.batch_max",
    };
    app.locale.t(key).to_string()
}

/// 生成解锁条件描述文本
fn unlock_condition_text(condition: &UnlockCondition, app: &App) -> String {
    match condition {
        UnlockCondition::Always => String::new(),
        UnlockCondition::TotalWhispers(w) => {
            format!(
                "({}≥{})",
                app.locale.t("resources.whispers"),
                format_number(*w)
            )
        }
        UnlockCondition::Dimension(d) => {
            app.locale
                .tf("tab_labels.node_dimension", &[&d.to_string()])
        }
        UnlockCondition::DimensionAndRebirths(d, r) => {
            let dim_text = app
                .locale
                .tf("tab_labels.node_dimension", &[&d.to_string()]);
            format!("{} + ×{}R", dim_text, r)
        }
        UnlockCondition::DimensionAndTruths(d, t) => {
            let dim_text = app
                .locale
                .tf("tab_labels.node_dimension", &[&d.to_string()]);
            format!("{} + {}FT", dim_text, t)
        }
    }
}

/// 右侧面板：协同效应 + 维度信息 + 批量模式
fn render_side_panel(f: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(55),
            Constraint::Percentage(30),
            Constraint::Percentage(15),
        ])
        .split(area);

    render_synergies(f, v_chunks[0], app, theme);
    render_dimension_info(f, v_chunks[1], app, theme);
    render_batch_indicator(f, v_chunks[2], app, theme);
}

/// 已激活协同效应列表
fn render_synergies(f: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let title = app.locale.t("tab_labels.node_synergies");
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

    for &synergy_id in SynergyId::ALL.iter() {
        let data = synergy_id.data();
        let is_active = app.game_state.synergies.active.contains(&synergy_id);

        if is_active {
            let name = app.locale.t(data.name_key);
            let effect = app.locale.t(data.effect_desc_key);
            lines.push(Line::from(vec![
                Span::styled(
                    " ✦ ",
                    Style::default()
                        .fg(theme.cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    name.to_string(),
                    Style::default()
                        .fg(theme.cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
            lines.push(Line::from(Span::styled(
                format!("   {}", effect),
                Style::default().fg(theme.text),
            )));
        }
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "  ...",
            Style::default().fg(theme.locked),
        )));
    }

    let widget = Paragraph::new(lines);
    f.render_widget(widget, inner);
}

/// 维度信息面板
fn render_dimension_info(f: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let block = Block::default()
        .title("Dimensions")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.bg));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.width < 4 || inner.height < 1 {
        return;
    }

    let max_dim = app.game_state.stats.max_dimension_unlocked;
    let mut lines: Vec<Line> = Vec::new();

    for dim in 1..=3u32 {
        let dim_label = app.locale.tf("tab_labels.node_dimension", &[&dim.to_string()]);
        let color = dimension_color(dim, theme);
        let node_count = app.game_state.nodes.dimension_total(dim);

        if dim <= max_dim {
            lines.push(Line::from(vec![
                Span::styled(
                    format!(" {} ", dim_label),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("({} nodes)", node_count),
                    Style::default().fg(theme.text),
                ),
            ]));
        } else {
            lines.push(Line::from(Span::styled(
                format!(" {} 🔒", dim_label),
                Style::default().fg(theme.locked),
            )));
        }
    }

    let widget = Paragraph::new(lines);
    f.render_widget(widget, inner);
}

/// 批量模式指示器
fn render_batch_indicator(f: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let batch_title = app.locale.t("batch.batch_label");
    let block = Block::default()
        .title(batch_title.to_string())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.bg));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.width < 4 || inner.height < 1 {
        return;
    }

    let mode_label = batch_mode_label(app);
    let batch_key_hint = app.locale.t("keybinds_extra.batch_key");

    let lines = vec![Line::from(vec![
        Span::styled(
            format!(" {} ", mode_label),
            Style::default()
                .fg(theme.deep_gold)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  {}", batch_key_hint),
            Style::default().fg(theme.locked),
        ),
    ])];

    let widget = Paragraph::new(lines);
    f.render_widget(widget, inner);
}
