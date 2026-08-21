use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::App;
use crate::game::cultist::CultistTier;
use crate::ui::format::format_number;
use crate::ui::theme::Theme;

/// 渲染祭坛页面
///
/// 布局：
/// - 左侧 60%：信徒 ASCII 可视化（上）+ 统计摘要（下）
/// - 右侧 40%：Oracle Log 区域
pub fn render_altar(f: &mut Frame, area: Rect, app: &App) {
    let theme = Theme::default();

    // 左右分割：60% / 40%
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    render_left_panel(f, h_chunks[0], app, &theme);
    render_oracle_log(f, h_chunks[1], app, &theme);
}

/// 左侧面板：信徒可视化 + 统计
fn render_left_panel(f: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    // 上下分割：可视化区域占 70%，统计区域占 30%
    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

    render_cultist_visualization(f, v_chunks[0], app, theme);
    render_cultist_stats(f, v_chunks[1], app, theme);
}

/// 信徒 ASCII 可视化区域
fn render_cultist_visualization(f: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let san = app.game_state.san.current;

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if san < 20.0 {
            theme.blood_red
        } else {
            theme.border
        }))
        .style(Style::default().bg(theme.bg));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.width < 2 || inner.height < 1 {
        return;
    }

    // 如果祭坛动画有信徒，使用动画渲染
    if !app.animation.altar.cultists.is_empty() {
        // 注意：area_width/area_height 在 sync_cultists 时已设置
        app.animation.altar.render(f, inner, theme);
        return;
    }

    // 否则显示空祭坛（无信徒时画三角形祭坛 + 提示）
    if app.game_state.cultists.total_count() == 0 {
        // 画一个简单的祭坛图案
        let altar_lines = vec![
            Line::from(Span::styled("", Style::default())),
            Line::from(Span::styled("       △       ", Style::default().fg(theme.toxic_purple))),
            Line::from(Span::styled("      ╱ ╲      ", Style::default().fg(theme.toxic_purple))),
            Line::from(Span::styled("     ╱   ╲     ", Style::default().fg(theme.toxic_purple))),
            Line::from(Span::styled("    ╱  ◈  ╲    ", Style::default().fg(theme.toxic_purple))),
            Line::from(Span::styled("   ╱_______╲   ", Style::default().fg(theme.toxic_purple))),
            Line::from(Span::styled("", Style::default())),
            Line::from(Span::styled(
                "...",
                Style::default().fg(theme.locked),
            )),
        ];
        let widget = Paragraph::new(altar_lines).alignment(Alignment::Center);
        f.render_widget(widget, inner);
        return;
    }
    // 信徒存在但动画还没同步（不应该到这里，但作为 fallback）
    let empty_msg = Paragraph::new("...")
        .style(Style::default().fg(theme.locked))
        .alignment(Alignment::Center);
    f.render_widget(empty_msg, inner);
}

/// 信徒统计摘要区域
fn render_cultist_stats(f: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let stats_title = app.locale.t("tab_labels.altar_cultist_stats");
    let block = Block::default()
        .title(stats_title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.bg));
    let inner = block.inner(area);
    f.render_widget(block, area);

    // 两列布局：左列 T1-T5，右列 T6-T10
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    // 渲染左列 T1-T5
    let mut left_lines: Vec<Line> = Vec::new();
    for i in 0..5 {
        let tier = CultistTier::ALL[i];
        let name = app.locale.t(tier.data().name_key);
        let count = app.game_state.cultists.counts[tier.index()];
        let color = tier_color(&tier, theme);
        left_lines.push(Line::from(vec![
            Span::styled(format!("  T{} ", i + 1), Style::default().fg(theme.locked)),
            Span::styled(format!("{:<8}", name), Style::default().fg(theme.text)),
            Span::styled(
                format!("{:>4}", count),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
        ]));
    }
    f.render_widget(Paragraph::new(left_lines), cols[0]);

    // 渲染右列 T6-T10
    let mut right_lines: Vec<Line> = Vec::new();
    for i in 5..10 {
        let tier = CultistTier::ALL[i];
        let name = app.locale.t(tier.data().name_key);
        let count = app.game_state.cultists.counts[tier.index()];
        let color = tier_color(&tier, theme);
        right_lines.push(Line::from(vec![
            Span::styled(format!("  T{} ", i + 1), Style::default().fg(theme.locked)),
            Span::styled(format!("{:<8}", name), Style::default().fg(theme.text)),
            Span::styled(
                format!("{:>4}", count),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    // 右列最后加总计行（如果有空间）
    let total_count = app.game_state.cultists.total_count();
    let production_per_sec = app
        .last_tick_result
        .as_ref()
        .map(|r| r.production.total_production_per_sec)
        .unwrap_or(0.0);
    if right_lines.len() < inner.height as usize {
        right_lines.push(Line::from(vec![
            Span::styled("  ----------", Style::default().fg(theme.border)),
        ]));
    }
    if right_lines.len() < inner.height as usize {
        let total_label = app.locale.t("tab_labels.altar_total_production");
        right_lines.push(Line::from(vec![
            Span::styled(
                format!("  {} {} ", total_label, total_count),
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("+{}/s", format_number(production_per_sec)),
                Style::default().fg(theme.neon_green).add_modifier(Modifier::BOLD),
            ),
        ]));
    }
    f.render_widget(Paragraph::new(right_lines), cols[1]);
}

/// Oracle Log 区域（token 流入事件列表）
fn render_oracle_log(f: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let title = app.locale.t("tab_labels.altar_oracle_log");
    let san = app.game_state.san.current;

    let border_color = if san < 20.0 {
        theme.toxic_purple
    } else {
        theme.border
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(theme.bg));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    if let Some(ref tick) = app.last_tick_result {
        let prod = &tick.production;

        // 总产出
        lines.push(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                format!("{}/s", format_number(prod.total_production_per_sec)),
                Style::default().fg(theme.neon_green).add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(""));

        // 产出分解
        lines.push(Line::from(Span::styled(
            format!(
                "  {} {}/s",
                app.locale.t("tabs.cultists"),
                format_number(prod.total_cultist_production)
            ),
            Style::default().fg(theme.ghost_blue),
        )));

        if prod.node_independent_production > 0.0 {
            lines.push(Line::from(Span::styled(
                format!(
                    "  {} {}/s",
                    app.locale.t("tabs.nodes"),
                    format_number(prod.node_independent_production)
                ),
                Style::default().fg(theme.cyan),
            )));
        }

        // 加成信息（只显示非零的）
        let has_bonuses = prod.global_bonus_pct > 0.0
            || prod.dimension_bonus_pct > 0.0
            || prod.synergy_bonus_pct > 0.0;

        if has_bonuses {
            lines.push(Line::from(Span::styled(
                "  ──────────",
                Style::default().fg(theme.border),
            )));
        }

        if prod.global_bonus_pct > 0.0 {
            lines.push(Line::from(Span::styled(
                format!("  Global  +{:.0}%", prod.global_bonus_pct * 100.0),
                Style::default().fg(theme.deep_gold),
            )));
        }
        if prod.dimension_bonus_pct > 0.0 {
            lines.push(Line::from(Span::styled(
                format!("  Dim     +{:.0}%", prod.dimension_bonus_pct * 100.0),
                Style::default().fg(theme.amber),
            )));
        }
        if prod.synergy_bonus_pct > 0.0 {
            lines.push(Line::from(Span::styled(
                format!("  Synergy +{:.0}%", prod.synergy_bonus_pct * 100.0),
                Style::default().fg(theme.cyan),
            )));
        }

        // SAN 效率
        lines.push(Line::from(Span::styled(
            format!("  SAN eff  {:.0}%", prod.san_efficiency * 100.0),
            Style::default().fg(theme.san_color(san)),
        )));

        // 变异事件
        if let Some(ref mutation) = tick.mutation {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!(
                    "  MUTATION +{} / -{:.1} SAN",
                    format_number(mutation.whispers_gained),
                    mutation.san_cost
                ),
                Style::default().fg(theme.toxic_purple).add_modifier(Modifier::BOLD),
            )));
        }
    } else {
        lines.push(Line::from(Span::styled(
            "  ...",
            Style::default().fg(theme.locked),
        )));
    }

    // SAN 低时添加腐蚀装饰线
    if san < 30.0 {
        lines.push(Line::from(""));
        let glitch = if san < 10.0 {
            "  CORRUPTED"
        } else {
            "  signal degraded"
        };
        lines.push(Line::from(Span::styled(
            glitch,
            Style::default().fg(theme.blood_red).add_modifier(Modifier::SLOW_BLINK),
        )));
    }

    let widget = Paragraph::new(lines);
    f.render_widget(widget, inner);
}

/// 根据信徒等级返回对应颜色
/// T1: 暗灰 (locked), T2: 绿 (neon_green), T3: 蓝 (ghost_blue),
/// T4: 紫 (toxic_purple), T5+: 红/金交替
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
