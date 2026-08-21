// Token 心电图

use std::collections::VecDeque;

use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::Frame;

use crate::ui::theme::Theme;

// ── 常量 ──────────────────────────────────────────────

/// 心电图变暗灰阈值（秒）
pub const HEARTBEAT_IDLE_THRESHOLD: u32 = 10;
/// 心电图闪白阈值倍数
pub const HEARTBEAT_FLASH_MULTIPLIER: f64 = 3.0;
/// 闪白持续时间（秒）
const FLASH_DURATION: f64 = 0.5;

/// Unicode 方块字符（从低到高）
const BLOCK_CHARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

// ── 数据结构 ──────────────────────────────────────────

/// Token 心电图状态
pub struct TokenHeartbeat {
    /// 每秒 token 流入量的环形缓冲区
    pub history: VecDeque<u32>,
    /// 最大历史长度（等于显示宽度）
    pub max_len: usize,
    /// 当前秒累计 token 数
    pub current_second_tokens: u32,
    /// 秒计时器
    pub second_timer: f64,
    /// 连续无 token 秒数
    pub idle_seconds: u32,
    /// 正常水平（用于判断 3 倍闪白阈值）
    pub normal_level: f64,
    /// 闪白位置和剩余时间
    pub flash_positions: Vec<(usize, f64)>,
}

impl TokenHeartbeat {
    pub fn new(width: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(width),
            max_len: width,
            current_second_tokens: 0,
            second_timer: 0.0,
            idle_seconds: 0,
            normal_level: 0.0,
            flash_positions: Vec::new(),
        }
    }

    /// 记录 token 流入事件
    pub fn record_tokens(&mut self, count: u32) {
        self.current_second_tokens = self.current_second_tokens.saturating_add(count);
    }

    /// 更新计时器，推进秒
    pub fn update(&mut self, dt: f64) {
        // 衰减闪白位置计时器
        for flash in &mut self.flash_positions {
            flash.1 -= dt;
        }
        self.flash_positions.retain(|&(_, remaining)| remaining > 0.0);

        self.second_timer += dt;

        // 每满 1 秒推入一次历史
        while self.second_timer >= 1.0 {
            self.second_timer -= 1.0;

            let value = self.current_second_tokens;

            // 推入历史缓冲区
            self.history.push_back(value);
            // 环形缓冲区：超出 max_len 时移除最旧数据
            while self.history.len() > self.max_len {
                self.history.pop_front();
            }

            // 更新 idle_seconds
            if value == 0 {
                self.idle_seconds = self.idle_seconds.saturating_add(1);
            } else {
                self.idle_seconds = 0;
            }

            // 更新 normal_level：指数移动平均
            self.normal_level = self.normal_level * 0.9 + value as f64 * 0.1;

            // 检查是否需要闪白（超过 3 倍正常水平）
            if self.normal_level > 0.0
                && value as f64 > HEARTBEAT_FLASH_MULTIPLIER * self.normal_level
            {
                let index = self.history.len().saturating_sub(1);
                self.flash_positions.push((index, FLASH_DURATION));
            }

            // 重置当前秒累计
            self.current_second_tokens = 0;
        }
    }

    /// 将 token 数量映射到 Unicode 方块字符 ▁▂▃▄▅▆▇█
    /// 单调不减：a <= b 时 token_to_block(a, max) <= token_to_block(b, max)
    pub fn token_to_block(count: u32, max_count: u32) -> char {
        if max_count == 0 {
            return BLOCK_CHARS[0];
        }
        let ratio = count as f64 / max_count as f64;
        // 映射到 0..7 的索引，clamp 确保不越界
        let index = (ratio * 7.0).round() as usize;
        let index = index.min(7);
        BLOCK_CHARS[index]
    }

    /// 获取指定位置的柱体颜色
    /// 正常: 荧光绿 #00FF41, 闪白: 白色 #FFFFFF, 空闲 ≥ 10s: 暗灰 #555555
    pub fn bar_color(&self, index: usize) -> Color {
        // 空闲 ≥ 10s：暗灰
        if self.idle_seconds >= HEARTBEAT_IDLE_THRESHOLD {
            return Color::Rgb(0x55, 0x55, 0x55);
        }

        // 检查闪白位置
        if self.flash_positions.iter().any(|&(pos, _)| pos == index) {
            return Color::Rgb(0xFF, 0xFF, 0xFF);
        }

        // 检查该位置的值是否超过 3 倍正常水平
        if let Some(&value) = self.history.get(index) {
            if self.normal_level > 0.0
                && value as f64 > HEARTBEAT_FLASH_MULTIPLIER * self.normal_level
            {
                return Color::Rgb(0xFF, 0xFF, 0xFF);
            }
        }

        // 正常：荧光绿
        Color::Rgb(0x00, 0xFF, 0x41)
    }

    /// 渲染心电图到指定区域
    pub fn render(&self, frame: &mut Frame, area: Rect, _theme: &Theme) {
        if area.width == 0 || area.height == 0 || self.history.is_empty() {
            return;
        }

        let buf = frame.buffer_mut();
        let width = area.width as usize;
        let history_len = self.history.len();

        // 找到历史中的最大值，用于归一化
        let max_count = self.history.iter().copied().max().unwrap_or(0);

        // 右对齐：newest data on the right
        let offset = if history_len < width {
            width - history_len
        } else {
            0
        };

        // 渲染底行（方块字符向上生长）
        let render_y = area.y + area.height - 1;

        // 只渲染 history 中能显示的部分（超出宽度时截取最新的）
        let start_idx = history_len.saturating_sub(width);

        for (i, &count) in self.history.iter().skip(start_idx).enumerate() {
            let x = area.x + (offset + i) as u16;

            // 边界检查
            if x >= area.x + area.width || render_y >= area.y + area.height {
                continue;
            }

            let block_char = Self::token_to_block(count, max_count);
            let history_index = start_idx + i;
            let color = self.bar_color(history_index);

            let cell = &mut buf[(x, render_y)];
            cell.set_char(block_char);
            cell.set_fg(color);
        }
    }
}
