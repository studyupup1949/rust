use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// 主布局区域
pub struct AppLayout {
    /// 顶部状态栏 (3 行)
    pub status_bar: Rect,
    /// Tab 导航栏 (1 行)
    pub tab_bar: Rect,
    /// 主内容区 (70%)
    pub content: Rect,
    /// 侧边栏 (30%)
    pub sidebar: Rect,
    /// 底部资源栏 (1 行)
    pub resource_bar: Rect,
    /// 快捷键提示栏 (1 行)
    pub keybind_bar: Rect,
}

impl AppLayout {
    /// 根据终端尺寸计算所有区域
    pub fn new(area: Rect) -> Self {
        // 垂直分割：状态栏3行、Tab栏1行、中间区域(弹性)、资源栏1行、快捷键栏1行
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // 顶部状态栏
                Constraint::Length(1),  // Tab 导航
                Constraint::Min(5),    // 主内容区 + 侧边栏
                Constraint::Length(1),  // 底部资源栏
                Constraint::Length(1),  // 快捷键提示
            ])
            .split(area);

        // 中间区域水平分割：主内容 70% / 侧边栏 30%
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(70),
                Constraint::Percentage(30),
            ])
            .split(vertical[2]);

        Self {
            status_bar: vertical[0],
            tab_bar: vertical[1],
            content: horizontal[0],
            sidebar: horizontal[1],
            resource_bar: vertical[3],
            keybind_bar: vertical[4],
        }
    }
}
