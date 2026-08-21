use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::App;
use crate::game::shop::{ShopPath, ShopUpgradeId};
use crate::ui::theme::Theme;

/// 五条路径的顺序
const PATHS: [ShopPath; 5] = [
    ShopPath::Power,
    ShopPath::Knowledge,
    ShopPath::Madness,
    ShopPath::Transcendence,
    ShopPath::Cult,
];

/// 渲染真理商店页面
///
/// 布局：左侧窄栏路径列表 + 右侧宽栏升级详情
pub fn render_shop(f: &mut Frame, area: Rect, app: &App) {
    let theme = Theme::default();

    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(18), Constraint::Min(30)])
        .split(area);

    render_path_list(f, h_chunks[0], app, &theme);
    render_upgrade_detail(f, h_chunks[1], app, &theme);
}

/// 获取路径对应的 i18n key
fn path_title_key(path: ShopPath) -> &'static str {
    match path {
        ShopPath::Power => "tab_labels.shop_path_power",
        ShopPath::Knowledge => "tab_labels.shop_path_knowledge",
        ShopPath::Madness => "tab_labels.shop_path_madness",
        ShopPath::Transcendence => "tab_labels.shop_path_transcendence",
        ShopPath::Cult => "tab_labels.shop_path_cult",
    }
}

/// 获取路径对应的主题色
fn path_color(path: ShopPath, theme: &Theme) -> ratatui::style::Color {
    match path {
        ShopPath::Power => theme.blood_red,
        ShopPath::Knowledge => theme.ghost_blue,
        ShopPath::Madness => theme.toxic_purple,
        ShopPath::Transcendence => theme.deep_gold,
        ShopPath::Cult => theme.amber,
    }
}

/// 获取某路径下的所有升级 ID
fn upgrades_for_path(path: ShopPath) -> Vec<ShopUpgradeId> {
    ShopUpgradeId::ALL
        .iter()
        .filter(|id| id.data().path == path)
        .copied()
        .collect()
}

/// 左侧：路径列表
fn render_path_list(f: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let title = app.locale.t("tabs.shop");
    let block = Block::default()
        .title(title.to_string())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.bg));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.width < 2 || inner.height < 1 {
        return;
    }

    let truths = app.game_state.currency.forbidden_truths;
    let truths_label = app.locale.t("status.truths_label");
    let mut lines: Vec<Line> = Vec::new();

    // 真理余额
    lines.push(Line::from(vec![
        Span::styled(
            format!(" {}: ", truths_label),
            Style::default().fg(theme.text),
        ),
        Span::styled(
            format!("{}", truths),
            Style::default().fg(theme.deep_gold).add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(""));

    for (i, &path) in PATHS.iter().enumerate() {
        let is_selected = i == app.shop_selected_path;
        let color = path_color(path, theme);
        let name = app.locale.t(path_title_key(path));

        let row_style = if is_selected {
            Style::default().fg(color).add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            Style::default().fg(color)
        };

        // 显示路径名 + 已购买数/总数
        let upgrades = upgrades_for_path(path);
        let bought = upgrades.iter().filter(|id| {
            let data = id.data();
            app.game_state.shop.level(**id) >= data.max_level
        }).count();

        lines.push(Line::from(vec![
            Span::styled(
                format!(" {} ", name),
                row_style,
            ),
            Span::styled(
                format!("{}/{}", bought, upgrades.len()),
                Style::default().fg(if is_selected { color } else { theme.locked }),
            ),
        ]));
    }

    // 底部导航提示
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        " ←→ ↑↓ Enter",
        Style::default().fg(theme.locked),
    )));

    let widget = Paragraph::new(lines);
    f.render_widget(widget, inner);
}

/// 右侧：当前路径的升级详情列表
fn render_upgrade_detail(f: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let current_path = PATHS[app.shop_selected_path];
    let color = path_color(current_path, theme);
    let title = app.locale.t(path_title_key(current_path));

    let block = Block::default()
        .title(Span::styled(
            title.to_string(),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .style(Style::default().bg(theme.bg));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.width < 4 || inner.height < 1 {
        return;
    }

    let upgrades = upgrades_for_path(current_path);
    let truths = app.game_state.currency.forbidden_truths;
    let mut lines: Vec<Line> = Vec::new();

    for (i, &upgrade_id) in upgrades.iter().enumerate() {
        let data = upgrade_id.data();
        let current_level = app.game_state.shop.level(upgrade_id);
        let is_maxed = current_level >= data.max_level;
        let is_selected = i == app.shop_selected_index;

        let name = app.locale.t(data.name_key);
        let desc = app.locale.t(data.desc_key);

        // 等级显示
        let level_str = app.locale.tf(
            "tab_labels.shop_level",
            &[&current_level.to_string(), &data.max_level.to_string()],
        );

        // 费用/状态
        let (cost_line, cost_color) = if is_maxed {
            if data.is_one_time {
                (app.locale.t("tab_labels.shop_purchased").to_string(), theme.neon_green)
            } else {
                (app.locale.t("tab_labels.shop_max_level").to_string(), theme.neon_green)
            }
        } else {
            let next_cost = data.costs[current_level as usize];
            let can_afford = truths >= next_cost;
            let cost_text = app.locale.tf("tab_labels.shop_cost", &[&next_cost.to_string()]);
            let c = if can_afford { theme.deep_gold } else { theme.locked };
            (cost_text, c)
        };

        let name_color = if is_maxed { theme.neon_green } else { color };

        let row_style = if is_selected {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };

        // 第一行：名称 + 等级 + 费用
        lines.push(
            Line::from(vec![
                Span::styled(
                    format!(" {}", name),
                    Style::default().fg(name_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" {} ", level_str),
                    Style::default().fg(theme.text),
                ),
                Span::styled(
                    cost_line,
                    Style::default().fg(cost_color),
                ),
            ]).style(row_style),
        );

        // 第二行：完整效果描述（自动换行）
        lines.push(
            Line::from(Span::styled(
                format!("   {}", desc),
                Style::default().fg(theme.locked),
            )).style(row_style),
        );

        // 升级之间空一行
        if i + 1 < upgrades.len() {
            lines.push(Line::from(""));
        }
    }

    let widget = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(widget, inner);
}
