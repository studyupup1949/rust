use thiserror::Error;
use std::time::Duration;

/// ADB 操作相关的错误类型
#[derive(Debug, Error)]
pub enum ADBError {
    /// ADB 命令执行错误
    #[error("ADB 命令错误: {0}")]
    CommandError(String),

    /// 设备通信错误
    #[error("设备通信错误: {0}")]
    DeviceError(String),

    /// 文件操作错误
    #[error("文件操作错误: {0}")]
    FileError(String),

    /// 配置错误
    #[error("配置错误: {0}")]
    ConfigError(String),

    /// 超时错误
    #[error("操作超时 ({duration:?}): {message}")]
    TimeoutError {
        message: String,
        duration: Duration,
    },

    /// 设备不存在
    #[error("设备不存在: {0}")]
    DeviceNotFound(String),

    /// 应用不存在
    #[error("应用不存在: {0}")]
    AppNotFound(String),

    /// 权限不足
    #[error("权限不足: {0}")]
    PermissionDenied(String),

    /// 连接错误
    #[error("连接错误: {0}")]
    ConnectionError(String),

    /// 解析错误
    #[error("解析错误: {0}")]
    ParseError(String),

    /// 未知错误
    #[error("未知错误: {0}")]
    UnknownError(String),
}

// 为标准错误类型实现 From trait，简化错误处理
impl From<std::io::Error> for ADBError {
    fn from(error: std::io::Error) -> Self {
        ADBError::FileError(error.to_string())
    }
}

impl From<std::str::Utf8Error> for ADBError {
    fn from(error: std::str::Utf8Error) -> Self {
        ADBError::ParseError(format!("UTF-8 解码错误: {}", error))
    }
}

impl From<std::num::ParseIntError> for ADBError {
    fn from(error: std::num::ParseIntError) -> Self {
        ADBError::ParseError(format!("数字解析错误: {}", error))
    }
}

impl From<regex::Error> for ADBError {
    fn from(error: regex::Error) -> Self {
        ADBError::ParseError(format!("正则表达式错误: {}", error))
    }
}

// 添加结果类型别名简化使用
pub type ADBResult<T> = Result<T, ADBError>;