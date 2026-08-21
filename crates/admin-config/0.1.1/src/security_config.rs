//! 安全配置模块
//!
//! 提供加密密钥、CORS、CSRF 等安全相关配置

use serde::{Deserialize, Serialize};

/// 安全配置
///
/// 包含加密密钥、跨域配置、CSRF 防护等安全相关设置
///
/// # 字段说明
///
/// - `aes_key`: AES-256 加密密钥（64位十六进制，32字节）
/// - `aes_iv`: AES 初始化向量（32位十六进制，16字节）
/// - `api_key_encrypt_key`: API 密钥加密密钥（64位十六进制，32字节）
/// - `password_salt`: 密码加密盐值
/// - `enable_cors`: 是否启用跨域资源共享
/// - `allowed_origins`: 允许的来源域名（逗号分隔）
/// - `enable_csrf`: 是否启用 CSRF 防护
///
/// # 示例
///
/// ```rust
/// use admin_config::SecurityConfig;
///
/// let config = SecurityConfig::default();
/// assert_eq!(config.aes_key.len(), 64);
/// assert_eq!(config.aes_iv.len(), 32);
/// assert!(config.enable_cors);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// AES 密钥（64位十六进制，32字节）
    pub aes_key: String,
    /// AES 向量（32位十六进制，16字节）
    pub aes_iv: String,
    /// API 密钥加密密钥（64位十六进制，32字节，用于 AES-256-GCM）
    pub api_key_encrypt_key: String,
    /// 密码加密盐值
    pub password_salt: String,
    /// 是否启用 CORS
    pub enable_cors: bool,
    /// 允许的来源（逗号分隔）
    pub allowed_origins: String,
    /// 是否启用 CSRF 防护
    pub enable_csrf: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            aes_key: Self::generate_hex(32),
            aes_iv: Self::generate_hex(16),
            api_key_encrypt_key: Self::generate_hex(32),
            password_salt: Self::generate_hex(16),
            enable_cors: true,
            allowed_origins: String::new(),
            enable_csrf: false,
        }
    }
}

impl SecurityConfig {
    /// 获取允许的来源列表
    ///
    /// 将逗号分隔的来源字符串解析为 Vec
    pub fn allowed_origins_list(&self) -> Vec<String> {
        self.allowed_origins
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// 生成指定字节长度的随机十六进制字符串
    pub fn generate_hex(byte_len: usize) -> String {
        (0..byte_len).map(|_| format!("{:02x}", rand::random::<u8>())).collect()
    }

    /// 验证安全配置的有效性
    ///
    /// 检查密钥长度和格式是否正确
    pub fn validate(&self) -> Result<(), String> {
        if self.aes_key.len() != 64 {
            return Err(format!(
                "aes_key 必须是 64 位十六进制字符串（32 字节），当前长度: {}",
                self.aes_key.len()
            ));
        }

        if self.aes_iv.len() != 32 {
            return Err(format!(
                "aes_iv 必须是 32 位十六进制字符串（16 字节），当前长度: {}",
                self.aes_iv.len()
            ));
        }

        if self.api_key_encrypt_key.len() != 64 {
            return Err(format!(
                "api_key_encrypt_key 必须是 64 位十六进制字符串（32 字节），当前长度: {}",
                self.api_key_encrypt_key.len()
            ));
        }

        if !self.aes_key.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("aes_key 必须只包含十六进制字符 (0-9, a-f, A-F)".to_string());
        }

        if !self.aes_iv.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("aes_iv 必须只包含十六进制字符 (0-9, a-f, A-F)".to_string());
        }

        if !self.api_key_encrypt_key.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("api_key_encrypt_key 必须只包含十六进制字符 (0-9, a-f, A-F)".to_string());
        }

        Ok(())
    }
}
