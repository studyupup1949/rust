use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for TencentSmsConfig {
    fn default() -> Self {
        Self {
            sdk_app_id: "".to_string(),
            secret_id: "".to_string(),
            secret_key: "".to_string(),
            region: "ap-guangzhou".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliyunSmsConfig {
    /// Access Key ID
    pub access_key_id: String,
    /// Access Key Secret
    pub access_key_secret: String,
    /// 区域
    pub region: String,
}

impl Default for AliyunSmsConfig {
    fn default() -> Self {
        Self {
            access_key_id: "".to_string(),
            access_key_secret: "".to_string(),
            region: "".to_string(),
        }
    }
}

impl Default for SmsConfig {
    fn default() -> Self {
        Self {
            provider: "tencent".to_string(),
            app_id: "".to_string(),
            app_key: "".to_string(),
            sign_name: "您的应用".to_string(),
            template_id: "123456".to_string(),
            tencent: Some(TencentSmsConfig::default()),
            aliyun: Some(AliyunSmsConfig::default()),
        }
    }
}
