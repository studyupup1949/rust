// 祭坛信徒动画（位置、闪烁、光晕）

use rand::Rng;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::Frame;
use std::f64::consts::PI;

use crate::game::cultist::{CultistState, CultistTier};
use crate::ui::theme::Theme;

// ── 常量 ──────────────────────────────────────────────

/// SAN 疯狂模式阈值
pub const SAN_MADNESS_THRESHOLD: f64 = 30.0;
/// 疯狂模式速度倍数
pub const SAN_MADNESS_SPEED_MULT: f64 = 3.0;
/// 同类聚集概率
pub const CULTIST_CLUSTER_PROB: f64 = 0.30;

/// T1 移动间隔（秒）
const MOVE_INTERVAL_T1: f64 = 2.0;
/// T2 移动间隔（秒）
const MOVE_INTERVAL_T2: f64 = 1.5;
/// T3+ 移动间隔（秒）
const MOVE_INTERVAL_T3_PLUS: f64 = 1.0;

/// 四方向偏移：上、下、左、右
const DIRECTIONS: [(i16, i16); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];

// ── 数据结构 ──────────────────────────────────────────

/// 单个信徒的动画状态
#[derive(Clone)]
pub struct CultistAnim {
    /// 当前位置（祭坛内相对坐标）
    pub x: u16,
    pub y: u16,
    /// 信徒等级
    pub tier: CultistTier,
    /// 移动计时器（秒）
    pub move_timer: f64,
    /// 移动间隔（T1: 2.0, T2: 1.5, T3+: 1.0）
    pub move_interval: f64,
    /// 闪烁相位（0.0 - 2π，每个信徒不同）
    pub flicker_phase: f64,
}

/// 祭坛动画状态
pub struct AltarAnimations {
    /// 所有信徒的动画状态
    pub cultists: Vec<CultistAnim>,
    /// 祭坛区域宽度（用于边界检查）
    pub area_width: u16,
    /// 祭坛区域高度
    pub area_height: u16,
}

/// 根据信徒等级返回基础移动间隔
fn base_move_interval(tier: CultistTier) -> f64 {
    match tier {
        CultistTier::T1 => MOVE_INTERVAL_T1,
        CultistTier::T2 => MOVE_INTERVAL_T2,
        _ => MOVE_INTERVAL_T3_PLUS,
    }
}


/// 判断信徒等级是否为 T5 或更高（拥有光晕效果）
pub fn has_halo(tier: CultistTier) -> bool {
    tier.index() >= CultistTier::T5.index()
}

impl AltarAnimations {
    pub fn new() -> Self {
        Self {
            cultists: Vec::new(),
            area_width: 0,
            area_height: 0,
        }
    }

    /// 同步信徒列表（招募/合成后数量变化时调用）
    ///
    /// 根据 `CultistState` 中各等级的数量，增减 `self.cultists`：
    /// - 新增信徒：分配随机位置（不与已有信徒重叠）和唯一闪烁相位
    /// - 多余信徒：从末尾移除
    pub fn sync_cultists(
        &mut self,
        cultist_state: &CultistState,
        area: Rect,
        rng: &mut impl Rng,
    ) {
        self.area_width = area.width;
        self.area_height = area.height;

        // 区域太小则清空
        if area.width == 0 || area.height == 0 {
            self.cultists.clear();
            return;
        }

        // 构建目标数量：按等级展开为 flat list
        let mut desired: Vec<CultistTier> = Vec::new();
        for &tier in &CultistTier::ALL {
            let count = cultist_state.counts[tier.index()];
            for _ in 0..count {
                desired.push(tier);
            }
        }

        // 保留已有的、等级匹配的信徒（按等级顺序匹配）
        let mut kept: Vec<CultistAnim> = Vec::new();
        let mut old_by_tier: Vec<Vec<CultistAnim>> = vec![Vec::new(); 10];
        for c in self.cultists.drain(..) {
            old_by_tier[c.tier.index()].push(c);
        }

        // 收集已使用的相位，确保新信徒相位唯一
        let mut used_phases: Vec<f64> = Vec::new();

        for &tier in &desired {
            if let Some(existing) = old_by_tier[tier.index()].pop() {
                used_phases.push(existing.flicker_phase);
                kept.push(existing);
            } else {
                // 新信徒：分配随机位置和唯一相位
                let pos = self.find_random_empty_pos(&kept, rng);
                let phase = self.generate_unique_phase(&used_phases, rng);
                used_phases.push(phase);
                let interval = base_move_interval(tier);
                kept.push(CultistAnim {
                    x: pos.0,
                    y: pos.1,
                    tier,
                    move_timer: rng.gen_range(0.0..interval),
                    move_interval: interval,
                    flicker_phase: phase,
                });
            }
        }

        self.cultists = kept;
    }

    /// 更新所有信徒动画
    /// san: 当前 SAN 值（< 30 时速度 ×3 + 随机模式）
    pub fn update(&mut self, dt: f64, san: f64, rng: &mut impl Rng) {
        if self.area_width == 0 || self.area_height == 0 {
            return;
        }

        let is_madness = san < SAN_MADNESS_THRESHOLD;
        let speed_mult = if is_madness { SAN_MADNESS_SPEED_MULT } else { 1.0 };

        let len = self.cultists.len();
        for i in 0..len {
            // 计算有效间隔
            let effective_interval = self.cultists[i].move_interval / speed_mult;

            self.cultists[i].move_timer += dt;

            if self.cultists[i].move_timer >= effective_interval {
                self.cultists[i].move_timer -= effective_interval;

                // 计算移动方向
                if let Some((dx, dy)) = self.calculate_move_direction(i, san, rng) {
                    let new_x = (self.cultists[i].x as i16 + dx) as u16;
                    let new_y = (self.cultists[i].y as i16 + dy) as u16;
                    self.cultists[i].x = new_x;
                    self.cultists[i].y = new_y;
                }
            }
        }
    }

    /// 计算信徒移动方向
    /// SAN < 30 时完全随机；否则 30% 偏向同类，70% 随机相邻空位
    fn calculate_move_direction(
        &self,
        idx: usize,
        san: f64,
        rng: &mut impl Rng,
    ) -> Option<(i16, i16)> {
        let cx = self.cultists[idx].x;
        let cy = self.cultists[idx].y;
        let tier = self.cultists[idx].tier;

        // 收集所有可用的相邻空位
        let empty_neighbors: Vec<(i16, i16)> = DIRECTIONS
            .iter()
            .filter(|&&(dx, dy)| {
                let nx = cx as i16 + dx;
                let ny = cy as i16 + dy;
                // 边界检查
                if nx < 0 || ny < 0 || nx >= self.area_width as i16 || ny >= self.area_height as i16
                {
                    return false;
                }
                // 碰撞检查：不与其他信徒重叠
                let (ux, uy) = (nx as u16, ny as u16);
                !self
                    .cultists
                    .iter()
                    .enumerate()
                    .any(|(j, c)| j != idx && c.x == ux && c.y == uy)
            })
            .copied()
            .collect();

        if empty_neighbors.is_empty() {
            return None;
        }

        let is_madness = san < SAN_MADNESS_THRESHOLD;

        // SAN < 30：完全随机
        if is_madness {
            let pick = rng.gen_range(0..empty_neighbors.len());
            return Some(empty_neighbors[pick]);
        }

        // 正常模式：30% 偏向同类，70% 随机
        let roll: f64 = rng.gen();
        if roll < CULTIST_CLUSTER_PROB {
            // 尝试偏向同类信徒
            if let Some(dir) = self.direction_toward_same_tier(idx, tier, &empty_neighbors) {
                return Some(dir);
            }
        }

        // 随机选择一个空位
        let pick = rng.gen_range(0..empty_neighbors.len());
        Some(empty_neighbors[pick])
    }

    /// 返回 T5+ 信徒的光晕位置（周围 1 格空位）
    pub fn halo_positions(&self) -> Vec<(u16, u16)> {
        let mut positions = Vec::new();
        let occupied: Vec<(u16, u16)> = self.cultists.iter().map(|c| (c.x, c.y)).collect();

        for c in &self.cultists {
            if !has_halo(c.tier) {
                continue;
            }
            // 检查 8 个方向（包括对角线）
            for dy in -1i16..=1 {
                for dx in -1i16..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = c.x as i16 + dx;
                    let ny = c.y as i16 + dy;
                    if nx < 0
                        || ny < 0
                        || nx >= self.area_width as i16
                        || ny >= self.area_height as i16
                    {
                        continue;
                    }
                    let pos = (nx as u16, ny as u16);
                    if !occupied.contains(&pos) && !positions.contains(&pos) {
                        positions.push(pos);
                    }
                }
            }
        }
        positions
    }

    // ── 内部辅助方法 ──────────────────────────────────

    /// 在已有信徒列表中找一个不重叠的随机位置
    fn find_random_empty_pos(
        &self,
        existing: &[CultistAnim],
        rng: &mut impl Rng,
    ) -> (u16, u16) {
        let w = self.area_width;
        let h = self.area_height;
        // 最多尝试 100 次随机位置
        for _ in 0..100 {
            let x = rng.gen_range(0..w);
            let y = rng.gen_range(0..h);
            let collides = existing.iter().any(|c| c.x == x && c.y == y)
                || self.cultists.iter().any(|c| c.x == x && c.y == y);
            if !collides {
                return (x, y);
            }
        }
        // 回退：顺序扫描找第一个空位
        for y in 0..h {
            for x in 0..w {
                let collides = existing.iter().any(|c| c.x == x && c.y == y)
                    || self.cultists.iter().any(|c| c.x == x && c.y == y);
                if !collides {
                    return (x, y);
                }
            }
        }
        // 极端情况：区域已满，放在 (0,0)
        (0, 0)
    }

    /// 生成一个与已有相位不同的唯一闪烁相位
    fn generate_unique_phase(&self, used: &[f64], rng: &mut impl Rng) -> f64 {
        // 最多尝试 100 次
        for _ in 0..100 {
            let phase = rng.gen_range(0.0..(2.0 * PI));
            // 检查与已有相位的最小距离（避免过于接近）
            let too_close = used.iter().any(|&p| (p - phase).abs() < 0.01);
            if !too_close {
                return phase;
            }
        }
        // 回退：均匀分布
        let n = used.len() as f64 + 1.0;
        (used.len() as f64 / n) * 2.0 * PI
    }

    /// 找到偏向同类信徒的移动方向
    /// 在可用空位中选择最接近同类信徒的方向
    fn direction_toward_same_tier(
        &self,
        idx: usize,
        tier: CultistTier,
        empty_neighbors: &[(i16, i16)],
    ) -> Option<(i16, i16)> {
        let cx = self.cultists[idx].x as i16;
        let cy = self.cultists[idx].y as i16;

        // 找到所有同类信徒的位置
        let same_tier_positions: Vec<(i16, i16)> = self
            .cultists
            .iter()
            .enumerate()
            .filter(|&(j, c)| j != idx && c.tier == tier)
            .map(|(_, c)| (c.x as i16, c.y as i16))
            .collect();

        if same_tier_positions.is_empty() {
            return None;
        }

        // 找到最近的同类信徒
        let (nearest_x, nearest_y) = same_tier_positions
            .iter()
            .min_by_key(|&&(sx, sy)| (sx - cx).abs() + (sy - cy).abs())
            .copied()?;

        // 在可用空位中选择最接近该同类信徒的方向
        empty_neighbors
            .iter()
            .min_by_key(|&&(dx, dy)| {
                let nx = cx + dx;
                let ny = cy + dy;
                (nx - nearest_x).abs() + (ny - nearest_y).abs()
            })
            .copied()
    }

    /// 渲染信徒到帧（替代现有的静态网格渲染）
    ///
    /// - 使用动画位置（area.x + cultist.x, area.y + cultist.y）
    /// - 应用闪烁亮度（各信徒独立相位）
    /// - 渲染 T5+ 光晕符号 `·`
    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let buf = frame.buffer_mut();

        // ── 0. 渲染三角形祭坛图案（居中）──
        let altar_art = [
            "△",
            "╱ ╲",
            "╱   ╲",
            "╱  ◈  ╲",
            "╱_______╲",
        ];
        let art_height = altar_art.len() as u16;
        let art_max_width = altar_art.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u16;
        let start_y = area.y + area.height.saturating_sub(art_height) / 2;
        let start_x = area.x + area.width.saturating_sub(art_max_width) / 2;

        for (i, line) in altar_art.iter().enumerate() {
            let y = start_y + i as u16;
            if y >= area.y + area.height {
                break;
            }
            let line_width = line.chars().count() as u16;
            let x = start_x + art_max_width.saturating_sub(line_width) / 2;
            for (j, ch) in line.chars().enumerate() {
                let cx = x + j as u16;
                if cx >= area.x + area.width {
                    break;
                }
                if ch != ' ' {
                    if let Some(cell) = buf.cell_mut(ratatui::layout::Position::new(cx, y)) {
                        cell.set_char(ch);
                        cell.set_fg(theme.toxic_purple);
                    }
                }
            }
        }

        // ── 1. 渲染 T5+ 光晕（先渲染，信徒符号覆盖在上层）──
        let halo_positions = self.halo_positions();
        let halo_color = scale_color(theme.locked, 0.5);
        for (hx, hy) in &halo_positions {
            let abs_x = area.x + hx;
            let abs_y = area.y + hy;
            if abs_x >= area.x + area.width || abs_y >= area.y + area.height {
                continue;
            }
            let cell = &mut buf[(abs_x, abs_y)];
            cell.set_char('·');
            cell.set_fg(halo_color);
        }

        // ── 2. 渲染信徒符号 ──
        for cultist in &self.cultists {
            let abs_x = area.x + cultist.x;
            let abs_y = area.y + cultist.y;
            if abs_x >= area.x + area.width || abs_y >= area.y + area.height {
                continue;
            }

            let symbol = cultist.tier.data().symbol;
            let base_color = tier_color(&cultist.tier, theme);

            // 闪烁亮度：用 flicker_phase 直接驱动（每个信徒相位不同）
            let brightness = (0.5 + 0.5 * cultist.flicker_phase.sin()).clamp(0.3, 1.0);
            let fg = scale_color(base_color, brightness);

            let cell = &mut buf[(abs_x, abs_y)];
            if let Some(ch) = symbol.chars().next() {
                cell.set_char(ch);
            }
            cell.set_fg(fg);
        }
    }
}

// ── 辅助函数 ──────────────────────────────────────────

/// 根据信徒等级返回对应颜色（与 tabs/altar.rs 中的配色一致）
fn tier_color(tier: &CultistTier, theme: &Theme) -> Color {
    match tier {
        CultistTier::T1 => theme.locked,
        CultistTier::T2 => theme.neon_green,
        CultistTier::T3 => theme.ghost_blue,
        CultistTier::T4 => theme.toxic_purple,
        CultistTier::T5 | CultistTier::T7 | CultistTier::T9 => theme.blood_red,
        CultistTier::T6 | CultistTier::T8 | CultistTier::T10 => theme.deep_gold,
    }
}

/// 按亮度系数缩放 RGB 颜色
/// 对于非 RGB 颜色，直接返回原色
fn scale_color(color: Color, brightness: f64) -> Color {
    match color {
        Color::Rgb(r, g, b) => {
            let r = (r as f64 * brightness).round().clamp(0.0, 255.0) as u8;
            let g = (g as f64 * brightness).round().clamp(0.0, 255.0) as u8;
            let b = (b as f64 * brightness).round().clamp(0.0, 255.0) as u8;
            Color::Rgb(r, g, b)
        }
        other => other,
    }
}
