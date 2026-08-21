use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentConfig {
    /// 是否启用开发模式
    pub enabled: bool,
    /// 是否启用调试模式
    pub debug: bool,
    /// 是否启用热重载
    pub hot_reload: bool,
    /// 是否显示错误详情
    pub show_error_details: bool,
}

impl Default for DevelopmentConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            debug: false,
            hot_reload: false,
            show_error_details: true,
        }
    }
}
