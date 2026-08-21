// 事件触发特效（tachyonfx Effect 工厂函数）

use ratatui::layout::Rect;
use ratatui::style::Color;
use tachyonfx::fx::{self, Direction};
use tachyonfx::{Effect, EffectTimer, Interpolation};

/// 幽灵蓝 #00BFFF
const GHOST_BLUE: Color = Color::Rgb(0x00, 0xBF, 0xFF);
/// 荧光绿 #00FF41
const NEON_GREEN: Color = Color::Rgb(0x00, 0xFF, 0x41);
/// 毒紫 #BF00FF
const TOXIC_PURPLE: Color = Color::Rgb(0xBF, 0x00, 0xFF);
/// 青色 #00FFCC
const CYAN: Color = Color::Rgb(0x00, 0xFF, 0xCC);
/// 纯白
const WHITE: Color = Color::Rgb(0xFF, 0xFF, 0xFF);
/// 纯黑
const BLACK: Color = Color::Rgb(0x00, 0x00, 0x00);

/// Token 闪白效果：白色 → 幽灵蓝 #00BFFF，0.3s
/// 需求 9.1, 9.2
pub fn token_flash_effect(area: Rect) -> Effect {
    fx::fade_from_fg(
        WHITE,
        EffectTimer::from_ms(300, Interpolation::Linear),
    )
    .with_area(area)
}

/// 购买信徒滑入效果：从底部滑入，0.5s
/// 需求 10.1, 10.2
pub fn purchase_slide_effect(area: Rect) -> Effect {
    fx::slide_in(
        Direction::DownToUp,
        3,
        0,
        BLACK,
        EffectTimer::from_ms(500, Interpolation::CubicOut),
    )
    .with_area(area)
}

/// 节点价格闪绿效果：荧光绿 #00FF41 → 正常色，0.3s
/// 需求 11.1
pub fn node_price_flash_effect(area: Rect) -> Effect {
    fx::fade_from_fg(
        NEON_GREEN,
        EffectTimer::from_ms(300, Interpolation::Linear),
    )
    .with_area(area)
}

/// 信徒合成动画：dissolve 0.5s → coalesce 0.5s，总计 1.0s
/// 需求 12.1, 12.2, 12.3
pub fn fusion_effect(area: Rect) -> Effect {
    fx::sequence(&[
        fx::dissolve(EffectTimer::from_ms(500, Interpolation::CubicIn)),
        fx::coalesce(EffectTimer::from_ms(500, Interpolation::CubicOut)),
    ])
    .with_area(area)
}

/// 变异事件全屏脉冲：毒紫 #BF00FF 背景脉冲，0.8s
/// 需求 13.1, 13.2
pub fn mutation_pulse_effect(area: Rect) -> Effect {
    fx::fade_from(
        TOXIC_PURPLE,
        TOXIC_PURPLE,
        EffectTimer::from_ms(800, Interpolation::QuadOut),
    )
    .with_area(area)
}

/// SAN 修复闪绿：荧光绿 → 正常色，0.5s
/// 需求 14.1
pub fn san_repair_flash_effect(area: Rect) -> Effect {
    fx::fade_from_fg(
        NEON_GREEN,
        EffectTimer::from_ms(500, Interpolation::Linear),
    )
    .with_area(area)
}

/// 协同波纹效果：径向扩散 + 青色 #00FFCC，1.5s
/// 使用 sweep_in 模拟径向扩散，配合青色前景渐变
/// 需求 15.1, 15.2
pub fn synergy_ripple_effect(area: Rect) -> Effect {
    fx::parallel(&[
        fx::sweep_in(
            Direction::LeftToRight,
            8,
            3,
            CYAN,
            EffectTimer::from_ms(1500, Interpolation::QuadOut),
        ),
        fx::fade_from_fg(
            CYAN,
            EffectTimer::from_ms(1500, Interpolation::Linear),
        ),
    ])
    .with_area(area)
}

/// 成就横幅效果：底部滑入 + coalesce 文字，3.0s
/// 滑入 0.5s，coalesce 文字显现 2.5s
/// 需求 16.1, 16.2, 16.3
pub fn achievement_banner_effect(area: Rect) -> Effect {
    fx::sequence(&[
        fx::slide_in(
            Direction::DownToUp,
            2,
            0,
            BLACK,
            EffectTimer::from_ms(500, Interpolation::CubicOut),
        ),
        fx::coalesce(EffectTimer::from_ms(2500, Interpolation::Linear)),
    ])
    .with_area(area)
}

/// 维度解锁效果：全屏闪白 0.5s → dissolve 展开 1.5s，总计 2.0s
/// 需求 17.1, 17.2, 17.3
pub fn dimension_unlock_effect(area: Rect) -> Effect {
    fx::sequence(&[
        fx::fade_from(
            WHITE,
            WHITE,
            EffectTimer::from_ms(500, Interpolation::QuadOut),
        ),
        fx::dissolve(EffectTimer::from_ms(1500, Interpolation::CubicInOut)),
    ])
    .with_area(area)
}
