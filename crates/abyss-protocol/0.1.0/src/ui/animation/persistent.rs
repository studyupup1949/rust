// 常驻动画：碎语平滑滚动、产出脉动、SAN 渐变、深渊呼吸、边框微光

use std::f64::consts::TAU;

use ratatui::style::Color;

// ── 常量 ──────────────────────────────────────────────

/// 帧间隔（秒），10 FPS
pub const FRAME_DT: f64 = 0.1;
/// 产出脉动频率 (Hz)
pub const PRODUCTION_PULSE_FREQ: f64 = 2.0;
/// 产出脉动最低亮度
pub const PRODUCTION_PULSE_MIN: f64 = 0.7;
/// 深渊呼吸频率 (Hz)
pub const ABYSS_BREATH_FREQ: f64 = 0.5;
/// 边框微光频率 (Hz)
pub const BORDER_GLOW_FREQ: f64 = 0.3;
/// 产出为零时的固定暗灰系数
pub const PRODUCTION_ZERO_DIM: f64 = 0.3;
/// 深渊呼吸最低亮度
pub const ABYSS_BREATH_MIN: f64 = 0.4;
/// 边框微光最大色相偏移（度）
pub const BORDER_HUE_MAX_OFFSET: f64 = 5.0;
/// 碎语插值速率（每秒趋近比例）
pub const WHISPER_LERP_RATE: f64 = 8.0;
/// SAN 插值速率（每秒趋近比例）
pub const SAN_LERP_RATE: f64 = 6.0;

// ── 数据结构 ──────────────────────────────────────────

/// 常驻动画状态
pub struct PersistentAnimations {
    /// 碎语平滑显示值（插值目标为实际碎语值）
    pub whisper_display_value: f64,
    /// 产出脉动相位（0.0 - 2π）
    pub production_pulse_phase: f64,
    /// 深渊呼吸相位（0.0 - 2π）
    pub abyss_breath_phase: f64,
    /// 边框微光色相偏移相位
    pub border_hue_phase: f64,
    /// SAN 进度条显示值（平滑插值）
    pub san_display_value: f64,
}

impl PersistentAnimations {
    pub fn new() -> Self {
        Self {
            whisper_display_value: 0.0,
            production_pulse_phase: 0.0,
            abyss_breath_phase: 0.0,
            border_hue_phase: 0.0,
            san_display_value: 100.0,
        }
    }

    /// 更新所有常驻动画状态
    /// dt: 帧间隔（秒），actual_whispers: 实际碎语值，actual_san: 实际 SAN 值
    pub fn update(&mut self, dt: f64, actual_whispers: f64, actual_san: f64) {
        // ── 碎语平滑插值 ──
        // 如果显示值超过实际值（例如消费后），立即 snap 到实际值
        if self.whisper_display_value > actual_whispers {
            self.whisper_display_value = actual_whispers;
        } else if self.whisper_display_value < actual_whispers {
            // 线性插值趋近: display += (actual - display) * rate * dt
            let delta = (actual_whispers - self.whisper_display_value) * WHISPER_LERP_RATE * dt;
            // 确保至少前进一个最小步长，避免在极小差距时停滞
            let delta = delta.max(dt * 0.01);
            self.whisper_display_value += delta;
            // clamp: 不超过实际值
            if self.whisper_display_value > actual_whispers {
                self.whisper_display_value = actual_whispers;
            }
        }
        // 处理 NaN/Infinity
        if !self.whisper_display_value.is_finite() {
            self.whisper_display_value = actual_whispers;
        }

        // ── SAN 平滑插值 ──
        // SAN 可以上升也可以下降，双向插值
        let san_diff = actual_san - self.san_display_value;
        if san_diff.abs() > f64::EPSILON {
            let san_delta = san_diff * SAN_LERP_RATE * dt;
            // 确保至少前进一个最小步长
            let san_delta = if san_diff > 0.0 {
                san_delta.max(dt * 0.01)
            } else {
                san_delta.min(-dt * 0.01)
            };
            self.san_display_value += san_delta;
            // clamp: 不超过目标方向
            if san_diff > 0.0 && self.san_display_value > actual_san {
                self.san_display_value = actual_san;
            } else if san_diff < 0.0 && self.san_display_value < actual_san {
                self.san_display_value = actual_san;
            }
        }
        // 处理 NaN/Infinity
        if !self.san_display_value.is_finite() {
            self.san_display_value = actual_san;
        }

        // ── 产出脉动相位推进 ──
        self.production_pulse_phase += TAU * PRODUCTION_PULSE_FREQ * dt;
        self.production_pulse_phase %= TAU;

        // ── 深渊呼吸相位推进 ──
        self.abyss_breath_phase += TAU * ABYSS_BREATH_FREQ * dt;
        self.abyss_breath_phase %= TAU;

        // ── 边框微光相位推进 ──
        self.border_hue_phase += TAU * BORDER_GLOW_FREQ * dt;
        self.border_hue_phase %= TAU;
    }

    /// 返回碎语平滑显示值
    /// 确保 display_value <= actual_value（由 update 保证）
    pub fn whisper_display(&self) -> f64 {
        self.whisper_display_value
    }

    /// 计算产出脉动亮度系数
    /// 正弦波 2Hz，范围 [PRODUCTION_PULSE_MIN, 1.0] = [0.7, 1.0]
    /// 产出为零时返回固定暗灰系数
    pub fn production_brightness(&self, production_per_sec: f64) -> f64 {
        if production_per_sec == 0.0 {
            return PRODUCTION_ZERO_DIM;
        }
        // min + (1.0 - min) * (0.5 + 0.5 * sin(phase))
        // 当 sin = -1 时结果 = min, 当 sin = 1 时结果 = 1.0
        PRODUCTION_PULSE_MIN
            + (1.0 - PRODUCTION_PULSE_MIN) * (0.5 + 0.5 * self.production_pulse_phase.sin())
    }

    /// 计算深渊呼吸亮度系数
    /// 正弦波 0.5Hz，范围 [ABYSS_BREATH_MIN, 1.0] = [0.4, 1.0]
    pub fn abyss_breath_brightness(&self) -> f64 {
        ABYSS_BREATH_MIN
            + (1.0 - ABYSS_BREATH_MIN) * (0.5 + 0.5 * self.abyss_breath_phase.sin())
    }

    /// 计算边框微光色相偏移量（度）
    /// 正弦波 0.3Hz，范围 [-BORDER_HUE_MAX_OFFSET, +BORDER_HUE_MAX_OFFSET] = [-5.0, +5.0]
    pub fn border_hue_offset(&self) -> f64 {
        BORDER_HUE_MAX_OFFSET * self.border_hue_phase.sin()
    }

    /// 计算 SAN 进度条的平滑插值颜色
    pub fn san_bar_color(&self) -> Color {
        interpolate_san_color(self.san_display_value)
    }
}

// ── SAN 颜色插值 ─────────────────────────────────────

/// SAN 值到颜色的连续插值
/// 100: #00FF41 (绿) → 80: #AAFF00 (黄绿) → 60: #FFAA00 (橙)
/// → 40: #FF0033 (血红) → 20: #BF00FF (毒紫)
pub fn interpolate_san_color(san: f64) -> Color {
    let san = san.clamp(0.0, 100.0);

    // 关键色定义 (san_value, r, g, b)
    let stops: [(f64, u8, u8, u8); 5] = [
        (100.0, 0x00, 0xFF, 0x41), // 荧光绿
        (80.0, 0xAA, 0xFF, 0x00),  // 黄绿
        (60.0, 0xFF, 0xAA, 0x00),  // 橙
        (40.0, 0xFF, 0x00, 0x33),  // 血红
        (20.0, 0xBF, 0x00, 0xFF),  // 毒紫
    ];

    // SAN >= 100 返回第一个关键色
    if san >= stops[0].0 {
        return Color::Rgb(stops[0].1, stops[0].2, stops[0].3);
    }
    // SAN <= 20 返回最后一个关键色
    if san <= stops[stops.len() - 1].0 {
        return Color::Rgb(
            stops[stops.len() - 1].1,
            stops[stops.len() - 1].2,
            stops[stops.len() - 1].3,
        );
    }

    // 找到 san 所在的两个关键色之间
    for i in 0..stops.len() - 1 {
        let (s_high, r1, g1, b1) = stops[i];
        let (s_low, r2, g2, b2) = stops[i + 1];
        if san <= s_high && san >= s_low {
            // t=0 对应 s_high（上界色），t=1 对应 s_low（下界色）
            let t = (s_high - san) / (s_high - s_low);
            let r = (r1 as f64 + (r2 as f64 - r1 as f64) * t).round() as u8;
            let g = (g1 as f64 + (g2 as f64 - g1 as f64) * t).round() as u8;
            let b = (b1 as f64 + (b2 as f64 - b1 as f64) * t).round() as u8;
            return Color::Rgb(r, g, b);
        }
    }

    // fallback（不应到达）
    Color::Rgb(0xBF, 0x00, 0xFF)
}
