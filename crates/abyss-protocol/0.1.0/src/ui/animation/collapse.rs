// 转生崩溃序列状态机

use ratatui::buffer::Buffer;
use ratatui::style::Color;

use crate::game::rebirth::RebirthSummary;
use crate::i18n::LocaleManager;
use crate::ui::format::format_number;

// ── 常量 ──────────────────────────────────────────────

/// 崩溃序列总时长（秒）
pub const COLLAPSE_TOTAL_DURATION: f64 = 4.0;

// ── 枚举 ──────────────────────────────────────────────

/// 崩溃序列阶段
#[derive(Debug, Clone, PartialEq)]
pub enum CollapsePhase {
    /// 未激活
    Inactive,
    /// 阶段 1: 红黑闪烁 (0-0.5s)
    Flicker { elapsed: f64 },
    /// 阶段 2: 径向 dissolve (0.5-1.5s)
    Dissolve { elapsed: f64 },
    /// 阶段 3: 崩溃文字 coalesce (1.5-2.5s)
    Coalesce { elapsed: f64 },
    /// 阶段 4: 结算数据淡入 (2.5-4.0s)
    Settlement { elapsed: f64, lines_visible: usize },
}

// ── 数据结构 ──────────────────────────────────────────

/// 崩溃序列状态
pub struct CollapseSequence {
    /// 当前阶段
    pub phase: CollapsePhase,
    /// 转生结算数据
    pub summary: Option<RebirthSummary>,
    /// 阶段 2 的 dissolve 快照（正常渲染的 Buffer 副本）
    pub screen_snapshot: Option<Buffer>,
}

impl CollapseSequence {
    pub fn new() -> Self {
        Self {
            phase: CollapsePhase::Inactive,
            summary: None,
            screen_snapshot: None,
        }
    }

    /// 触发崩溃序列
    pub fn trigger(&mut self, summary: RebirthSummary, current_screen: &Buffer) {
        self.summary = Some(summary);
        self.screen_snapshot = Some(current_screen.clone());
        self.phase = CollapsePhase::Flicker { elapsed: 0.0 };
    }

    /// 更新序列状态（推进阶段）
    pub fn update(&mut self, dt: f64) {
        self.phase = match self.phase {
            CollapsePhase::Inactive => CollapsePhase::Inactive,
            CollapsePhase::Flicker { elapsed } => {
                let elapsed = elapsed + dt;
                if elapsed >= 0.5 {
                    CollapsePhase::Dissolve { elapsed: 0.0 }
                } else {
                    CollapsePhase::Flicker { elapsed }
                }
            }
            CollapsePhase::Dissolve { elapsed } => {
                let elapsed = elapsed + dt;
                if elapsed >= 1.0 {
                    CollapsePhase::Coalesce { elapsed: 0.0 }
                } else {
                    CollapsePhase::Dissolve { elapsed }
                }
            }
            CollapsePhase::Coalesce { elapsed } => {
                let elapsed = elapsed + dt;
                if elapsed >= 1.0 {
                    CollapsePhase::Settlement {
                        elapsed: 0.0,
                        lines_visible: 0,
                    }
                } else {
                    CollapsePhase::Coalesce { elapsed }
                }
            }
            CollapsePhase::Settlement {
                elapsed,
                ..
            } => {
                let elapsed = elapsed + dt;
                if elapsed >= 1.5 {
                    self.summary = None;
                    self.screen_snapshot = None;
                    CollapsePhase::Inactive
                } else {
                    let lines_visible = (elapsed / 0.2) as usize;
                    CollapsePhase::Settlement {
                        elapsed,
                        lines_visible,
                    }
                }
            }
        };
    }

    /// 序列是否正在播放
    pub fn is_active(&self) -> bool {
        self.phase != CollapsePhase::Inactive
    }

    /// 序列是否阻止玩家输入
    pub fn blocks_input(&self) -> bool {
        self.phase != CollapsePhase::Inactive
    }

    /// 渲染当前阶段到帧
    /// 返回 true 表示序列仍在播放（阻止其他渲染）
    pub fn render(&self, frame: &mut ratatui::Frame, locale: &LocaleManager) -> bool {
        match &self.phase {
            CollapsePhase::Inactive => false,
            CollapsePhase::Flicker { elapsed } => {
                self.render_flicker(frame, *elapsed);
                true
            }
            CollapsePhase::Dissolve { elapsed } => {
                self.render_dissolve(frame, *elapsed);
                true
            }
            CollapsePhase::Coalesce { elapsed } => {
                self.render_coalesce(frame, *elapsed, locale);
                true
            }
            CollapsePhase::Settlement { elapsed, lines_visible } => {
                self.render_settlement(frame, *elapsed, *lines_visible, locale);
                true
            }
        }
    }

    /// Flicker 阶段：红黑交替闪烁 10Hz
    fn render_flicker(&self, frame: &mut ratatui::Frame, elapsed: f64) {
        let is_red = (elapsed * 10.0) as u32 % 2 == 0;
        let bg = if is_red {
            Color::Rgb(0xFF, 0x00, 0x33)
        } else {
            Color::Black
        };

        let buf = frame.buffer_mut();
        let area = buf.area;
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                let cell = &mut buf[(x, y)];
                cell.set_char(' ');
                cell.set_bg(bg);
                cell.set_fg(bg);
            }
        }
    }

    /// Dissolve 阶段：从屏幕快照径向 dissolve 消散
    fn render_dissolve(&self, frame: &mut ratatui::Frame, elapsed: f64) {
        let buf = frame.buffer_mut();
        let area = buf.area;
        let cx = area.x as f64 + area.width as f64 / 2.0;
        let cy = area.y as f64 + area.height as f64 / 2.0;
        let max_dist = ((cx * cx) + (cy * cy)).sqrt();
        // phase duration = 1.0s
        let progress = (elapsed / 1.0).clamp(0.0, 1.0);

        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                let dx = x as f64 - cx;
                let dy = y as f64 - cy;
                let dist = (dx * dx + dy * dy).sqrt();
                let norm_dist = if max_dist > 0.0 { dist / max_dist } else { 0.0 };

                if norm_dist > progress {
                    // 尚未消散：显示快照内容（如果有）
                    if let Some(snapshot) = &self.screen_snapshot {
                        let snap_area = snapshot.area;
                        if x >= snap_area.x
                            && x < snap_area.x + snap_area.width
                            && y >= snap_area.y
                            && y < snap_area.y + snap_area.height
                        {
                            let src = &snapshot[(x, y)];
                            let dst = &mut buf[(x, y)];
                            dst.set_char(src.symbol().chars().next().unwrap_or(' '));
                            dst.set_fg(src.fg);
                            dst.set_bg(src.bg);
                        }
                    }
                } else {
                    // 已消散：显示黑色
                    let cell = &mut buf[(x, y)];
                    cell.set_char(' ');
                    cell.set_bg(Color::Black);
                    cell.set_fg(Color::Black);
                }
            }
        }
    }

    /// Coalesce 阶段：纯黑屏中央逐字显现崩溃提示文本
    fn render_coalesce(
        &self,
        frame: &mut ratatui::Frame,
        elapsed: f64,
        locale: &LocaleManager,
    ) {
        let buf = frame.buffer_mut();
        let area = buf.area;

        // 先填充纯黑
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                let cell = &mut buf[(x, y)];
                cell.set_char(' ');
                cell.set_bg(Color::Black);
                cell.set_fg(Color::Black);
            }
        }

        let text = locale.t("collapse.consciousness_collapsing");
        let chars: Vec<char> = text.chars().collect();
        let total_chars = chars.len();
        if total_chars == 0 {
            return;
        }

        // phase duration = 1.0s，按时间比例显示字符数
        let visible_count = ((elapsed / 1.0) * total_chars as f64).ceil() as usize;
        let visible_count = visible_count.min(total_chars);

        // 计算文本在屏幕中央的起始位置
        // 中文字符占 2 列宽，需要计算实际显示宽度
        let display_width: usize = chars.iter().take(total_chars).map(|c| {
            if c.is_ascii() { 1 } else { 2 }
        }).sum();
        let start_x = area.x + area.width.saturating_sub(display_width as u16) / 2;
        let start_y = area.y + area.height / 2;

        if start_y >= area.y + area.height {
            return;
        }

        let blood_red = Color::Rgb(0xFF, 0x00, 0x33);
        let mut cur_x = start_x;
        for ch in chars.iter().take(visible_count) {
            let char_width: u16 = if ch.is_ascii() { 1 } else { 2 };
            if cur_x + char_width <= area.x + area.width {
                let cell = &mut buf[(cur_x, start_y)];
                cell.set_char(*ch);
                cell.set_fg(blood_red);
                cell.set_bg(Color::Black);
            }
            cur_x += char_width;
        }
    }

    /// Settlement 阶段：转生结算数据逐行淡入
    fn render_settlement(
        &self,
        frame: &mut ratatui::Frame,
        _elapsed: f64,
        lines_visible: usize,
        locale: &LocaleManager,
    ) {
        let buf = frame.buffer_mut();
        let area = buf.area;

        // 先填充纯黑
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                let cell = &mut buf[(x, y)];
                cell.set_char(' ');
                cell.set_bg(Color::Black);
                cell.set_fg(Color::Black);
            }
        }

        let neon_green = Color::Rgb(0x00, 0xFF, 0x41);

        // 构建结算行
        let lines = if let Some(summary) = &self.summary {
            let duration_secs = summary.cycle_duration_secs;
            let mins = duration_secs / 60;
            let secs = duration_secs % 60;
            let duration_str = format!("{}m {}s", mins, secs);

            vec![
                locale.t("rebirth.rebirth_title").to_string(),
                String::new(),
                format!(
                    "{}: {}",
                    locale.t("rebirth.cycle_duration"),
                    duration_str
                ),
                format!(
                    "{}: {}",
                    locale.t("rebirth.whispers_harvested"),
                    format_number(summary.whispers_harvested)
                ),
                format!(
                    "{}: {}",
                    locale.t("rebirth.nodes_constructed"),
                    summary.nodes_constructed
                ),
                format!(
                    "{}: {}/s",
                    locale.t("rebirth.peak_production"),
                    format_number(summary.peak_production)
                ),
                format!(
                    "{}: {}",
                    locale.t("rebirth.san_repairs"),
                    summary.san_repairs
                ),
                format!(
                    "{}: {}",
                    locale.t("rebirth.mutations_witnessed"),
                    summary.mutations_witnessed
                ),
                String::new(),
                format!(
                    "{}: +{}",
                    locale.t("rebirth.truths_gained"),
                    summary.truths_gained
                ),
                format!(
                    "{}: {}",
                    locale.t("rebirth.total_truths"),
                    summary.total_truths_after
                ),
            ]
        } else {
            vec![locale.t("rebirth.rebirth_title").to_string()]
        };

        let total_lines = lines.len();
        let visible = lines_visible.min(total_lines);

        // 垂直居中
        let start_y = area.y + area.height.saturating_sub(total_lines as u16) / 2;

        for (i, line) in lines.iter().take(visible).enumerate() {
            let y = start_y + i as u16;
            if y >= area.y + area.height {
                break;
            }

            // 水平居中：计算显示宽度
            let display_width: usize = line.chars().map(|c| {
                if c.is_ascii() { 1 } else { 2 }
            }).sum();
            let start_x = area.x + area.width.saturating_sub(display_width as u16) / 2;

            let mut cur_x = start_x;
            for ch in line.chars() {
                let char_width: u16 = if ch.is_ascii() { 1 } else { 2 };
                if cur_x + char_width <= area.x + area.width {
                    let cell = &mut buf[(cur_x, y)];
                    cell.set_char(ch);
                    cell.set_fg(neon_green);
                    cell.set_bg(Color::Black);
                }
                cur_x += char_width;
            }
        }
    }
}
