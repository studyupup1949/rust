//! ZeroSSL API Key 验证模块
//! 验证 ZeroSSL API Key 并检查 ACME 访问权限

use crate::auth::{SecureCredential, ValidationResult, EabCredentials};
use crate::error::{AuthError, AuthResult};
use rat_logger::{info, error};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// ZeroSSL API 基础 URL
const ZEROSSL_API_BASE: &str = "https://api.zerossl.com";

/// ZeroSSL 认证器
#[derive(Debug)]
pub struct ZeroSslAuth {
    /// 安全凭证
    credential: SecureCredential,
    /// HTTP 客户端
    client: Client,
}

/// ZeroSSL API 响应结构
#[derive(Debug, Deserialize)]
struct ZeroSslResponse<T> {
    success: bool,
    error: Option<ZeroSslError>,
    #[serde(flatten)]
    result: Option<T>,
}

/// ZeroSSL 错误信息
#[derive(Debug, Deserialize)]
struct ZeroSslError {
    code: u32,
    #[serde(rename = "type")]
    error_type: String,
    message: String,
}

/// EAB 凭证响应
#[derive(Debug, Deserialize)]
struct EabCredentialsResponse {
    success: bool,
    eab_kid: Option<String>,
    eab_hmac_key: Option<String>,
}

/// 账户信息响应
#[derive(Debug, Deserialize)]
struct AccountInfo {
    id: String,
    email: String,
    #[serde(rename = "type")]
    account_type: String,
    status: String,
    created: String,
    updated: String,
}

/// 证书配额信息
#[derive(Debug, Deserialize)]
struct CertificateQuota {
    limit: u32,
    used: u32,
    remaining: u32,
}

/// 账户详细信息
#[derive(Debug, Deserialize)]
struct AccountDetails {
    #[serde(flatten)]
    info: AccountInfo,
    certificate_quota: Option<CertificateQuota>,
    features: Option<Vec<String>>,
}

impl ZeroSslAuth {
    /// 创建新的 ZeroSSL 认证器
    pub fn new(credential: SecureCredential) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("acme-commander/0.1.0")
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            credential,
            client,
        }
    }
    
    /// 验证 API Key 有效性并返回验证结果
    pub async fn verify(&self) -> AuthResult<ValidationResult> {
        // 首先验证 API Key 基本有效性
        let account_info = self.get_account_info().await?;
        
        // 获取 EAB 凭证
        let eab_credentials = self.get_eab_credentials().await.ok();
        
        // 验证 ACME 功能是否启用
        self.validate_acme_access(&eab_credentials).await?;
        
        // 构建验证结果
        Ok(ValidationResult::ZeroSsl {
            eab_credentials,
            account_info: Some(account_info.info.email),
        })
    }
    
    /// 获取账户信息
    async fn get_account_info(&self) -> AuthResult<AccountDetails> {
        let url = format!("{}/account", ZEROSSL_API_BASE);
        let api_key = self.credential.expose();
        let masked_key = if api_key.len() > 8 {
            format!("{}**", &api_key[..8])
        } else {
            "****".to_string()
        };
        
        info!("🔗 请求 ZeroSSL API: {}", url);
        info!("🔑 使用 API Key: {}", masked_key);
        
        let response = self.client
            .get(&url)
            .query(&[("access_key", api_key)])
            .send()
            .await
            .map_err(|e| {
                error!("❌ HTTP 请求失败: {}", e);
                AuthError::ServiceError(format!("请求失败: {}", e))
            })?;
        
        let status = response.status();
        info!("📡 HTTP 响应状态: {}", status);
        
        if status == 401 {
            return Err(AuthError::InvalidToken("Invalid API key".to_string()));
        }
        
        if status == 429 {
            return Err(AuthError::RateLimitExceeded);
        }
        
        let account_info: AccountDetails = response
            .json()
            .await
            .map_err(|e| {
                error!("❌ 解析响应失败: {}", e);
                AuthError::ServiceError(format!("解析响应失败: {}", e))
            })?;
        
        // 检查账户状态
        if account_info.info.status != "active" {
            return Err(AuthError::ServiceError(
                format!("Account status is not active: {}", account_info.info.status)
            ));
        }
        
        Ok(account_info)
    }
    
    /// 获取 EAB 凭证
    async fn get_eab_credentials(&self) -> AuthResult<EabCredentials> {
        let url = format!("{}/acme/eab-credentials", ZEROSSL_API_BASE);
        let api_key = self.credential.expose();
        let masked_key = if api_key.len() > 8 {
            format!("{}**", &api_key[..8])
        } else {
            "****".to_string()
        };
        
        info!("🔗 请求 ZeroSSL EAB API: {}", url);
        info!("🔑 使用 API Key: {}", masked_key);
        
        let response = self.client
            .get(&url)
            .query(&[("access_key", api_key)])
            .send()
            .await
            .map_err(|e| {
                error!("❌ HTTP 请求失败: {}", e);
                AuthError::ServiceError(format!("Request failed: {}", e))
            })?;
        
        let status = response.status();
        info!("📡 HTTP 响应状态: {}", status);
        
        if status == 401 {
            return Err(AuthError::InvalidToken("Invalid API key".to_string()));
        }
        
        if status == 403 {
            return Err(AuthError::AcmeDisabled);
        }
        
        let eab_response: EabCredentialsResponse = response
            .json()
            .await
            .map_err(|e| {
                error!("❌ 解析响应失败: {}", e);
                AuthError::ServiceError(format!("Failed to parse response: {}", e))
            })?;
        
        if !eab_response.success {
            return Err(AuthError::ServiceError("Failed to get EAB credentials".to_string()));
        }
        
        let kid = eab_response.eab_kid
            .ok_or_else(|| AuthError::ServiceError("Missing EAB kid".to_string()))?;
        
        let hmac_key = eab_response.eab_hmac_key
            .ok_or_else(|| AuthError::ServiceError("Missing EAB HMAC key".to_string()))?;
        
        Ok(EabCredentials {
            kid,
            hmac_key,
        })
    }
    
    /// 验证 ACME 访问权限
    async fn validate_acme_access(&self, eab_credentials: &Option<EabCredentials>) -> AuthResult<()> {
        if eab_credentials.is_none() {
            return Err(AuthError::AcmeDisabled);
        }
        
        // 可以添加更多的 ACME 功能检查
        // 例如检查证书配额等
        
        Ok(())
    }
    
    /// 检查证书配额
    pub async fn check_certificate_quota(&self) -> AuthResult<CertificateQuota> {
        let account_info = self.get_account_info().await?;
        
        account_info.certificate_quota
            .ok_or_else(|| AuthError::ServiceError("证书配额信息不可用".to_string()))
    }
    
    /// 获取账户功能列表
    pub async fn get_account_features(&self) -> AuthResult<Vec<String>> {
        let account_info = self.get_account_info().await?;
        
        Ok(account_info.features.unwrap_or_default())
    }
    
    /// 验证域名是否可以申请证书
    pub async fn validate_domain(&self, domain: &str) -> AuthResult<bool> {
        let url = format!("{}/validation/domain", ZEROSSL_API_BASE);
        
        #[derive(Serialize)]
        struct DomainValidationRequest {
            access_key: String,
            domain: String,
        }
        
        let request = DomainValidationRequest {
            access_key: self.credential.expose().to_string(),
            domain: domain.to_string(),
        };
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AuthError::ServiceError(format!("Request failed: {}", e)))?;
        
        #[derive(Deserialize)]
        struct DomainValidationResponse {
            success: bool,
            valid: Option<bool>,
            error: Option<ZeroSslError>,
        }
        
        let validation_response: DomainValidationResponse = response
            .json()
            .await
            .map_err(|e| AuthError::ServiceError(format!("Failed to parse response: {}", e)))?;
        
        if !validation_response.success {
            if let Some(error) = validation_response.error {
                return Err(AuthError::ServiceError(error.message));
            }
            return Err(AuthError::ServiceError("域名验证失败".to_string()));
        }
        
        Ok(validation_response.valid.unwrap_or(false))
    }
    
    /// 获取 ACME 目录 URL
    pub fn get_acme_directory_url(&self) -> &'static str {
        "https://acme.zerossl.com/v2/DV90"
    }
    
    /// 检查 API Key 是否即将过期
    pub async fn check_api_key_expiry(&self) -> AuthResult<Option<chrono::DateTime<chrono::Utc>>> {
        // ZeroSSL API Key 通常不会过期，但可以检查账户状态
        let account_info = self.get_account_info().await?;
        
        // 如果账户类型是试用版，可能有时间限制
        if account_info.info.account_type == "trial" {
            // 试用账户通常有 90 天限制
            if let Ok(created) = chrono::DateTime::parse_from_rfc3339(&account_info.info.created) {
                let expiry = created + chrono::Duration::days(90);
                return Ok(Some(expiry.with_timezone(&chrono::Utc)));
            }
        }
        
        Ok(None) // 正式账户通常不会过期
    }
}

/// 便捷函数：快速验证 ZeroSSL API Key
pub async fn verify_zerossl_api_key(api_key: &str) -> AuthResult<ValidationResult> {
    use crate::auth::Provider;
    let credential = SecureCredential::new(api_key.to_string(), Provider::ZeroSsl);
    let auth = ZeroSslAuth::new(credential);
    auth.verify().await
}

/// 便捷函数：获取 EAB 凭证
pub async fn get_zerossl_eab_credentials(api_key: &str) -> AuthResult<EabCredentials> {
    use crate::auth::Provider;
    let credential = SecureCredential::new(api_key.to_string(), Provider::ZeroSsl);
    let auth = ZeroSslAuth::new(credential);
    auth.get_eab_credentials().await
}

/// 便捷函数：检查证书配额
pub async fn check_zerossl_quota(api_key: &str) -> AuthResult<CertificateQuota> {
    use crate::auth::Provider;
    let credential = SecureCredential::new(api_key.to_string(), Provider::ZeroSsl);
    let auth = ZeroSslAuth::new(credential);
    auth.check_certificate_quota().await
}