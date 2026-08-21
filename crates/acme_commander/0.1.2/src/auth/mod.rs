//! 认证模块
//! 提供 Cloudflare Token 和 ZeroSSL API Key 验证功能

pub mod cloudflare;
pub mod zerossl;

use crate::error::{AuthError, AuthResult};
use secrecy::{Secret, ExposeSecret};
use governor::{Quota, RateLimiter, DefaultDirectRateLimiter};
use std::num::NonZeroU32;
use std::sync::LazyLock;

/// 支持的认证提供商
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    /// Cloudflare
    Cloudflare,
    /// ZeroSSL
    ZeroSsl,
}

impl Provider {
    /// 获取提供商名称
    pub fn name(&self) -> &'static str {
        match self {
            Provider::Cloudflare => "Cloudflare",
            Provider::ZeroSsl => "ZeroSSL",
        }
    }
    
    /// 从字符串解析提供商
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "cloudflare" | "cf" => Some(Provider::Cloudflare),
            "zerossl" | "zero" => Some(Provider::ZeroSsl),
            _ => None,
        }
    }
}

/// 验证结果
#[derive(Debug, Clone)]
pub enum ValidationResult {
    /// Cloudflare 验证结果
    Cloudflare {
        /// 账户 ID
        account_id: String,
        /// 用户邮箱
        email: Option<String>,
        /// 权限列表
        permissions: Vec<String>,
    },
    /// ZeroSSL 验证结果
    ZeroSsl {
        /// EAB 凭证
        eab_credentials: Option<EabCredentials>,
        /// 账户信息
        account_info: Option<String>,
    },
}

/// EAB (External Account Binding) 凭证
#[derive(Debug, Clone)]
pub struct EabCredentials {
    /// EAB Kid
    pub kid: String,
    /// EAB HMAC Key
    pub hmac_key: String,
}

/// 安全凭证包装器
#[derive(Debug, Clone)]
pub struct SecureCredential {
    /// 加密的凭证
    credential: Secret<String>,
    /// 提供商类型
    provider: Provider,
}

impl SecureCredential {
    /// 创建新的安全凭证
    pub fn new(credential: String, provider: Provider) -> Self {
        Self {
            credential: Secret::new(credential),
            provider,
        }
    }
    
    /// 获取提供商
    pub fn provider(&self) -> Provider {
        self.provider
    }
    
    /// 暴露凭证（仅在需要时使用）
    pub fn expose(&self) -> &str {
        self.credential.expose_secret()
    }
}

/// 请求限流器 - 每分钟最多 10 次请求
static RATE_LIMITER: LazyLock<DefaultDirectRateLimiter> = LazyLock::new(|| {
    RateLimiter::direct(Quota::per_minute(NonZeroU32::new(10).unwrap()))
});

/// 验证凭证
pub async fn validate_credentials(
    provider: Provider,
    credential: &str,
) -> AuthResult<ValidationResult> {
    // 应用速率限制
    RATE_LIMITER.until_ready().await;
    
    let secure_cred = SecureCredential::new(credential.to_string(), provider);
    
    match provider {
        Provider::Cloudflare => {
            let auth = cloudflare::CloudflareAuth::new(secure_cred);
            auth.verify().await
        }
        Provider::ZeroSsl => {
            let auth = zerossl::ZeroSslAuth::new(secure_cred);
            auth.verify().await
        }
    }
}

/// 验证凭证（带重试机制）
pub async fn validate_credentials_with_retry(
    provider: Provider,
    credential: &str,
    max_retries: u32,
) -> AuthResult<ValidationResult> {
    let mut last_error = None;
    
    for attempt in 0..=max_retries {
        match validate_credentials(provider, credential).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);
                
                // 如果不是最后一次尝试，等待一段时间后重试
                if attempt < max_retries {
                    let delay = std::time::Duration::from_secs(2_u64.pow(attempt));
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
    
    Err(last_error.unwrap_or_else(|| {
        AuthError::ServiceError("验证过程中发生未知错误".to_string())
    }))
}

/// 批量验证多个凭证
pub async fn validate_multiple_credentials(
    credentials: Vec<(Provider, String)>,
) -> Vec<(Provider, AuthResult<ValidationResult>)> {
    let mut results = Vec::new();
    
    for (provider, credential) in credentials {
        let result = validate_credentials(provider, &credential).await;
        results.push((provider, result));
    }
    
    results
}

/// 检查凭证格式是否有效
pub fn validate_credential_format(provider: Provider, credential: &str) -> AuthResult<()> {
    if credential.trim().is_empty() {
        return Err(AuthError::InvalidToken("凭证不能为空".to_string()));
    }
    
    match provider {
        Provider::Cloudflare => {
            // Cloudflare Token 格式验证
            if credential.len() < 40 {
                return Err(AuthError::InvalidToken(
                    "Cloudflare 令牌似乎太短".to_string()
                ));
            }
            
            // 检查是否包含无效字符
            if !credential.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
                return Err(AuthError::InvalidToken(
                    "Cloudflare 令牌包含无效字符".to_string()
                ));
            }
        }
        Provider::ZeroSsl => {
            // ZeroSSL API Key 格式验证
            if credential.len() < 32 {
                return Err(AuthError::InvalidToken(
                    "ZeroSSL API 密钥似乎太短".to_string()
                ));
            }
            
            // 检查是否为有效的十六进制字符串
            if !credential.chars().all(|c| c.is_ascii_hexdigit()) {                return Err(AuthError::InvalidToken(
                    "ZeroSSL API 密钥应仅包含十六进制字符".to_string()
                ));
            }
        }
    }
    
    Ok(())
}
