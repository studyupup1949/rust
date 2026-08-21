//! Cloudflare Token 验证模块
//! 通过 Cloudflare API 验证 Token 权限并获取账户信息

use crate::auth::{SecureCredential, ValidationResult};
use crate::error::{AuthError, AuthResult};
use rat_logger::{info, error};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Cloudflare API 基础 URL
const CLOUDFLARE_API_BASE: &str = "https://api.cloudflare.com/client/v4";

/// Cloudflare 认证器
#[derive(Debug)]
pub struct CloudflareAuth {
    /// 安全凭证
    credential: SecureCredential,
    /// HTTP 客户端
    client: Client,
}

/// Cloudflare API 响应结构
#[derive(Debug, Deserialize)]
struct CloudflareResponse<T> {
    success: bool,
    errors: Vec<CloudflareError>,
    messages: Vec<CloudflareMessage>,
    result: Option<T>,
}

/// Cloudflare 错误信息
#[derive(Debug, Deserialize)]
struct CloudflareError {
    code: u32,
    message: String,
}

/// Cloudflare 消息
#[derive(Debug, Deserialize)]
struct CloudflareMessage {
    code: u32,
    message: String,
}

/// Token 验证响应
#[derive(Debug, Deserialize)]
struct TokenVerifyResult {
    id: String,
    status: String,
}

/// 用户信息响应
#[derive(Debug, Deserialize)]
struct UserResult {
    id: String,
    email: String,
    first_name: Option<String>,
    last_name: Option<String>,
    username: String,
    telephone: Option<String>,
    country: Option<String>,
    zipcode: Option<String>,
    created_on: String,
    modified_on: String,
    two_factor_authentication_enabled: bool,
}

/// Token 权限信息
#[derive(Debug, Deserialize)]
struct TokenPermission {
    id: String,
    name: String,
}

/// Token 详细信息
#[derive(Debug, Deserialize)]
struct TokenDetails {
    id: String,
    name: Option<String>,
    status: String,
    issued_on: Option<String>,
    modified_on: Option<String>,
    not_before: Option<String>,
    expires_on: Option<String>,
    policies: Option<Vec<TokenPolicy>>,
    condition: Option<TokenCondition>,
}

/// Token 策略
#[derive(Debug, Deserialize)]
struct TokenPolicy {
    id: String,
    effect: String,
    resources: serde_json::Value,
    permission_groups: Vec<TokenPermission>,
}

/// Token 条件
#[derive(Debug, Deserialize)]
struct TokenCondition {
    request_ip: Option<TokenIpCondition>,
}

/// IP 条件
#[derive(Debug, Deserialize)]
struct TokenIpCondition {
    #[serde(rename = "in")]
    in_list: Option<Vec<String>>,
    #[serde(rename = "not_in")]
    not_in_list: Option<Vec<String>>,
}

impl CloudflareAuth {
    /// 创建新的 Cloudflare 认证器
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
    
    /// 验证 Token 有效性并返回验证结果
    pub async fn verify(&self) -> AuthResult<ValidationResult> {
        // 只调用一次 verify API 获取完整的 token 信息
        let token_details = self.get_token_details().await?;
        
        // 检查 token 状态
        if token_details.status != "active" {
            return Err(AuthError::InvalidToken(format!("Token 状态无效: {}", token_details.status)));
        }
        
        // 构建验证结果，提取权限信息
        let permissions = self.extract_permissions(&token_details);
        
        // 输出验证成功的详细信息
        info!("🎉 Cloudflare Token 验证成功!");
        info!("📋 Token ID: {}", token_details.id);
        if let Some(name) = &token_details.name {
            info!("📋 Token 名称: {}", name);
        }
        info!("📋 Token 状态: {}", token_details.status);
        if let Some(expires_on) = &token_details.expires_on {
            info!("⏰ Token 过期时间: {}", expires_on);
        } else {
            info!("⏰ Token 永不过期");
        }
        if !permissions.is_empty() {
            info!("🔐 Token 权限:");
            for permission in &permissions {
                info!("   - {}", permission);
            }
        }
        
        Ok(ValidationResult::Cloudflare {
            account_id: token_details.id.clone(),
            email: None, // 不再尝试获取用户邮箱
            permissions,
        })
    }
    
    /// 验证 Token 基本有效性
    async fn verify_token(&self) -> AuthResult<TokenVerifyResult> {
        let url = format!("{}/user/tokens/verify", CLOUDFLARE_API_BASE);
        let token = self.credential.expose();
        let masked_token = if token.len() > 8 {
            format!("{}**", &token[..8])
        } else {
            "****".to_string()
        };
        
        rat_logger::debug!("🔗 请求 Cloudflare API: {}", url);
        rat_logger::debug!("🔑 使用 Token: {}", masked_token);
        
        let response = self.client
            .get(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| {
                error!("❌ HTTP 请求失败: {}", e);
                AuthError::ServiceError(format!("请求失败: {}", e))
            })?;
        
        let status = response.status();
        rat_logger::debug!("📡 HTTP 响应状态: {}", status);
        
        let body: CloudflareResponse<TokenVerifyResult> = response
            .json()
            .await
            .map_err(|e| {
                error!("❌ 解析响应失败: {}", e);
                AuthError::ServiceError(format!("解析响应失败: {}", e))
            })?;
        
        if !body.success {
            let error_msg = body.errors
                .first()
                .map(|e| e.message.clone())
                .unwrap_or_else(|| "Unknown error".to_string());
            
            return match status.as_u16() {
                401 => Err(AuthError::InvalidToken("Invalid or expired token".to_string())),
                403 => Err(AuthError::InsufficientPermissions),
                429 => Err(AuthError::RateLimitExceeded),
                _ => Err(AuthError::ServiceError(error_msg)),
            };
        }
        
        body.result.ok_or_else(|| {
            AuthError::InvalidResponse
        })
    }
    

    /// 获取 Token 详细信息
    async fn get_token_details(&self) -> AuthResult<TokenDetails> {
        let url = format!("{}/user/tokens/verify", CLOUDFLARE_API_BASE);
        let token = self.credential.expose();
        let masked_token = if token.len() > 8 {
            format!("{}**", &token[..8])
        } else {
            "****".to_string()
        };
        
        rat_logger::debug!("🔗 请求 Cloudflare API: {}", url);
        rat_logger::debug!("🔑 使用 Token: {}", masked_token);
        
        let response = self.client
            .get(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| {
                error!("❌ HTTP 请求失败: {}", e);
                AuthError::ServiceError(format!("Request failed: {}", e))
            })?;
        
        let status = response.status();
        rat_logger::debug!("📡 HTTP 响应状态: {}", status);
        
        // 先获取原始响应文本用于调试
        let response_text = response
            .text()
            .await
            .map_err(|e| {
                error!("❌ 读取响应文本失败: {}", e);
                AuthError::ServiceError(format!("Failed to read response text: {}", e))
            })?;
        
        rat_logger::debug!("📋 原始 API 响应: {}", response_text);
        
        // 解析 JSON 响应
        let body: CloudflareResponse<TokenDetails> = serde_json::from_str(&response_text)
            .map_err(|e| {
                error!("❌ 解析响应失败: {}", e);
                error!("📋 响应内容: {}", response_text);
                AuthError::ServiceError(format!("Failed to parse response: {}", e))
            })?;
        
        if !body.success {
            let error_msg = body.errors
                .first()
                .map(|e| e.message.clone())
                .unwrap_or_else(|| "获取令牌详情失败".to_string());
            
            // 如果有错误消息，记录详细信息
            if !body.messages.is_empty() {
                for msg in &body.messages {
                    rat_logger::debug!("📋 API 消息: {} (代码: {})", msg.message, msg.code);
                }
            }
            
            return Err(AuthError::ServiceError(error_msg));
        }
        
        // 记录成功消息
        if !body.messages.is_empty() {
            for msg in &body.messages {
                rat_logger::debug!("✅ API 消息: {} (代码: {})", msg.message, msg.code);
            }
        }
        
        body.result.ok_or_else(|| {
            AuthError::InvalidResponse
        })
    }
    
    /// 验证必要的权限
    fn validate_permissions(&self, token_details: &TokenDetails) -> AuthResult<()> {
        let required_permissions = [
            "com.cloudflare.api.account.zone:read",
            "com.cloudflare.api.account.zone.dns_record:edit",
        ];
        
        let available_permissions: Vec<String> = token_details
            .policies
            .as_ref()
            .map(|policies| {
                policies
                    .iter()
                    .flat_map(|policy| &policy.permission_groups)
                    .map(|perm| perm.id.clone())
                    .collect()
            })
            .unwrap_or_default();
        
        for required in &required_permissions {
            if !available_permissions.iter().any(|perm| perm.contains(required)) {
                return Err(AuthError::InsufficientPermissions);
            }
        }
        
        Ok(())
    }
    
    /// 提取权限列表
    fn extract_permissions(&self, token_details: &TokenDetails) -> Vec<String> {
        token_details
            .policies
            .as_ref()
            .map(|policies| {
                policies
                    .iter()
                    .flat_map(|policy| &policy.permission_groups)
                    .map(|perm| perm.name.clone())
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// 检查 Token 是否即将过期
    pub async fn check_token_expiry(&self) -> AuthResult<Option<chrono::DateTime<chrono::Utc>>> {
        let token_details = self.get_token_details().await?;
        
        if let Some(expires_on) = token_details.expires_on {
            match chrono::DateTime::parse_from_rfc3339(&expires_on) {
                Ok(expiry) => Ok(Some(expiry.with_timezone(&chrono::Utc))),
                Err(_) => Ok(None),
            }
        } else {
            Ok(None) // Token 永不过期
        }
    }
    
    /// 获取账户的区域列表（用于验证权限）
    pub async fn list_zones(&self) -> AuthResult<Vec<String>> {
        let url = format!("{}/zones", CLOUDFLARE_API_BASE);
        let token = self.credential.expose();
        let masked_token = if token.len() > 8 {
            format!("{}**", &token[..8])
        } else {
            "****".to_string()
        };
        
        rat_logger::debug!("🔗 请求 Cloudflare API: {}", url);
        rat_logger::debug!("🔑 使用 Token: {}", masked_token);
        
        let response = self.client
            .get(&url)
            .bearer_auth(token)
            .query(&[("per_page", "5")]) // 只获取前5个区域用于测试
            .send()
            .await
            .map_err(|e| {
                error!("❌ HTTP 请求失败: {}", e);
                AuthError::ServiceError(format!("Request failed: {}", e))
            })?;
        
        #[derive(Deserialize)]
        struct Zone {
            id: String,
            name: String,
        }
        
        let status = response.status();
        rat_logger::debug!("📡 HTTP 响应状态: {}", status);
        
        let body: CloudflareResponse<Vec<Zone>> = response
            .json()
            .await
            .map_err(|e| {
                error!("❌ 解析响应失败: {}", e);
                AuthError::ServiceError(format!("Failed to parse response: {}", e))
            })?;
        
        if !body.success {
            return Err(AuthError::ServiceError("列出区域失败".to_string()));
        }
        
        Ok(body.result
            .unwrap_or_default()
            .into_iter()
            .map(|zone| zone.name)
            .collect())
    }
}

/// 便捷函数：快速验证 Cloudflare Token
pub async fn verify_cloudflare_token(token: &str) -> AuthResult<ValidationResult> {
    use crate::auth::Provider;
    let credential = SecureCredential::new(token.to_string(), Provider::Cloudflare);
    let auth = CloudflareAuth::new(credential);
    auth.verify().await
}

/// 便捷函数：检查 Token 权限
pub async fn check_cloudflare_permissions(token: &str) -> AuthResult<Vec<String>> {
    use crate::auth::Provider;
    let credential = SecureCredential::new(token.to_string(), Provider::Cloudflare);
    let auth = CloudflareAuth::new(credential);
    
    match auth.verify().await? {
        ValidationResult::Cloudflare { permissions, .. } => Ok(permissions),
        _ => unreachable!(),
    }
}