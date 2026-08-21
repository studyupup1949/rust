//! ACME Commander 日志系统
//! 基于 rat_logger 的日志系统，提供统一的日志接口和配置
//!
//! ## 设计原则
//!
//! 作为库使用时：
//! - 不主动初始化日志系统，由调用者负责
//! - 提供安全的日志宏，未初始化时静默失败
//! - 不同级别的日志有不同处理策略
//!
//! 作为独立应用使用时：
//! - 在main函数中负责初始化日志系统
//! - 支持丰富的配置选项

use rat_logger::{LoggerBuilder, Level, LevelFilter};
use rat_logger::config::{Record, Metadata};
use rat_logger::handler::term::TermConfig;
use rat_logger::{FormatConfig, LevelStyle, ColorConfig};
use std::sync::Arc;
use std::path::PathBuf;
use chrono::Local;

/// 日志级别映射
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => Level::Error,
            LogLevel::Warn => Level::Warn,
            LogLevel::Info => Level::Info,
            LogLevel::Debug => Level::Debug,
            LogLevel::Trace => Level::Trace,
        }
    }
}

impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        }
    }
}

/// 日志输出类型
#[derive(Debug, Clone)]
pub enum LogOutput {
    /// 终端输出
    Terminal,
    /// 文件输出
    File {
        log_dir: PathBuf,
        max_file_size: u64,
        max_compressed_files: u32,
    },
    /// UDP网络输出
    Udp {
        server_addr: String,
        server_port: u16,
        auth_token: String,
        app_id: String,
    },
}

/// 日志配置
#[derive(Debug, Clone)]
pub struct LogConfig {
    pub enabled: bool,
    pub level: LogLevel,
    pub output: LogOutput,
    pub use_colors: bool,
    pub use_emoji: bool,
    pub show_timestamp: bool,
    pub show_module: bool,
    pub enable_async: bool,
    pub batch_size: usize,
    pub batch_interval_ms: u64,
    pub buffer_size: usize,
}

impl Default for LogConfig {
    fn default() -> Self {
        LogConfig {
            enabled: true,
            level: LogLevel::Info,
            output: LogOutput::Terminal,
            use_colors: true,
            use_emoji: true,
            show_timestamp: true,
            show_module: true,
            enable_async: false,
            batch_size: 2048,
            batch_interval_ms: 25,
            buffer_size: 16 * 1024,
        }
    }
}

impl LogConfig {
    /// 创建禁用日志的配置
    pub fn disabled() -> Self {
        LogConfig {
            enabled: false,
            ..Default::default()
        }
    }

    /// 创建文件日志配置
    pub fn file<P: Into<PathBuf>>(log_dir: P) -> Self {
        LogConfig {
            enabled: true,
            level: LogLevel::Info,
            output: LogOutput::File {
                log_dir: log_dir.into(),
                max_file_size: 10 * 1024 * 1024, // 10MB
                max_compressed_files: 5,
            },
            use_colors: false, // 文件日志不使用颜色
            use_emoji: false,  // 文件日志不使用emoji
            show_timestamp: true,
            show_module: true,
            enable_async: false,
            batch_size: 2048,
            batch_interval_ms: 25,
            buffer_size: 16 * 1024,
        }
    }

    /// 创建UDP日志配置
    pub fn udp(server_addr: String, server_port: u16, auth_token: String, app_id: String) -> Self {
        LogConfig {
            enabled: true,
            level: LogLevel::Info,
            output: LogOutput::Udp {
                server_addr,
                server_port,
                auth_token,
                app_id,
            },
            use_colors: false, // UDP日志不使用颜色
            use_emoji: false,  // UDP日志不使用emoji
            show_timestamp: true,
            show_module: true,
            enable_async: true,
            batch_size: 2048,
            batch_interval_ms: 25,
            buffer_size: 16 * 1024,
        }
    }
}

/// 日志管理器
pub struct LogManager {
    config: LogConfig,
}

impl LogManager {
    /// 创建新的日志管理器
    pub fn new(config: LogConfig) -> Self {
        Self { config }
    }

    /// 初始化日志系统
    pub fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 如果完全禁用日志，直接返回
        if !self.config.enabled {
            return Ok(());
        }

        // 创建ACME Commander主题格式配置
        let format_config = FormatConfig {
            timestamp_format: "%H:%M:%S%.3f".to_string(),
            level_style: LevelStyle {
                error: "ERROR".to_string(),
                warn: "WARN ".to_string(),
                info: "INFO ".to_string(),
                debug: "DEBUG".to_string(),
                trace: "TRACE".to_string(),
            },
            format_template: "{timestamp} [{level}] [ACME-CMD] {message}".to_string(),
        };

        // 创建颜色配置（如果启用颜色）
        let color_config = if self.config.use_colors {
            Some(ColorConfig {
                error: "\x1b[91m".to_string(),      // 亮红色
                warn: "\x1b[93m".to_string(),       // 亮黄色
                info: "\x1b[92m".to_string(),       // 亮绿色
                debug: "\x1b[96m".to_string(),      // 亮青色
                trace: "\x1b[95m".to_string(),      // 亮紫色
                timestamp: "\x1b[90m".to_string(),   // 深灰色
                target: "\x1b[94m".to_string(),      // 亮蓝色
                file: "\x1b[95m".to_string(),       // 亮紫色
                message: "\x1b[97m".to_string(),      // 亮白色
            })
        } else {
            None
        };

        let term_config = TermConfig {
            enable_color: self.config.use_colors,
            format: Some(format_config),
            color: color_config,
        };

        let level_filter = LevelFilter::from(self.config.level);

        let mut builder = LoggerBuilder::new().with_level(level_filter);

        // 配置异步模式和批量处理
        if self.config.enable_async {
            builder = builder.with_async_mode(true);

            // 设置批量配置
            let batch_config = rat_logger::producer_consumer::BatchConfig {
                batch_size: self.config.batch_size,
                batch_interval_ms: self.config.batch_interval_ms,
                buffer_size: self.config.buffer_size,
            };
            builder = builder.with_batch_config(batch_config);
        }

        // 添加终端处理器
        builder = builder.add_terminal_with_config(term_config);

        // 根据输出类型添加其他处理器
        match &self.config.output {
            LogOutput::File { log_dir, max_file_size, max_compressed_files } => {
                use rat_logger::config::FileConfig;
                let file_config = FileConfig {
                    log_dir: log_dir.clone(),
                    max_file_size: *max_file_size,
                    max_compressed_files: *max_compressed_files as usize,
                    compression_level: 4,
                    min_compress_threads: 2,
                    skip_server_logs: false,
                    is_raw: false,
                    compress_on_drop: false,
                    force_sync: true,
                    format: Some(FormatConfig {
                        timestamp_format: "%Y-%m-%d %H:%M:%S%.3f".to_string(),
                        level_style: LevelStyle {
                            error: "ERROR".to_string(),
                            warn: "WARN ".to_string(),
                            info: "INFO ".to_string(),
                            debug: "DEBUG".to_string(),
                            trace: "TRACE".to_string(),
                        },
                        format_template: "[{timestamp}] [{level}] [ACME-CMD] {message}".to_string(),
                    }),
                };
                builder = builder.add_file(file_config);
            }
            LogOutput::Udp { server_addr, server_port, auth_token, app_id } => {
                use rat_logger::config::NetworkConfig;
                let network_config = NetworkConfig {
                    server_addr: server_addr.clone(),
                    server_port: *server_port,
                    auth_token: auth_token.clone(),
                    app_id: app_id.clone(),
                };
                builder = builder.add_udp(network_config);
            }
            LogOutput::Terminal => {
                // 已经添加了终端处理器
            }
        }

        // 初始化日志器
        builder.init().map_err(|e| format!("日志初始化失败: {}", e))?;

        Ok(())
    }

    /// 获取配置
    pub fn config(&self) -> &LogConfig {
        &self.config
    }
}

/// 便捷的初始化函数
pub fn init_logger(config: LogConfig) -> Result<(), Box<dyn std::error::Error>> {
    let manager = LogManager::new(config);
    manager.initialize()
}

/// 使用默认配置初始化
pub fn init_default_logger() -> Result<(), Box<dyn std::error::Error>> {
    init_logger(LogConfig::default())
}

/// 强制刷新日志（仅在异步模式下有效）
pub fn flush_logs() {
    rat_logger::flush_logs!();
}

/// 条件性强制刷新日志
pub fn flush_logs_if_async(config: &LogConfig) {
    if config.enabled && config.enable_async {
        rat_logger::flush_logs!();
    }
}

// ============================================================================
// 简单日志宏 - 不需要配置参数
// ============================================================================

/// 简单的ACME信息日志
#[macro_export]
macro_rules! acme_info {
    ($($arg:tt)*) => {
        rat_logger::info!($($arg)*);
    };
}

/// 简单的ACME警告日志
#[macro_export]
macro_rules! acme_warn {
    ($($arg:tt)*) => {
        rat_logger::warn!($($arg)*);
    };
}

/// 简单的ACME错误日志
#[macro_export]
macro_rules! acme_log_error {
    ($($arg:tt)*) => {
        rat_logger::error!($($arg)*);
    };
}

/// 简单的ACME调试日志
#[macro_export]
macro_rules! acme_debug {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        rat_logger::debug!($($arg)*);
    };
}

/// 证书信息日志
#[macro_export]
macro_rules! cert_info {
    ($($arg:tt)*) => {
        rat_logger::info!($($arg)*);
    };
}

/// DNS信息日志
#[macro_export]
macro_rules! dns_info {
    ($($arg:tt)*) => {
        rat_logger::info!($($arg)*);
    };
}

/// 性能日志宏
#[macro_export]
macro_rules! perf_log {
    ($level:ident, $($arg:tt)*) => {
        rat_logger::$level!("[PERF] {}", format!($($arg)*));
    };
}

/// 审计日志宏
#[macro_export]
macro_rules! audit_log {
    ($level:ident, $($arg:tt)*) => {
        rat_logger::$level!("[AUDIT] {}", format!($($arg)*));
    };
}

/// 性能监控日志结构
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub operation: String,
    pub duration_ms: f64,
    pub success: bool,
    pub details: Option<String>,
}

impl PerformanceMetrics {
    /// 创建新的性能指标
    pub fn new(operation: String, duration_ms: f64, success: bool) -> Self {
        Self {
            operation,
            duration_ms,
            success,
            details: None,
        }
    }

    /// 添加详细信息
    pub fn with_details(mut self, details: String) -> Self {
        self.details = Some(details);
        self
    }

    /// 记录性能日志
    pub fn log(&self) {
        let status = if self.success { "SUCCESS" } else { "FAILED" };
        let details = self.details.as_deref().unwrap_or("");

        if self.success {
            perf_log!(info,
                "Operation: {} | Duration: {:.2}ms | Status: {} | Details: {}",
                self.operation, self.duration_ms, status, details
            );
        } else {
            perf_log!(warn,
                "Operation: {} | Duration: {:.2}ms | Status: {} | Details: {}",
                self.operation, self.duration_ms, status, details
            );
        }
    }
}

/// 审计日志结构
#[derive(Debug, Clone)]
pub struct AuditEvent {
    pub event_type: String,
    pub user_id: Option<String>,
    pub resource: String,
    pub action: String,
    pub result: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl AuditEvent {
    /// 创建新的审计事件
    pub fn new(event_type: String, resource: String, action: String, result: String) -> Self {
        Self {
            event_type,
            user_id: None,
            resource,
            action,
            result,
            timestamp: chrono::Utc::now(),
        }
    }

    /// 设置用户 ID
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// 记录审计日志
    pub fn log(&self) {
        let user_info = self.user_id.as_deref().unwrap_or("anonymous");

        audit_log!(info,
            "Type: {} | User: {} | Resource: {} | Action: {} | Result: {} | Time: {}",
            self.event_type, user_info, self.resource, self.action, self.result,
            self.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
        );
    }
}

/// 日志工具函数
pub mod utils {
    use super::*;
    use std::time::Instant;

    /// 性能计时器
    pub struct Timer {
        start: Instant,
        operation: String,
    }

    impl Timer {
        /// 开始计时
        pub fn start(operation: String) -> Self {
            Self {
                start: Instant::now(),
                operation,
            }
        }

        /// 结束计时并记录日志
        pub fn finish(self, success: bool) -> PerformanceMetrics {
            let duration = self.start.elapsed();
            let duration_ms = duration.as_secs_f64() * 1000.0;

            let metrics = PerformanceMetrics::new(self.operation, duration_ms, success);
            metrics.log();
            metrics
        }

        /// 结束计时并记录带详细信息的日志
        pub fn finish_with_details(
            self,
            success: bool,
            details: String
        ) -> PerformanceMetrics {
            let duration = self.start.elapsed();
            let duration_ms = duration.as_secs_f64() * 1000.0;

            let metrics = PerformanceMetrics::new(self.operation, duration_ms, success)
                .with_details(details);
            metrics.log();
            metrics
        }
    }

    /// 格式化字节大小
    pub fn format_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", bytes, UNITS[unit_index])
        } else {
            format!("{:.2} {}", size, UNITS[unit_index])
        }
    }

    /// 格式化持续时间
    pub fn format_duration(duration_ms: f64) -> String {
        if duration_ms < 1.0 {
            format!("{:.3}ms", duration_ms)
        } else if duration_ms < 1000.0 {
            format!("{:.2}ms", duration_ms)
        } else {
            format!("{:.2}s", duration_ms / 1000.0)
        }
    }
}