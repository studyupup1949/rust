use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::App;
use crate::game::achievements::{AchievementCategory, AchievementId};
use crate::ui::theme::Theme;

/// 五个类别的显示顺序
const CATEGORIES: [AchievementCategory; 5] = [
    AchievementCategory::Sacrifice,
    AchievementCategory::Collapse,
    AchievementCategory::Cult,
    AchievementCategory::Building,
    AchievementCategory::Hidden,
];

/// 获取类别对应的 i18n key
fn category_i18n_key(cat: AchievementCategory) -> &'static str {
    match cat {
        AchievementCategory::Sacrifice => "tab_labels.achievement_category_sacrifice",
        AchievementCategory::Collapse => "tab_labels.achievement_category_collapse",
        AchievementCategory::Cult => "tab_labels.achievement_category_cult",
        AchievementCategory::Building => "tab_labels.achievement_category_building",
        AchievementCategory::Hidden => "tab_labels.achievement_category_hidden",
    }
}

/// 获取类别对应的主题色
fn category_color(cat: AchievementCategory, theme: &Theme) -> ratatui::style::Color {
    match cat {
        AchievementCategory::Sacrifice => theme.blood_red,
        AchievementCategory::Collapse => theme.toxic_purple,
        AchievementCategory::Cult => theme.ghost_blue,
        AchievementCategory::Building => theme.neon_green,
        AchievementCategory::Hidden => theme.amber,
    }
}

/// 渲染成就页面
///
/// 按五个类别分组显示成就列表，支持 ↑↓ 导航高亮。
/// - 已达成：完整信息（名称 + 描述 + 奖励）+ ✓ 标记
/// - 未达成非隐藏：名称 + 进度条
/// - 隐藏未达成：显示 "???"
pub fn render_achievements(f: &mut Frame, area: Rect, app: &App) {
    let theme = Theme::default();

    let title = app.locale.t("tabs.achievements");
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
    // 全局可见行索引，用于匹配 app.achievement_selected
    let mut visible_index: usize = 0;

    for &cat in &CATEGORIES {
        let color = category_color(cat, &theme);
        let cat_name = app.locale.t(category_i18n_key(cat));

        // 收集该类别下的成就
        let achievements: Vec<AchievementId> = AchievementId::ALL
            .iter()
            .filter(|id| id.data().category == cat)
            .copied()
            .collect();

        // 统计已解锁数
        let unlocked_count = achievements
            .iter()
            .filter(|id| app.game_state.achievements.unlocked.contains(id))
            .count();

        // 类别标题行
        lines.push(Line::from(vec![
            Span::styled(
                format!("═══ {} ", cat_name),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("({}/{}) ", unlocked_count, achievements.len()),
                Style::default().fg(theme.locked),
            ),
            Span::styled(
                "═══",
                Style::default().fg(color),
            ),
        ]));

        // 每个成就一行
        for &ach_id in &achievements {
            let data = ach_id.data();
            let is_unlocked = app.game_state.achievements.unlocked.contains(&ach_id);
            let is_selected = visible_index == app.achievement_selected;

            let row_style = if is_selected {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };

            let line = if is_unlocked {
                // 已达成：✓ + 名称 + 描述 + 奖励
                let name = app.locale.t(data.name_key);
                let desc = app.locale.t(data.desc_key);
                let unlocked_label = app.locale.t("tab_labels.achievement_unlocked");

                let mut spans = vec![
                    Span::styled(
                        format!(" {} ", unlocked_label),
                        Style::default().fg(theme.neon_green).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("{}", name),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" - {}", desc),
                        Style::default().fg(theme.text),
                    ),
                ];

                if data.reward_truths > 0 {
                    let reward = app.locale.tf(
                        "tab_labels.achievement_reward",
                        &[&data.reward_truths.to_string()],
                    );
                    spans.push(Span::styled(
                        format!("  {}", reward),
                        Style::default().fg(theme.deep_gold),
                    ));
                }

                Line::from(spans).style(row_style)
            } else if data.is_hidden {
                // 隐藏未达成：???
                let hidden_label = app.locale.t("tab_labels.achievement_hidden");
                Line::from(vec![
                    Span::styled(
                        format!("   {}", hidden_label),
                        Style::default().fg(theme.locked),
                    ),
                ]).style(row_style)
            } else {
                // 未达成非隐藏：名称 + 进度指示
                let name = app.locale.t(data.name_key);
                Line::from(vec![
                    Span::styled(
                        "   ○ ",
                        Style::default().fg(theme.locked),
                    ),
                    Span::styled(
                        format!("{}", name),
                        Style::default().fg(theme.text),
                    ),
                    Span::styled(
                        " ░░░░░░░░",
                        Style::default().fg(theme.locked),
                    ),
                ]).style(row_style)
            };

            lines.push(line);
            visible_index += 1;
        }

        // 类别之间空一行
        lines.push(Line::from(""));
    }

    let widget = Paragraph::new(lines)
        .scroll((app.achievement_selected.saturating_sub(inner.height as usize / 2) as u16, 0));
    f.render_widget(widget, inner);
}
