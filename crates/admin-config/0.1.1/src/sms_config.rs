//! 短信配置模块
//!
//! 提供短信服务商（腾讯云、阿里云）的配置

use serde::{Deserialize, Serialize};

/// 短信配置
///
/// 支持腾讯云和阿里云短信服务
///
/// # 字段说明
///
/// - `provider`: 短信提供商，可选值：tencent/aliyun
/// - `app_id`: 应用 ID
/// - `app_key`: 应用密钥
/// - `sign_name`: 短信签名
/// - `template_id`: 短信模板 ID
/// - `tencent`: 腾讯云短信配置（可选）
/// - `aliyun`: 阿里云短信配置（可选）
///
/// # 示例
///
/// ```rust
/// use admin_config::SmsConfig;
///
/// let config = SmsConfig {
///     provider: "tencent".to_string(),
///     app_id: "1400000000".to_string(),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SmsConfig {
    /// 短信提供商 (tencent/aliyun)
    pub provider: String,
    /// 应用 ID
    pub app_id: String,
    /// 应用 Key
    pub app_key: String,
    /// 签名名称
    pub sign_name: String,
    /// 模板 ID
    pub template_id: String,
    /// 腾讯云短信配置
    pub tencent: Option<TencentSmsConfig>,
    /// 阿里云短信配置
    pub aliyun: Option<AliyunSmsConfig>,
}

/// 腾讯云短信配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TencentSmsConfig {
    /// SDK App ID
    pub sdk_app_id: String,
    /// Secret ID
    pub secret_id: String,
    /// Secret Key
    pub secret_key: String,
    /// 区域
    pub region: String,
}

/// 阿里云短信配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AliyunSmsConfig {
    /// Access Key ID
    pub access_key_id: String,
    /// Access Key Secret
    pub access_key_secret: String,
    /// 区域
    pub region: String,
}
