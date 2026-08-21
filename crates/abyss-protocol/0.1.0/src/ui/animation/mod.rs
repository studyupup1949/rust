pub mod persistent;
pub mod effects;
pub mod san_corruption;
pub mod collapse;
pub mod altar;
pub mod heartbeat;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use rand::Rng;
use tachyonfx::{Effect, Shader};

use crate::game::rebirth::RebirthSummary;

use self::effects::{
    achievement_banner_effect, dimension_unlock_effect, fusion_effect, mutation_pulse_effect,
    node_price_flash_effect, purchase_slide_effect, san_repair_flash_effect,
    synergy_ripple_effect, token_flash_effect,
};
use self::persistent::PersistentAnimations;
use self::san_corruption::SanCorruptionState;
use self::collapse::CollapseSequence;
use self::altar::AltarAnimations;
use self::heartbeat::TokenHeartbeat;

/// 默认心电图宽度
const DEFAULT_HEARTBEAT_WIDTH: usize = 30;

/// 动画系统聚合状态
pub struct AnimationState {
    /// 常驻动画状态
    pub persistent: PersistentAnimations,
    /// tachyonfx 事件特效列表
    pub effects: Vec<Effect>,
    /// SAN 腐蚀效果状态
    pub san_corruption: SanCorruptionState,
    /// 转生崩溃序列状态
    pub collapse: CollapseSequence,
    /// 祭坛信徒动画状态
    pub altar: AltarAnimations,
    /// Token 心电图状态
    pub heartbeat: TokenHeartbeat,
    /// 累计时间（秒），用于驱动常驻动画
    pub elapsed_secs: f64,
    /// Token flash 冷却计时器（秒），> 0 时不触发新的 flash
    pub token_flash_cooldown: f64,
}

impl AnimationState {
    pub fn new() -> Self {
        Self {
            persistent: PersistentAnimations::new(),
            effects: Vec::new(),
            san_corruption: SanCorruptionState::new(),
            collapse: CollapseSequence::new(),
            altar: AltarAnimations::new(),
            heartbeat: TokenHeartbeat::new(DEFAULT_HEARTBEAT_WIDTH),
            elapsed_secs: 0.0,
            token_flash_cooldown: 0.0,
        }
    }

    /// 每帧更新所有动画状态
    /// dt: 帧间隔（秒），通常为 0.1
    /// whispers: 实际碎语值
    /// san: 实际 SAN 值
    /// screen_area: 屏幕区域（用于 SAN 腐蚀位置生成）
    /// rng: 随机数生成器
    pub fn update(
        &mut self,
        dt: f64,
        whispers: f64,
        san: f64,
        screen_area: Rect,
        rng: &mut impl Rng,
    ) {
        self.elapsed_secs += dt;
        self.token_flash_cooldown = (self.token_flash_cooldown - dt).max(0.0);
        self.persistent.update(dt, whispers, san);
        self.san_corruption.update(dt, san, screen_area, rng);
        self.collapse.update(dt);
        self.altar.update(dt, san, rng);
        self.heartbeat.update(dt);
    }

    /// 处理 tachyonfx 事件特效
    /// 遍历所有效果，调用 process 推进动画，移除已完成的效果
    pub fn process_effects(&mut self, elapsed: tachyonfx::Duration, buf: &mut Buffer, area: Rect) {
        self.effects.retain_mut(|effect| {
            // Shader::process 返回 Some(remaining) 表示未完成，None 表示已完成
            effect.process(elapsed, buf, area).is_some()
        });
    }

    /// Token 闪白（已禁用：全局闪烁效果）
    pub fn trigger_token_flash(&mut self, _area: Rect) {}

    /// 购买信徒滑入效果（已禁用）
    pub fn trigger_purchase_slide(&mut self, _area: Rect) {}

    /// 购买节点价格闪绿效果（已禁用）
    pub fn trigger_node_price_flash(&mut self, _area: Rect) {}

    /// 信徒合成动画（已禁用）
    pub fn trigger_fusion(&mut self, _area: Rect) {}

    /// 变异事件全屏脉冲（已禁用）
    pub fn trigger_mutation_pulse(&mut self, _area: Rect) {}

    /// SAN 修复闪绿（已禁用）
    pub fn trigger_san_repair_flash(&mut self, _area: Rect) {}

    /// 协同激活波纹效果（已禁用）
    pub fn trigger_synergy_ripple(&mut self, _area: Rect) {}

    /// 成就解锁横幅（已禁用）
    pub fn trigger_achievement_banner(&mut self, _area: Rect) {}

    /// 维度解锁全屏特效（已禁用）
    pub fn trigger_dimension_unlock(&mut self, _area: Rect) {}

    /// 触发崩溃序列
    /// 需求 22.1
    pub fn trigger_collapse(&mut self, summary: RebirthSummary, current_screen: &Buffer) {
        self.collapse.trigger(summary, current_screen);
    }
}
