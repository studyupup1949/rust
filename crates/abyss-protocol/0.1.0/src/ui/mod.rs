pub mod animation;
pub mod format;
pub mod layout;
pub mod theme;
pub mod tabs;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, Paragraph, Wrap},
};

use crate::app::App;
use crate::game::rebirth;
use layout::AppLayout;
use theme::Theme;

/// 当前活跃的 Tab
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActiveTab {
    Altar,
    Cultists,
    Nodes,
    Shop,
    Achievements,
}

impl ActiveTab {
    /// Tab 对应的 i18n key
    pub fn i18n_key(&self) -> &'static str {
        match self {
            ActiveTab::Altar => "tabs.altar",
            ActiveTab::Cultists => "tabs.cultists",
            ActiveTab::Nodes => "tabs.nodes",
            ActiveTab::Shop => "tabs.shop",
            ActiveTab::Achievements => "tabs.achievements",
        }
    }

    /// 所有 Tab 按顺序
    pub const ALL: [ActiveTab; 5] = [
        ActiveTab::Altar,
        ActiveTab::Cultists,
        ActiveTab::Nodes,
        ActiveTab::Shop,
        ActiveTab::Achievements,
    ];

    /// 切换到下一个 Tab
    pub fn next(&self) -> Self {
        match self {
            ActiveTab::Altar => ActiveTab::Cultists,
            ActiveTab::Cultists => ActiveTab::Nodes,
            ActiveTab::Nodes => ActiveTab::Shop,
            ActiveTab::Shop => ActiveTab::Achievements,
            ActiveTab::Achievements => ActiveTab::Altar,
        }
    }

    /// 切换到上一个 Tab
    pub fn prev(&self) -> Self {
        match self {
            ActiveTab::Altar => ActiveTab::Achievements,
            ActiveTab::Cultists => ActiveTab::Altar,
            ActiveTab::Nodes => ActiveTab::Cultists,
            ActiveTab::Shop => ActiveTab::Nodes,
            ActiveTab::Achievements => ActiveTab::Shop,
        }
    }
}

/// UI 渲染器
pub struct Renderer {
    theme: Theme,
}

impl Renderer {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    /// 渲染完整的一帧（4 阶段渲染管线）
    pub fn render(&self, frame: &mut Frame, app: &mut App) {
        // 阶段 0：崩溃序列检查，激活时全屏覆盖并 return
        if app.animation.collapse.render(frame, &app.locale) {
            return;
        }

        let layout = AppLayout::new(frame.area());

        // 阶段 1：正常渲染（现有渲染逻辑不变）
        self.render_status_bar(frame, layout.status_bar, app);
        self.render_tab_bar(frame, layout.tab_bar, app);
        self.render_content(frame, layout.content, app);
        self.render_sidebar(frame, layout.sidebar, app);
        self.render_resource_bar(frame, layout.resource_bar, app);
        self.render_keybind_bar(frame, layout.keybind_bar, app);

        // 阶段 2：tachyonfx 事件特效（已禁用，避免全局闪烁）
        // let full_area = frame.area();
        // let elapsed = tachyonfx::Duration::from_millis(100);
        // app.animation.process_effects(elapsed, frame.buffer_mut(), full_area);

        // 阶段 3：SAN 腐蚀后处理
        let san_area = frame.area();
        app.animation.san_corruption.apply(frame.buffer_mut(), san_area);

        // 浮层渲染（在动画之上）
        if app.show_help {
            self.render_help_overlay(frame, frame.area(), app);
        }
        if app.show_rebirth_confirm {
            self.render_rebirth_confirm(frame, frame.area(), app);
        }
        if app.rebirth_summary.is_some() {
            self.render_rebirth_settlement(frame, frame.area(), app);
        }
    }

    /// 渲染顶部状态栏：协议名称 | SAN 进度条 | 计时器
    fn render_status_bar(&self, frame: &mut Frame, area: Rect, app: &App) {
        let game = &app.game_state;
        let locale = &app.locale;
        let cycle_start = game.current_cycle_start;

        let block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(self.theme.border))
            .style(Style::default().bg(self.theme.bg));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        // 四列布局：协议名 | 心电图波形 | 计时器 | SAN值
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(45),
                Constraint::Percentage(20),
                Constraint::Percentage(15),
            ])
            .split(inner);

        // 左侧：协议名称
        let title = format!("[!] {}", locale.t("ui.title"));
        let title_widget = Paragraph::new(title)
            .style(Style::default().fg(self.theme.neon_green).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Left);
        frame.render_widget(title_widget, cols[0]);

        // 中间：心电图波形
        app.animation.heartbeat.render(frame, cols[1], &self.theme);

        // 右侧：当前轮计时器
        let elapsed = chrono::Utc::now().signed_duration_since(cycle_start);
        let total_secs = elapsed.num_seconds().max(0) as u64;
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        let seconds = total_secs % 60;
        let timer_text = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);
        let timer_widget = Paragraph::new(timer_text)
            .style(Style::default().fg(self.theme.text))
            .alignment(Alignment::Right);
        frame.render_widget(timer_widget, cols[2]);

        // 最右：SAN 值（紧凑文字）
        let san = game.san.current;
        let san_color = self.theme.san_color(san);
        let san_text = format!("SAN:{:.0}%", san);
        let san_widget = Paragraph::new(san_text)
            .style(Style::default().fg(san_color).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Right);
        frame.render_widget(san_widget, cols[3]);
    }

    /// 渲染 Tab 导航栏
    fn render_tab_bar(&self, frame: &mut Frame, area: Rect, app: &App) {
        let active = app.active_tab;
        let locale = &app.locale;

        let spans: Vec<Span> = ActiveTab::ALL
            .iter()
            .enumerate()
            .flat_map(|(i, tab)| {
                let name = locale.t(tab.i18n_key());
                let label = format!("{}:{}", i + 1, name);
                let span = if *tab == active {
                    Span::styled(
                        format!("[{}]", label),
                        Style::default()
                            .fg(self.theme.neon_green)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    Span::styled(
                        format!(" {} ", label),
                        Style::default().fg(self.theme.locked),
                    )
                };
                vec![span, Span::raw(" ")]
            })
            .collect();

        let tabs_line = Line::from(spans);
        let widget = Paragraph::new(tabs_line)
            .style(Style::default().bg(self.theme.bg));
        frame.render_widget(widget, area);
    }

    /// 渲染主内容区（根据当前 Tab 分发到对应渲染器）
    fn render_content(&self, frame: &mut Frame, area: Rect, app: &App) {
        match app.active_tab {
            ActiveTab::Altar => tabs::altar::render_altar(frame, area, app),
            ActiveTab::Cultists => tabs::cultists::render_cultists(frame, area, app),
            ActiveTab::Nodes => tabs::nodes::render_nodes(frame, area, app),
            ActiveTab::Shop => tabs::shop::render_shop(frame, area, app),
            ActiveTab::Achievements => tabs::achievements::render_achievements(frame, area, app),
        }
    }

    /// 渲染侧边栏：上半部分资源概览，下半部分 ASCII Art
    fn render_sidebar(&self, frame: &mut Frame, area: Rect, app: &App) {
        let game = &app.game_state;
        let locale = &app.locale;

        let block = Block::default()
            .title(locale.t("ui.resources_title"))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border))
            .style(Style::default().bg(self.theme.bg));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        // 上下分割：资源概览 60% / ASCII Art 40%
        let parts = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(60),
                Constraint::Percentage(40),
            ])
            .split(inner);

        // 上半部分：资源概览
        let whispers_label = locale.t("status.whispers_label");
        let truths_label = locale.t("status.truths_label");
        let production_label = locale.t("status.production_rate");
        let san_label = locale.t("status.san_label");

        let production_per_sec = app.last_tick_result
            .as_ref()
            .map(|r| r.production.total_production_per_sec)
            .unwrap_or(0.0);
        let rate_str = format!("+{}/s", format::format_number(production_per_sec));
        let san = game.san.current;

        let resource_lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    format!("  {}: ", whispers_label),
                    Style::default().fg(self.theme.text),
                ),
                Span::styled(
                    format::format_number(game.currency.whispers),
                    Style::default().fg(self.theme.ghost_blue).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    format!("  {}: ", production_label),
                    Style::default().fg(self.theme.text),
                ),
                Span::styled(
                    rate_str,
                    Style::default().fg(self.theme.neon_green),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    format!("  {}: ", truths_label),
                    Style::default().fg(self.theme.text),
                ),
                Span::styled(
                    format!("{}", game.currency.forbidden_truths),
                    Style::default().fg(self.theme.deep_gold).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    format!("  {}: ", san_label),
                    Style::default().fg(self.theme.text),
                ),
                Span::styled(
                    format!("{:.0}%", san),
                    Style::default().fg(self.theme.san_color(san)).add_modifier(Modifier::BOLD),
                ),
            ]),
        ];
        let resources_widget = Paragraph::new(resource_lines);
        frame.render_widget(resources_widget, parts[0]);

        // 下半部分：深渊 ASCII Art（纯 ASCII，避免宽度问题）
        let art = vec![
            Line::from(Span::styled("    .     .    ", Style::default().fg(self.theme.locked))),
            Line::from(Span::styled("   .\\ | /.   ", Style::default().fg(self.theme.locked))),
            Line::from(Span::styled("   .-'---'-.   ", Style::default().fg(self.theme.toxic_purple))),
            Line::from(Span::styled("   |   *   |   ", Style::default().fg(self.theme.toxic_purple))),
            Line::from(Span::styled("   '-.._..-'   ", Style::default().fg(self.theme.toxic_purple))),
            Line::from(Span::styled("   ./ | \\.   ", Style::default().fg(self.theme.locked))),
            Line::from(Span::styled("    '     '    ", Style::default().fg(self.theme.locked))),
        ];
        let art_widget = Paragraph::new(art).alignment(Alignment::Center);
        frame.render_widget(art_widget, parts[1]);
    }

    /// 渲染底部资源栏：Whispers + 产出速率 | Truths | 转生次数
    fn render_resource_bar(&self, frame: &mut Frame, area: Rect, app: &App) {
        let game = &app.game_state;
        let locale = &app.locale;

        let whispers_label = locale.t("status.whispers_label");
        let truths_label = locale.t("status.truths_label");
        let rebirths_label = locale.t("status.rebirths");

        let production_per_sec = app.last_tick_result
            .as_ref()
            .map(|r| r.production.total_production_per_sec)
            .unwrap_or(0.0);
        let rate_str = format!("  +{}/s", format::format_number(production_per_sec));

        let spans = vec![
            Span::styled(
                format!(" {}: ", whispers_label),
                Style::default().fg(self.theme.text),
            ),
            Span::styled(
                format::format_number(game.currency.whispers),
                Style::default().fg(self.theme.ghost_blue).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                rate_str,
                Style::default().fg(self.theme.neon_green),
            ),
            Span::styled(
                "  │  ",
                Style::default().fg(self.theme.border),
            ),
            Span::styled(
                format!("{}: ", truths_label),
                Style::default().fg(self.theme.text),
            ),
            Span::styled(
                format!("{}", game.currency.forbidden_truths),
                Style::default().fg(self.theme.deep_gold).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  │  ",
                Style::default().fg(self.theme.border),
            ),
            Span::styled(
                format!("{}: ", rebirths_label),
                Style::default().fg(self.theme.text),
            ),
            Span::styled(
                format!("{}", game.stats.total_rebirths),
                Style::default().fg(self.theme.amber),
            ),
        ];

        let line = Line::from(spans);
        let widget = Paragraph::new(line)
            .style(Style::default().bg(self.theme.bg));
        frame.render_widget(widget, area);
    }

    /// 渲染快捷键提示栏
    fn render_keybind_bar(&self, frame: &mut Frame, area: Rect, app: &App) {
        let locale = &app.locale;

        let keys = [
            "keybinds.tab_switch",
            "keybinds.san_repair",
            "keybinds.save_key",
            "keybinds.lang_key",
            "keybinds.quit_key",
        ];

        let spans: Vec<Span> = keys
            .iter()
            .flat_map(|key| {
                vec![
                    Span::styled(
                        format!(" {} ", locale.t(key)),
                        Style::default().fg(self.theme.text),
                    ),
                    Span::styled(
                        " │ ",
                        Style::default().fg(self.theme.border),
                    ),
                ]
            })
            .collect();

        let line = Line::from(spans);
        let widget = Paragraph::new(line)
            .style(Style::default().bg(self.theme.bg));
        frame.render_widget(widget, area);
    }

    /// 计算居中弹窗区域
    fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(area);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }

    /// 渲染帮助浮层
    fn render_help_overlay(&self, frame: &mut Frame, area: Rect, app: &App) {
        let locale = &app.locale;
        let popup_area = Self::centered_rect(60, 70, area);

        frame.render_widget(Clear, popup_area);

        let block = Block::default()
            .title(locale.t("help.help_title"))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.neon_green))
            .style(Style::default().bg(self.theme.bg));
        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        let lines = vec![
            Line::from(Span::styled(
                locale.t("help.help_global"),
                Style::default().fg(self.theme.deep_gold).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(locale.t("help.help_tab_switch"), Style::default().fg(self.theme.text))),
            Line::from(Span::styled(locale.t("help.help_save"), Style::default().fg(self.theme.text))),
            Line::from(Span::styled(locale.t("help.help_lang"), Style::default().fg(self.theme.text))),
            Line::from(Span::styled(locale.t("help.help_quit"), Style::default().fg(self.theme.text))),
            Line::from(Span::styled(locale.t("help.help_space"), Style::default().fg(self.theme.text))),
            Line::from(Span::styled(locale.t("help.help_rebirth"), Style::default().fg(self.theme.text))),
            Line::from(Span::styled(locale.t("help.help_question"), Style::default().fg(self.theme.text))),
            Line::from(""),
            Line::from(Span::styled(
                locale.t("help.help_cultist"),
                Style::default().fg(self.theme.deep_gold).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(locale.t("help.help_updown"), Style::default().fg(self.theme.text))),
            Line::from(Span::styled(locale.t("help.help_enter"), Style::default().fg(self.theme.text))),
            Line::from(Span::styled(locale.t("help.help_fuse"), Style::default().fg(self.theme.text))),
            Line::from(""),
            Line::from(Span::styled(
                locale.t("help.help_node"),
                Style::default().fg(self.theme.deep_gold).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(locale.t("help.help_updown"), Style::default().fg(self.theme.text))),
            Line::from(Span::styled(locale.t("help.help_enter"), Style::default().fg(self.theme.text))),
            Line::from(Span::styled(locale.t("help.help_batch"), Style::default().fg(self.theme.text))),
            Line::from(""),
            Line::from(Span::styled(
                locale.t("help.help_shop"),
                Style::default().fg(self.theme.deep_gold).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(locale.t("help.help_arrows"), Style::default().fg(self.theme.text))),
            Line::from(Span::styled(locale.t("help.help_enter"), Style::default().fg(self.theme.text))),
        ];

        let widget = Paragraph::new(lines).wrap(Wrap { trim: false });
        frame.render_widget(widget, inner);
    }

    /// 渲染转生确认对话框
    fn render_rebirth_confirm(&self, frame: &mut Frame, area: Rect, app: &App) {
        let locale = &app.locale;
        let popup_area = Self::centered_rect(40, 25, area);

        frame.render_widget(Clear, popup_area);

        let block = Block::default()
            .title(locale.t("rebirth.confirm_title"))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.deep_gold))
            .style(Style::default().bg(self.theme.bg));
        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        // 计算预计获得真理数
        let total_whispers = app.game_state.stats.total_whispers_earned;
        let mind_splitter = app.game_state.nodes.count(crate::game::nodes::NodeId::MindSplitter);
        let causality_weapon = app.game_state.nodes.count(crate::game::nodes::NodeId::CausalityWeapon);
        let has_collapse_accel = app.game_state.shop.has(crate::game::shop::ShopUpgradeId::CollapseAccelerator);
        let truths = rebirth::calculate_rebirth_truths(
            total_whispers,
            false, // 主动转生
            mind_splitter,
            causality_weapon,
            has_collapse_accel,
        );

        let truths_text = locale.tf("rebirth.confirm_truths", &[&truths.to_string()]);

        let lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                truths_text,
                Style::default().fg(self.theme.deep_gold).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                locale.t("rebirth.confirm_yes_no"),
                Style::default().fg(self.theme.text),
            )),
        ];

        let widget = Paragraph::new(lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });
        frame.render_widget(widget, inner);
    }

    /// 渲染转生结算画面
    fn render_rebirth_settlement(&self, frame: &mut Frame, area: Rect, app: &App) {
        let locale = &app.locale;
        let summary = match &app.rebirth_summary {
            Some(s) => s,
            None => return,
        };

        let popup_area = Self::centered_rect(50, 60, area);

        frame.render_widget(Clear, popup_area);

        let block = Block::default()
            .title(locale.t("rebirth.rebirth_title"))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.toxic_purple))
            .style(Style::default().bg(self.theme.bg));
        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        // 格式化持续时间
        let dur_secs = summary.cycle_duration_secs;
        let hours = dur_secs / 3600;
        let minutes = (dur_secs % 3600) / 60;
        let seconds = dur_secs % 60;
        let duration_str = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);

        let lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(format!("{}: ", locale.t("rebirth.cycle_duration")), Style::default().fg(self.theme.text)),
                Span::styled(duration_str, Style::default().fg(self.theme.ghost_blue)),
            ]),
            Line::from(vec![
                Span::styled(format!("{}: ", locale.t("rebirth.whispers_harvested")), Style::default().fg(self.theme.text)),
                Span::styled(format::format_number(summary.whispers_harvested), Style::default().fg(self.theme.ghost_blue)),
            ]),
            Line::from(vec![
                Span::styled(format!("{}: ", locale.t("rebirth.nodes_constructed")), Style::default().fg(self.theme.text)),
                Span::styled(format!("{}", summary.nodes_constructed), Style::default().fg(self.theme.ghost_blue)),
            ]),
            Line::from(vec![
                Span::styled(format!("{}: ", locale.t("rebirth.peak_production")), Style::default().fg(self.theme.text)),
                Span::styled(format!("{}/s", format::format_number(summary.peak_production)), Style::default().fg(self.theme.neon_green)),
            ]),
            Line::from(vec![
                Span::styled(format!("{}: ", locale.t("rebirth.san_repairs")), Style::default().fg(self.theme.text)),
                Span::styled(format!("{}", summary.san_repairs), Style::default().fg(self.theme.ghost_blue)),
            ]),
            Line::from(vec![
                Span::styled(format!("{}: ", locale.t("rebirth.mutations_witnessed")), Style::default().fg(self.theme.text)),
                Span::styled(format!("{}", summary.mutations_witnessed), Style::default().fg(self.theme.toxic_purple)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(format!("{}: ", locale.t("rebirth.truths_gained")), Style::default().fg(self.theme.text)),
                Span::styled(format!("+{}", summary.truths_gained), Style::default().fg(self.theme.deep_gold).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled(format!("{}: ", locale.t("rebirth.total_truths")), Style::default().fg(self.theme.text)),
                Span::styled(format!("{}", summary.total_truths_after), Style::default().fg(self.theme.deep_gold)),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                locale.t("rebirth.press_any_key"),
                Style::default().fg(self.theme.locked),
            )),
        ];

        let widget = Paragraph::new(lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });
        frame.render_widget(widget, inner);
    }
}
