use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosConfig {
    /// Secret ID
    pub secret_id: String,
    /// Secret Key
    pub secret_key: String,
    /// 存储桶名称
    pub bucket: String,
    /// 区域
    pub region: String,
    /// CDN 域名
    pub cdn_domain: Option<String>,
}

impl Default for CosConfig {
    fn default() -> Self {
        Self {
            secret_id: "".to_string(),
            secret_key: "".to_string(),
            bucket: "".to_string(),
            region: "ap-guangzhou".to_string(),
            cdn_domain: Some("".to_string()),
        }
    }
}
