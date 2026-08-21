use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// ADB 配置结构体
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ADBConfig {
    /// ADB 可执行文件路径
    pub path: PathBuf,
    /// 重试最大次数
    pub max_retries: u32,
    /// 重试延迟（毫秒）
    pub retry_delay: u64,
    /// 操作超时（毫秒）
    pub timeout: u64,
    /// 日志级别
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_level: Option<String>,
    /// 额外的命令行参数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_args: Option<Vec<String>>,
}

impl Default for ADBConfig {
    fn default() -> Self {
        ADBConfig {
            path: PathBuf::from("adb"),
            max_retries: 3,
            retry_delay: 1000,
            timeout: 30000, // 30秒超时
            log_level: None,
            additional_args: None,
        }
    }
}

/// ADB 配置构建器
#[derive(Default)]
pub struct ADBConfigBuilder {
    path: Option<PathBuf>,
    max_retries: Option<u32>,
    retry_delay: Option<u64>,
    timeout: Option<u64>,
    log_level: Option<String>,
    additional_args: Option<Vec<String>>,
}

impl ADBConfigBuilder {
    /// 设置 ADB 可执行文件路径
    pub fn path(mut self, path: impl Into<PathBuf>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// 设置最大重试次数
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = Some(retries);
        self
    }

    /// 设置重试延迟
    pub fn retry_delay(mut self, delay: u64) -> Self {
        self.retry_delay = Some(delay);
        self
    }

    /// 设置操作超时
    pub fn timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// 设置日志级别
    pub fn log_level(mut self, level: &str) -> Self {
        self.log_level = Some(level.to_string());
        self
    }

    /// 添加额外命令行参数
    pub fn add_arg(mut self, arg: &str) -> Self {
        if self.additional_args.is_none() {
            self.additional_args = Some(Vec::new());
        }

        if let Some(args) = &mut self.additional_args {
            args.push(arg.to_string());
        }

        self
    }

    /// 构建 ADB 配置
    pub fn build(self) -> ADBConfig {
        let default = ADBConfig::default();

        ADBConfig {
            path: self.path.unwrap_or(default.path),
            max_retries: self.max_retries.unwrap_or(default.max_retries),
            retry_delay: self.retry_delay.unwrap_or(default.retry_delay),
            timeout: self.timeout.unwrap_or(default.timeout),
            log_level: self.log_level,
            additional_args: self.additional_args,
        }
    }
}