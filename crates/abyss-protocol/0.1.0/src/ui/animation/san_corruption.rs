// SAN 腐蚀后处理

use rand::Rng;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::Color;

// ── 常量 ──────────────────────────────────────────────

/// 边框闪烁间隔（秒），60-79 级
pub const BORDER_FLICKER_INTERVAL: f64 = 30.0;
/// 边框闪烁持续时间（秒）
pub const BORDER_FLICKER_DURATION: f64 = 0.2;
/// 符文替换比例，40-59 级
pub const RUNE_REPLACE_PCT: f64 = 0.01;
/// 符文位置刷新间隔（秒）
pub const RUNE_REFRESH_INTERVAL: f64 = 2.0;
/// 乱码替换比例，20-39 级
pub const GLITCH_REPLACE_PCT: f64 = 0.03;
/// Zalgo 替换比例，1-19 级
pub const ZALGO_REPLACE_PCT: f64 = 0.05;
/// 边框跳动间隔（秒），20-39 级
pub const BORDER_JUMP_INTERVAL: f64 = 0.5;

/// 符文字符集（用于 40-59 级替换）
pub const RUNE_CHARS: &[char] = &[
    'ᚠ', 'ᚢ', 'ᚦ', 'ᚨ', 'ᚱ', 'ᚲ', 'ᚷ', 'ᚹ', 'ᚺ', 'ᚾ', 'ᛁ', 'ᛃ', 'ᛇ',
    'ᛈ', 'ᛉ', 'ᛊ', 'ᛏ', 'ᛒ', 'ᛖ', 'ᛗ', 'ᛚ', 'ᛜ', 'ᛝ', 'ᛞ', 'ᛟ',
];

/// 乱码字符集（用于 20-39 级替换）
pub const GLITCH_CHARS: &[char] = &[
    '░', '▒', '▓', '█', '▄', '▀', '│', '┤', '╡', '╢', '╖', '╕', '╣', '║',
    '╗', '╝', '╜', '╛', '┐', '└', '┴', '┬', '├', '─', '┼', '╞', '╟', '╚',
    '╔', '╩', '╦', '╠', '═', '╬',
];

/// Zalgo 组合字符（用于 1-19 级叠加，U+0300-U+036F）
pub const ZALGO_COMBINING: &[char] = &[
    '\u{0300}', '\u{0301}', '\u{0302}', '\u{0303}', '\u{0304}', '\u{0305}',
    '\u{0306}', '\u{0307}', '\u{0308}', '\u{0309}', '\u{030A}', '\u{030B}',
    '\u{030C}', '\u{030D}', '\u{030E}', '\u{030F}', '\u{0310}', '\u{0311}',
    '\u{0312}', '\u{0313}', '\u{0314}', '\u{0315}', '\u{0316}', '\u{0317}',
    '\u{0318}', '\u{0319}', '\u{031A}', '\u{031B}', '\u{031C}', '\u{031D}',
    '\u{031E}', '\u{031F}', '\u{0320}', '\u{0321}', '\u{0322}', '\u{0323}',
    '\u{0324}', '\u{0325}', '\u{0326}', '\u{0327}', '\u{0328}', '\u{0329}',
    '\u{032A}', '\u{032B}', '\u{032C}', '\u{032D}', '\u{032E}', '\u{032F}',
    '\u{0330}', '\u{0331}', '\u{0332}', '\u{0333}', '\u{0334}', '\u{0335}',
    '\u{0336}', '\u{0337}', '\u{0338}', '\u{0339}', '\u{033A}', '\u{033B}',
    '\u{033C}', '\u{033D}', '\u{033E}', '\u{033F}', '\u{0340}', '\u{0341}',
    '\u{0342}', '\u{0343}', '\u{0344}', '\u{0345}', '\u{0346}', '\u{0347}',
    '\u{0348}', '\u{0349}', '\u{034A}', '\u{034B}', '\u{034C}', '\u{034D}',
    '\u{034E}', '\u{034F}', '\u{0350}', '\u{0351}', '\u{0352}', '\u{0353}',
    '\u{0354}', '\u{0355}', '\u{0356}', '\u{0357}', '\u{0358}', '\u{0359}',
    '\u{035A}', '\u{035B}', '\u{035C}', '\u{035D}', '\u{035E}', '\u{035F}',
    '\u{0360}', '\u{0361}', '\u{0362}', '\u{0363}', '\u{0364}', '\u{0365}',
    '\u{0366}', '\u{0367}', '\u{0368}', '\u{0369}', '\u{036A}', '\u{036B}',
    '\u{036C}', '\u{036D}', '\u{036E}', '\u{036F}',
];

// ── 枚举 ──────────────────────────────────────────────

/// SAN 腐蚀级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SanCorruptionLevel {
    /// 80-100: 无腐蚀
    None,
    /// 60-79: 轻微（边框闪烁）
    Mild,
    /// 40-59: 中度（符文替换 + 边框变色）
    Medium,
    /// 20-39: 严重（乱码 + 边框跳动 + 行偏移）
    Severe,
    /// 1-19: 极端（Zalgo + 触手 + 紫色偏移）
    Extreme,
}

impl SanCorruptionLevel {
    /// 从 SAN 值计算腐蚀级别
    pub fn from_san(san: f64) -> Self {
        if san >= 80.0 {
            SanCorruptionLevel::None
        } else if san >= 60.0 {
            SanCorruptionLevel::Mild
        } else if san >= 40.0 {
            SanCorruptionLevel::Medium
        } else if san >= 20.0 {
            SanCorruptionLevel::Severe
        } else {
            SanCorruptionLevel::Extreme
        }
    }
}

// ── 数据结构 ──────────────────────────────────────────

/// SAN 腐蚀效果状态
pub struct SanCorruptionState {
    /// 当前 SAN 级别
    pub current_level: SanCorruptionLevel,
    /// 边框闪烁计时器（60-79 级）
    pub border_flicker_timer: f64,
    /// 边框闪烁是否激活
    pub border_flickering: bool,
    /// 符文替换位置缓存（40-59 级）
    pub rune_positions: Vec<(u16, u16)>,
    /// 符文刷新计时器
    pub rune_refresh_timer: f64,
    /// 乱码位置缓存（20-39 级）
    pub glitch_positions: Vec<(u16, u16)>,
    /// 边框跳动计时器（20-39 级）
    pub border_jump_timer: f64,
    /// 行偏移缓存（20-39 级）
    pub line_offsets: Vec<(u16, i8)>,
    /// Zalgo 位置缓存（1-19 级）
    pub zalgo_positions: Vec<(u16, u16)>,
}

impl SanCorruptionState {
    pub fn new() -> Self {
        Self {
            current_level: SanCorruptionLevel::None,
            border_flicker_timer: 0.0,
            border_flickering: false,
            rune_positions: Vec::new(),
            rune_refresh_timer: 0.0,
            glitch_positions: Vec::new(),
            border_jump_timer: 0.0,
            line_offsets: Vec::new(),
            zalgo_positions: Vec::new(),
        }
    }

    /// 更新腐蚀状态（计时器、位置缓存）
    pub fn update(
        &mut self,
        dt: f64,
        san: f64,
        screen_area: Rect,
        rng: &mut impl Rng,
    ) {
        let new_level = SanCorruptionLevel::from_san(san);
        let old_level = self.current_level;
        self.current_level = new_level;

        // 级别变轻时，清除不再适用的效果缓存
        if new_level < old_level {
            if new_level < SanCorruptionLevel::Extreme {
                self.zalgo_positions.clear();
            }
            if new_level < SanCorruptionLevel::Severe {
                self.glitch_positions.clear();
                self.line_offsets.clear();
                self.border_jump_timer = 0.0;
            }
            if new_level < SanCorruptionLevel::Medium {
                self.rune_positions.clear();
                self.rune_refresh_timer = 0.0;
            }
            if new_level < SanCorruptionLevel::Mild {
                self.border_flicker_timer = 0.0;
                self.border_flickering = false;
            }
        }

        let total_cells = (screen_area.width as usize) * (screen_area.height as usize);
        if total_cells == 0 {
            return;
        }

        // Mild (60-79): 边框闪烁计时器
        if new_level >= SanCorruptionLevel::Mild {
            self.border_flicker_timer += dt;
            if self.border_flickering {
                // 闪烁中，检查是否超过持续时间
                if self.border_flicker_timer >= BORDER_FLICKER_DURATION {
                    self.border_flickering = false;
                    self.border_flicker_timer = 0.0;
                }
            } else {
                // 等待下次闪烁
                if self.border_flicker_timer >= BORDER_FLICKER_INTERVAL {
                    self.border_flickering = true;
                    self.border_flicker_timer = 0.0;
                }
            }
        }

        // Medium (40-59): 符文刷新计时器
        if new_level >= SanCorruptionLevel::Medium {
            self.rune_refresh_timer += dt;
            if self.rune_refresh_timer >= RUNE_REFRESH_INTERVAL || self.rune_positions.is_empty() {
                self.rune_refresh_timer = 0.0;
                Self::gen_random_positions(
                    &mut self.rune_positions,
                    total_cells,
                    RUNE_REPLACE_PCT,
                    screen_area,
                    rng,
                );
            }
        }

        // Severe (20-39): 乱码位置、边框跳动、行偏移
        if new_level >= SanCorruptionLevel::Severe {
            // 乱码位置每帧刷新（随机性更强）
            Self::gen_random_positions(
                &mut self.glitch_positions,
                total_cells,
                GLITCH_REPLACE_PCT,
                screen_area,
                rng,
            );

            // 边框跳动计时器
            self.border_jump_timer += dt;
            if self.border_jump_timer >= BORDER_JUMP_INTERVAL {
                self.border_jump_timer = 0.0;
            }

            // 行偏移缓存（随机选择少量行进行偏移）
            let num_lines = (screen_area.height as usize).max(1);
            let offset_count = (num_lines as f64 * 0.03).ceil() as usize; // 约 3% 的行
            self.line_offsets.clear();
            for _ in 0..offset_count {
                let y = rng.gen_range(0..screen_area.height);
                let offset: i8 = if rng.gen_bool(0.5) { 1 } else { -1 };
                self.line_offsets.push((y, offset));
            }
        }

        // Extreme (1-19): Zalgo 位置
        if new_level >= SanCorruptionLevel::Extreme {
            Self::gen_random_positions(
                &mut self.zalgo_positions,
                total_cells,
                ZALGO_REPLACE_PCT,
                screen_area,
                rng,
            );
        }
    }

    /// 生成随机位置列表
    fn gen_random_positions(
        positions: &mut Vec<(u16, u16)>,
        total_cells: usize,
        pct: f64,
        area: Rect,
        rng: &mut impl Rng,
    ) {
        let count = ((total_cells as f64) * pct).round() as usize;
        positions.clear();
        positions.reserve(count);
        for _ in 0..count {
            let x = rng.gen_range(area.x..area.x.saturating_add(area.width));
            let y = rng.gen_range(area.y..area.y.saturating_add(area.height));
            positions.push((x, y));
        }
    }

    /// 将腐蚀效果应用到 Buffer（后处理）
    /// 按级别累积叠加：Extreme 包含所有效果，Severe 包含 Mild+Medium+Severe，以此类推
    pub fn apply(&self, buf: &mut Buffer, area: Rect) {
        if self.current_level == SanCorruptionLevel::None {
            return;
        }

        let x_max = area.x.saturating_add(area.width);
        let y_max = area.y.saturating_add(area.height);

        // ── Mild (60-79): 边框闪烁 ──
        if self.current_level >= SanCorruptionLevel::Mild && self.border_flickering {
            Self::apply_border_dim(buf, area, x_max, y_max, Color::DarkGray);
        }

        // ── Medium (40-59): 符文替换 + 边框变色（暗橙色）──
        if self.current_level >= SanCorruptionLevel::Medium {
            // 符文替换
            for &(x, y) in &self.rune_positions {
                if x >= area.x && x < x_max && y >= area.y && y < y_max {
                    let ch = RUNE_CHARS[((x as usize) * 31 + (y as usize) * 37) % RUNE_CHARS.len()];
                    if let Some(cell) = buf.cell_mut(Position::new(x, y)) {
                        cell.set_symbol(&ch.to_string());
                    }
                }
            }
            // 边框变色：暗橙色
            Self::apply_border_dim(buf, area, x_max, y_max, Color::Rgb(180, 100, 0));
        }

        // ── Severe (20-39): 乱码替换 + 边框跳动 + 行偏移 ──
        if self.current_level >= SanCorruptionLevel::Severe {
            // 乱码替换
            for &(x, y) in &self.glitch_positions {
                if x >= area.x && x < x_max && y >= area.y && y < y_max {
                    let ch = GLITCH_CHARS[((x as usize) * 53 + (y as usize) * 17) % GLITCH_CHARS.len()];
                    if let Some(cell) = buf.cell_mut(Position::new(x, y)) {
                        cell.set_symbol(&ch.to_string());
                    }
                }
            }

            // 边框跳动：计时器前半段时替换少量边框字符
            if self.border_jump_timer < BORDER_JUMP_INTERVAL / 2.0 {
                let border_chars = ['│', '┤', '├', '─', '┐', '└', '┘', '┌'];
                // 只替换四个角
                for &(x, y) in &[
                    (area.x, area.y),
                    (x_max.saturating_sub(1), area.y),
                    (area.x, y_max.saturating_sub(1)),
                    (x_max.saturating_sub(1), y_max.saturating_sub(1)),
                ] {
                    if x < x_max && y < y_max {
                        let idx = ((x as usize) * 7 + (y as usize) * 13) % border_chars.len();
                        if let Some(cell) = buf.cell_mut(Position::new(x, y)) {
                            cell.set_symbol(&border_chars[idx].to_string());
                        }
                    }
                }
            }

            // 行偏移：将指定行的内容水平偏移 1 个字符
            for &(row_y, offset) in &self.line_offsets {
                let abs_y = area.y.saturating_add(row_y);
                if abs_y >= y_max {
                    continue;
                }
                if offset > 0 {
                    // 向右偏移：从右往左复制
                    for x in (area.x + 1..x_max).rev() {
                        let src_x = x - 1;
                        let sym = buf
                            .cell_mut(Position::new(src_x, abs_y))
                            .map(|c| c.symbol().to_string());
                        let fg = buf
                            .cell_mut(Position::new(src_x, abs_y))
                            .map(|c| c.fg);
                        if let (Some(sym), Some(fg)) = (sym, fg) {
                            if let Some(cell) = buf.cell_mut(Position::new(x, abs_y)) {
                                cell.set_symbol(&sym);
                                cell.set_fg(fg);
                            }
                        }
                    }
                    // 左端填空
                    if let Some(cell) = buf.cell_mut(Position::new(area.x, abs_y)) {
                        cell.set_symbol(" ");
                    }
                } else {
                    // 向左偏移：从左往右复制
                    for x in area.x..x_max.saturating_sub(1) {
                        let src_x = x + 1;
                        let sym = buf
                            .cell_mut(Position::new(src_x, abs_y))
                            .map(|c| c.symbol().to_string());
                        let fg = buf
                            .cell_mut(Position::new(src_x, abs_y))
                            .map(|c| c.fg);
                        if let (Some(sym), Some(fg)) = (sym, fg) {
                            if let Some(cell) = buf.cell_mut(Position::new(x, abs_y)) {
                                cell.set_symbol(&sym);
                                cell.set_fg(fg);
                            }
                        }
                    }
                    // 右端填空
                    if x_max > 0 {
                        if let Some(cell) = buf.cell_mut(Position::new(x_max - 1, abs_y)) {
                            cell.set_symbol(" ");
                        }
                    }
                }
            }
        }

        // ── Extreme (1-19): Zalgo 化 + 边框紫色偏移 ──
        if self.current_level >= SanCorruptionLevel::Extreme {
            // Zalgo 化：在指定位置的字符后追加组合字符
            for &(x, y) in &self.zalgo_positions {
                if x >= area.x && x < x_max && y >= area.y && y < y_max {
                    if let Some(cell) = buf.cell_mut(Position::new(x, y)) {
                        let mut sym = cell.symbol().to_string();
                        // 追加 1-2 个 Zalgo 组合字符（减少数量保持可读性）
                        let count = ((x as usize + y as usize) % 2) + 1;
                        for i in 0..count {
                            let idx = ((x as usize) * 41 + (y as usize) * 59 + i * 23) % ZALGO_COMBINING.len();
                            sym.push(ZALGO_COMBINING[idx]);
                        }
                        cell.set_symbol(&sym);
                    }
                }
            }

            // 紫色偏移：只影响边框，不影响内容可读性
            Self::apply_border_dim(buf, area, x_max, y_max, Color::Rgb(160, 50, 200));
        }
    }

    /// 将边框 cell 的前景色设为指定颜色
    fn apply_border_dim(buf: &mut Buffer, area: Rect, x_max: u16, y_max: u16, color: Color) {
        // 顶边和底边
        for x in area.x..x_max {
            if let Some(cell) = buf.cell_mut(Position::new(x, area.y)) {
                cell.set_fg(color);
            }
            if y_max > 0 {
                if let Some(cell) = buf.cell_mut(Position::new(x, y_max - 1)) {
                    cell.set_fg(color);
                }
            }
        }
        // 左边和右边
        for y in area.y..y_max {
            if let Some(cell) = buf.cell_mut(Position::new(area.x, y)) {
                cell.set_fg(color);
            }
            if x_max > 0 {
                if let Some(cell) = buf.cell_mut(Position::new(x_max - 1, y)) {
                    cell.set_fg(color);
                }
            }
        }
    }

    /// 对颜色施加紫色偏移
    fn purple_tint(color: Color) -> Color {
        match color {
            Color::Rgb(r, g, b) => {
                // 增加红蓝分量，降低绿色分量
                let r = r.saturating_add(30).min(255);
                let g = g.saturating_sub(20);
                let b = b.saturating_add(50).min(255);
                Color::Rgb(r, g, b)
            }
            _ => Color::Rgb(160, 50, 200), // 非 RGB 颜色直接替换为紫色
        }
    }
}
