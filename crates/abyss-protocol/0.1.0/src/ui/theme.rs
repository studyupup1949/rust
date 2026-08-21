use ratatui::style::Color;

/// 游戏配色方案
#[derive(Debug, Clone)]
pub struct Theme {
    /// 背景色：纯黑 #000000
    pub bg: Color,
    /// 主文字：冷灰白 #C0C0C0
    pub text: Color,
    /// 荧光绿：经典终端强调色 #00FF41
    pub neon_green: Color,
    /// 幽灵蓝：碎语货币色 #00BFFF
    pub ghost_blue: Color,
    /// 深金色：真理货币色 #FFD700
    pub deep_gold: Color,
    /// 血红：危险/警告 #FF0033
    pub blood_red: Color,
    /// 毒紫：突变 #BF00FF
    pub toxic_purple: Color,
    /// 暗灰：边框 #333333
    pub border: Color,
    /// 深灰：锁定/禁用 #555555
    pub locked: Color,
    /// 琥珀黄：成就 #FFAA00
    pub amber: Color,
    /// 青色：协同效应 #00FFCC
    pub cyan: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            bg: Color::Rgb(0x00, 0x00, 0x00),
            text: Color::Rgb(0xC0, 0xC0, 0xC0),
            neon_green: Color::Rgb(0x00, 0xFF, 0x41),
            ghost_blue: Color::Rgb(0x00, 0xBF, 0xFF),
            deep_gold: Color::Rgb(0xFF, 0xD7, 0x00),
            blood_red: Color::Rgb(0xFF, 0x00, 0x33),
            toxic_purple: Color::Rgb(0xBF, 0x00, 0xFF),
            border: Color::Rgb(0x33, 0x33, 0x33),
            locked: Color::Rgb(0x55, 0x55, 0x55),
            amber: Color::Rgb(0xFF, 0xAA, 0x00),
            cyan: Color::Rgb(0x00, 0xFF, 0xCC),
        }
    }
}

impl Theme {
    /// 根据 SAN 值返回对应的颜色
    /// 80-100: 荧光绿, 60-79: 黄绿, 40-59: 橙色, 20-39: 血红, 1-19: 毒紫
    pub fn san_color(&self, san: f64) -> Color {
        if san >= 80.0 {
            self.neon_green
        } else if san >= 60.0 {
            Color::Rgb(0xAA, 0xFF, 0x00) // yellow-green
        } else if san >= 40.0 {
            Color::Rgb(0xFF, 0xAA, 0x00) // orange
        } else if san >= 20.0 {
            self.blood_red
        } else {
            self.toxic_purple
        }
    }
}
