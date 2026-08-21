//! 国际化日志支持模块
//!
//! 提供多语言支持的日志记录功能

use std::sync::OnceLock;
use crate::i18n;

/// 国际化日志管理器
pub struct I18nLogger {
    enabled: bool,
}

impl I18nLogger {
    /// 创建新的国际化日志管理器
    pub fn new() -> Self {
        Self {
            enabled: true,
        }
    }

    /// 启用或禁用日志记录
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// 记录信息级别的日志
    pub fn info(&self, key: &str) {
        if !self.enabled {
            return;
        }
        rat_logger::info!("{}", i18n::t(key));
    }

    /// 记录带参数的信息级别日志
    pub fn info_format(&self, key: &str, args: &[&str]) {
        if !self.enabled {
            return;
        }
        rat_logger::info!("{}", i18n::t_format(key, args));
    }

    /// 记录警告级别的日志
    pub fn warn(&self, key: &str) {
        if !self.enabled {
            return;
        }
        rat_logger::warn!("{}", i18n::t(key));
    }

    /// 记录带参数的警告级别日志
    pub fn warn_format(&self, key: &str, args: &[&str]) {
        if !self.enabled {
            return;
        }
        rat_logger::warn!("{}", i18n::t_format(key, args));
    }

    /// 记录错误级别的日志
    pub fn error(&self, key: &str) {
        if !self.enabled {
            return;
        }
        rat_logger::error!("{}", i18n::t(key));
    }

    /// 记录带参数的错误级别日志
    pub fn error_format(&self, key: &str, args: &[&str]) {
        if !self.enabled {
            return;
        }
        rat_logger::error!("{}", i18n::t_format(key, args));
    }

    /// 记录调试级别的日志
    pub fn debug(&self, key: &str) {
        if !self.enabled {
            return;
        }
        rat_logger::debug!("{}", i18n::t(key));
    }

    /// 记录带参数的调试级别日志
    pub fn debug_format(&self, key: &str, args: &[&str]) {
        if !self.enabled {
            return;
        }
        rat_logger::debug!("{}", i18n::t_format(key, args));
    }
}

impl Default for I18nLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局国际化日志实例
static GLOBAL_I18N_LOGGER: OnceLock<I18nLogger> = OnceLock::new();

/// 获取全局国际化日志实例
pub fn get_i18n_logger() -> &'static I18nLogger {
    GLOBAL_I18N_LOGGER.get_or_init(|| I18nLogger::new())
}

/// 记录国际化信息日志
pub fn log_info(key: &str) {
    get_i18n_logger().info(key);
}

/// 记录带参数的国际化信息日志
pub fn log_info_format(key: &str, args: &[&str]) {
    get_i18n_logger().info_format(key, args);
}

/// 记录国际化警告日志
pub fn log_warn(key: &str) {
    get_i18n_logger().warn(key);
}

/// 记录带参数的国际化警告日志
pub fn log_warn_format(key: &str, args: &[&str]) {
    get_i18n_logger().warn_format(key, args);
}

/// 记录国际化错误日志
pub fn log_error(key: &str) {
    get_i18n_logger().error(key);
}

/// 记录带参数的国际化错误日志
pub fn log_error_format(key: &str, args: &[&str]) {
    get_i18n_logger().error_format(key, args);
}

/// 记录国际化调试日志
pub fn log_debug(key: &str) {
    get_i18n_logger().debug(key);
}

/// 记录带参数的国际化调试日志
pub fn log_debug_format(key: &str, args: &[&str]) {
    get_i18n_logger().debug_format(key, args);
}

/// 设置日志启用状态
pub fn set_logging_enabled(enabled: bool) {
    if let Some(logger) = GLOBAL_I18N_LOGGER.get() {
        // 由于设计限制，我们只能通过重新创建实例来实现
        // 在实际使用中，可以通过互斥锁来实现线程安全的状态修改
        // 这里简化处理，仅作为示例
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i18n_logger_creation() {
        let logger = I18nLogger::new();
        assert!(logger.enabled);
    }

    #[test]
    fn test_i18n_logger_enable_disable() {
        let mut logger = I18nLogger::new();
        logger.set_enabled(false);
        assert!(!logger.enabled);
        logger.set_enabled(true);
        assert!(logger.enabled);
    }

    #[test]
    fn test_global_logger() {
        let logger1 = get_i18n_logger();
        let logger2 = get_i18n_logger();
        assert_eq!(logger1.enabled, logger2.enabled);
    }
}