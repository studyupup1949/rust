use crossterm::event::{KeyCode, KeyEvent};
use rand::rngs::ThreadRng;
use ratatui::layout::Rect;

use crate::game::cultist::CultistTier;
use crate::game::nodes::NodeId;
use crate::game::rebirth;
use crate::game::save::SaveManager;
use crate::game::shop::{ShopPath, ShopUpgradeId};
use crate::game::{BatchMode, GameState, RebirthSummary, TickResult};
use crate::i18n::{Locale, LocaleManager};
use crate::monitor::{MockMonitor, TokenEvent, TokenMonitor};
use crate::ui::ActiveTab;
use crate::ui::animation::AnimationState;

/// Monitor 的统一包装，支持真实监控和 Mock 模式
pub enum MonitorKind {
    Real(TokenMonitor),
    Mock(MockMonitor),
}

impl MonitorKind {
    pub fn poll_events(&mut self) -> Vec<TokenEvent> {
        match self {
            MonitorKind::Real(m) => m.poll_events(),
            MonitorKind::Mock(m) => m.poll_events(),
        }
    }
}

/// App 运行模式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppMode {
    Running,
    Quitting,
}

/// 主应用结构，聚合所有子系统
pub struct App {
    pub mode: AppMode,
    pub active_tab: ActiveTab,
    pub game_state: GameState,
    pub locale: LocaleManager,
    pub monitor: MonitorKind,
    pub save_manager: SaveManager,
    pub mock_mode: bool,
    pub tick_count: u64,
    /// 当前批量购买模式
    pub batch_mode: BatchMode,
    /// 信徒列表选中索引
    pub cultist_selected: usize,
    /// 节点列表选中索引
    pub node_selected: usize,
    /// 商店选中路径
    pub shop_selected_path: usize,
    /// 商店选中索引
    pub shop_selected_index: usize,
    /// 成就列表选中索引
    pub achievement_selected: usize,
    /// 是否显示帮助浮层
    pub show_help: bool,
    /// 是否显示转生确认对话框
    pub show_rebirth_confirm: bool,
    /// 转生结算数据（显示结算画面时非 None）
    pub rebirth_summary: Option<RebirthSummary>,
    /// 上次 tick 的结果（供 UI 渲染使用）
    pub last_tick_result: Option<TickResult>,
    /// 动画系统状态
    pub animation: AnimationState,
    /// RNG 实例
    pub rng: ThreadRng,
}

impl App {
    /// 初始化所有子系统，创建 App 实例
    pub fn new(
        locale: LocaleManager,
        game_state: GameState,
        save_manager: SaveManager,
        monitor: MonitorKind,
        mock_mode: bool,
    ) -> Self {
        Self {
            mode: AppMode::Running,
            active_tab: ActiveTab::Altar,
            game_state,
            locale,
            monitor,
            save_manager,
            mock_mode,
            tick_count: 0,
            batch_mode: BatchMode::X1,
            cultist_selected: 0,
            node_selected: 0,
            shop_selected_path: 0,
            shop_selected_index: 0,
            achievement_selected: 0,
            show_help: false,
            show_rebirth_confirm: false,
            rebirth_summary: None,
            last_tick_result: None,
            animation: AnimationState::new(),
            rng: rand::thread_rng(),
        }
    }

    /// 是否应该退出
    pub fn should_quit(&self) -> bool {
        self.mode == AppMode::Quitting
    }

    /// 处理键盘输入
    pub fn handle_key_event(&mut self, key: KeyEvent) {
        // 1. 如果显示转生结算画面，任意键关闭
        if self.rebirth_summary.is_some() {
            self.rebirth_summary = None;
            return;
        }

        // 2. 如果显示帮助浮层，任意键关闭
        if self.show_help {
            self.show_help = false;
            return;
        }

        // 3. 如果显示转生确认对话框，只响应 Y/N/Esc
        if self.show_rebirth_confirm {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    if rebirth::can_active_rebirth(&self.game_state) {
                        let summary =
                            rebirth::execute_rebirth(&mut self.game_state, false);
                        let area = Rect::new(0, 0, 80, 24);
                        self.animation.trigger_collapse(summary.clone(), &ratatui::buffer::Buffer::empty(area));
                        self.rebirth_summary = Some(summary);
                    }
                    self.show_rebirth_confirm = false;
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    self.show_rebirth_confirm = false;
                }
                _ => {}
            }
            return;
        }

        // 4. 全局快捷键
        match key.code {
            // Tab / Shift+Tab: 切换面板
            KeyCode::Tab => {
                self.active_tab = self.active_tab.next();
            }
            KeyCode::BackTab => {
                self.active_tab = self.active_tab.prev();
            }
            // 数字键 1-5: 直接跳转 Tab
            KeyCode::Char('1') => self.active_tab = ActiveTab::Altar,
            KeyCode::Char('2') => self.active_tab = ActiveTab::Cultists,
            KeyCode::Char('3') => self.active_tab = ActiveTab::Nodes,
            KeyCode::Char('4') => self.active_tab = ActiveTab::Shop,
            KeyCode::Char('5') => self.active_tab = ActiveTab::Achievements,
            // Q / Esc: 退出
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.mode = AppMode::Quitting;
            }
            KeyCode::Esc => {
                self.mode = AppMode::Quitting;
            }
            // S: 手动存档
            KeyCode::Char('s') | KeyCode::Char('S') => {
                let _ = self.save_manager.save(&self.game_state);
            }
            // L: 切换语言
            KeyCode::Char('l') | KeyCode::Char('L') => {
                let new_locale = match self.locale.current() {
                    Locale::En => Locale::Zh,
                    Locale::Zh => Locale::En,
                };
                let _ = self.locale.switch_locale(new_locale);
            }
            // Space: 修复 SAN
            KeyCode::Char(' ') => {
                if self.game_state.san.repair(
                    &mut self.game_state.currency,
                    &self.game_state.nodes,
                    &self.game_state.shop,
                ).is_ok() {
                    let area = Rect::new(0, 0, 80, 24);
                    self.animation.trigger_san_repair_flash(area);
                }
            }
            // R: 主动转生确认
            KeyCode::Char('r') | KeyCode::Char('R') => {
                if rebirth::can_active_rebirth(&self.game_state) {
                    self.show_rebirth_confirm = true;
                }
            }
            // ?: 帮助浮层
            KeyCode::Char('?') => {
                self.show_help = true;
            }
            // 其他键: 交给 Tab 专属处理
            _ => {
                self.handle_tab_key(key);
            }
        }
    }

    /// Tab 专属快捷键分发
    fn handle_tab_key(&mut self, key: KeyEvent) {
        match self.active_tab {
            ActiveTab::Cultists => self.handle_cultist_key(key),
            ActiveTab::Nodes => self.handle_node_key(key),
            ActiveTab::Shop => self.handle_shop_key(key),
            ActiveTab::Achievements => self.handle_achievement_key(key),
            _ => {}
        }
    }

    /// 信徒页快捷键：↑↓ 导航、Enter 招募、F 合成
    fn handle_cultist_key(&mut self, key: KeyEvent) {
        let area = Rect::new(0, 0, 80, 24); // 动画区域占位，渲染时使用实际区域
        match key.code {
            KeyCode::Up => {
                if self.cultist_selected > 0 {
                    self.cultist_selected -= 1;
                }
            }
            KeyCode::Down => {
                if self.cultist_selected < CultistTier::ALL.len() - 1 {
                    self.cultist_selected += 1;
                }
            }
            KeyCode::Enter => {
                let tier = CultistTier::ALL[self.cultist_selected];
                if self.game_state.cultists.recruit(
                    tier,
                    &mut self.game_state.currency,
                    &self.game_state.shop,
                ).is_ok() {
                    self.animation.trigger_purchase_slide(area);
                    self.animation.altar.sync_cultists(&self.game_state.cultists, area, &mut self.rng);
                }
            }
            KeyCode::Char('f') | KeyCode::Char('F') => {
                let tier = CultistTier::ALL[self.cultist_selected];
                if let Ok(refund) = self.game_state.cultists.fuse(
                    tier,
                    &self.game_state.shop,
                    &self.game_state.nodes,
                ) {
                    self.game_state.currency.whispers += refund;
                    self.game_state.stats.total_fusions += 1;
                    self.animation.trigger_fusion(area);
                    self.animation.altar.sync_cultists(&self.game_state.cultists, area, &mut self.rng);
                }
            }
            _ => {}
        }
    }

    /// 节点页快捷键：↑↓ 导航、Enter 购买、B 批量模式切换
    fn handle_node_key(&mut self, key: KeyEvent) {
        let area = Rect::new(0, 0, 80, 24);
        match key.code {
            KeyCode::Up => {
                if self.node_selected > 0 {
                    self.node_selected -= 1;
                }
            }
            KeyCode::Down => {
                if self.node_selected < NodeId::ALL.len() - 1 {
                    self.node_selected += 1;
                }
            }
            KeyCode::Enter => {
                let node_id = NodeId::ALL[self.node_selected];
                if self.game_state.nodes.purchase(
                    node_id,
                    &mut self.game_state.currency,
                    &self.game_state.shop,
                ).is_ok() {
                    self.animation.trigger_node_price_flash(area);
                }
            }
            KeyCode::Char('b') | KeyCode::Char('B') => {
                self.batch_mode = self.batch_mode.next();
            }
            _ => {}
        }
    }

    /// 商店页快捷键：↑↓←→ 导航、Enter 购买
    fn handle_shop_key(&mut self, key: KeyEvent) {
        const PATHS: [ShopPath; 5] = [
            ShopPath::Power,
            ShopPath::Knowledge,
            ShopPath::Madness,
            ShopPath::Transcendence,
            ShopPath::Cult,
        ];

        match key.code {
            KeyCode::Left => {
                if self.shop_selected_path > 0 {
                    self.shop_selected_path -= 1;
                    self.shop_selected_index = 0;
                }
            }
            KeyCode::Right => {
                if self.shop_selected_path < PATHS.len() - 1 {
                    self.shop_selected_path += 1;
                    self.shop_selected_index = 0;
                }
            }
            KeyCode::Up => {
                if self.shop_selected_index > 0 {
                    self.shop_selected_index -= 1;
                }
            }
            KeyCode::Down => {
                let current_path = PATHS[self.shop_selected_path];
                let count = ShopUpgradeId::ALL
                    .iter()
                    .filter(|id| id.data().path == current_path)
                    .count();
                if self.shop_selected_index + 1 < count {
                    self.shop_selected_index += 1;
                }
            }
            KeyCode::Enter => {
                let current_path = PATHS[self.shop_selected_path];
                let upgrades: Vec<ShopUpgradeId> = ShopUpgradeId::ALL
                    .iter()
                    .filter(|id| id.data().path == current_path)
                    .copied()
                    .collect();
                if let Some(&upgrade_id) = upgrades.get(self.shop_selected_index) {
                    let _ = self.game_state.shop.purchase(
                        upgrade_id,
                        &mut self.game_state.currency,
                    );
                }
            }
            _ => {}
        }
    }

    /// 成就页快捷键：↑↓ 导航
    fn handle_achievement_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => {
                if self.achievement_selected > 0 {
                    self.achievement_selected -= 1;
                }
            }
            KeyCode::Down => {
                if self.achievement_selected < 52 {
                    self.achievement_selected += 1;
                }
            }
            _ => {}
        }
    }

    /// 执行一次 tick：poll events → engine tick → check auto-save → trigger animations
    pub fn tick(&mut self) {
        // 1. 获取新 token 事件
        let events = self.monitor.poll_events();

        // 2. 处理每个事件，并记录 token 流入（用于心电图和闪白动画）
        let mut token_count: u64 = 0;
        for event in &events {
            token_count += event.total_tokens();
            self.game_state.process_token_event(event);
        }

        // 3. 执行游戏引擎 tick (dt = 0.1 秒，对应主循环 100ms / 10 FPS)
        let dt = 0.1;
        let result = self.game_state.tick(dt, &mut self.rng);

        // 4. 连接游戏事件到动画触发
        let screen_area = ratatui::layout::Rect::new(0, 0, 80, 24); // 默认区域，渲染时会用实际区域
        self.connect_animation_events(&result, token_count, screen_area);

        self.last_tick_result = Some(result);

        // 5. 检查自动存档
        if self.save_manager.should_auto_save() {
            let _ = self.save_manager.save(&self.game_state);
        }

        // 6. 自动同步信徒到祭坛动画（每帧检查数量变化）
        let total_cultists: u32 = self.game_state.cultists.counts.iter().sum();
        let anim_cultists = self.animation.altar.cultists.len() as u32;
        if total_cultists != anim_cultists {
            let area = ratatui::layout::Rect::new(
                0, 0,
                self.animation.altar.area_width.max(40),
                self.animation.altar.area_height.max(15),
            );
            self.animation.altar.sync_cultists(&self.game_state.cultists, area, &mut self.rng);
        }

        // 7. 递增 tick_count
        self.tick_count += 1;
    }

    /// 根据 TickResult 中的事件触发对应动画
    fn connect_animation_events(
        &mut self,
        result: &TickResult,
        token_count: u64,
        area: Rect,
    ) {
        // Token 流入 → 闪白 + 心电图记录
        if token_count > 0 {
            self.animation.trigger_token_flash(area);
            self.animation.heartbeat.record_tokens(token_count as u32);
        }

        // 变异事件 → 全屏脉冲
        if result.mutation.is_some() {
            self.animation.trigger_mutation_pulse(area);
        }

        // 新成就解锁 → 成就横幅
        if !result.new_achievements.is_empty() {
            self.animation.trigger_achievement_banner(area);
        }

        // 新协同激活 → 波纹效果
        if !result.new_synergies.is_empty() {
            self.animation.trigger_synergy_ripple(area);
        }

        // 维度解锁 → 全屏特效
        if result.dimension_unlocked.is_some() {
            self.animation.trigger_dimension_unlock(area);
        }

        // 自动招募 → 购买滑入 + 祭坛同步
        if !result.auto_recruits.is_empty() {
            self.animation.trigger_purchase_slide(area);
            self.animation.altar.sync_cultists(&self.game_state.cultists, area, &mut self.rng);
        }

        // 自动合成 → 合成动画 + 祭坛同步
        if !result.auto_fusions.is_empty() {
            self.animation.trigger_fusion(area);
            self.animation.altar.sync_cultists(&self.game_state.cultists, area, &mut self.rng);
        }

        // 被动转生（SAN 崩溃）→ 崩溃序列
        if let Some(ref summary) = result.rebirth_triggered {
            self.animation.trigger_collapse(summary.clone(), &ratatui::buffer::Buffer::empty(area));
            self.rebirth_summary = Some(summary.clone());
        }
    }
}
