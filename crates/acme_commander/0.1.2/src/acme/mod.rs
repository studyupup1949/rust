//! ACME 协议核心模块
//! 实现 ACME v2 协议的客户端功能

use crate::auth::{EabCredentials, ValidationResult};
use crate::crypto::KeyPair;
use crate::error::{AcmeError, AcmeResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use url::Url;

pub mod account;
pub mod authorization;
pub mod certificate;
pub mod challenge;
pub mod directory;
pub mod jws;
pub mod order;

// 重新导出主要类型
pub use account::{Account, AccountStatus, AccountManager, register_or_find_account};
pub use authorization::{Authorization, AuthorizationStatus, Identifier, IdentifierType};
pub use certificate::{CertificateManager, CertificateRequest, CertificateInfo, CertificateChain};
pub use challenge::{Challenge, ChallengeStatus, ChallengeType, ChallengeRecoveryManager, recover_challenge_from_authorization};
pub use directory::{Directory, DirectoryMeta, AcmeServer, DirectoryManager};
pub use jws::Jws;
pub use order::{Order, OrderStatus, OrderManager};

/// ACME 客户端
#[derive(Debug)]
pub struct AcmeClient {
    /// HTTP 客户端
    client: Client,
    /// ACME 目录 URL
    directory_url: Url,
    /// ACME 目录信息
    directory: Option<Directory>,
    /// 账户密钥对
    account_key: KeyPair,
    /// 账户 URL（注册后获得）
    account_url: Option<String>,
    /// Nonce 缓存
    nonce: Option<String>,
    /// 是否为 dry-run 模式
    dry_run: bool,
}

/// ACME 错误响应
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AcmeErrorResponse {
    /// 错误类型
    #[serde(rename = "type")]
    pub error_type: String,
    /// 错误详情
    pub detail: String,
    /// 错误状态码
    pub status: Option<u16>,
    /// 错误实例
    pub instance: Option<String>,
    /// 子问题
    pub subproblems: Option<Vec<AcmeErrorResponse>>,
}

/// ACME 客户端配置
#[derive(Debug, Clone)]
pub struct AcmeConfig {
    /// ACME 目录 URL
    pub directory_url: String,
    /// 联系邮箱
    pub contact_email: Option<String>,
    /// 是否同意服务条款
    pub terms_of_service_agreed: bool,
    /// EAB 凭证（用于需要外部账户绑定的 CA）
    pub eab_credentials: Option<EabCredentials>,
    /// HTTP 超时时间
    pub timeout: Duration,
    /// 是否为 dry-run 模式
    pub dry_run: bool,
    /// 用户代理
    pub user_agent: String,
}

impl Default for AcmeConfig {
    fn default() -> Self {
        Self {
            directory_url: "https://acme-v02.api.letsencrypt.org/directory".to_string(),
            contact_email: None,
            terms_of_service_agreed: false,
            eab_credentials: None,
            timeout: Duration::from_secs(30),
            dry_run: false,
            user_agent: "acme-commander/0.1.0".to_string(),
        }
    }
}

impl AcmeConfig {
    /// 创建新的 ACME 配置
    pub fn new(directory_url: String, _account_key: KeyPair) -> Self {
        Self {
            directory_url,
            ..Default::default()
        }
    }
}

impl AcmeClient {
    /// 创建新的 ACME 客户端
    pub fn new(config: AcmeConfig, account_key: KeyPair) -> AcmeResult<Self> {
        let directory_url = Url::parse(&config.directory_url)
            .map_err(|e| AcmeError::InvalidUrl(format!("无效的目录 URL: {}", e)))?;
        
        let client = Client::builder()
            .timeout(config.timeout)
            .user_agent(&config.user_agent)
            .build()
            .map_err(|e| AcmeError::HttpError(format!("创建 HTTP 客户端失败: {}", e)))?;
        
        Ok(Self {
            client,
            directory_url,
            directory: None,
            account_key,
            account_url: None,
            nonce: None,
            dry_run: config.dry_run,
        })
    }
    
    /// 获取 ACME 目录信息
    pub async fn get_directory(&mut self) -> AcmeResult<&Directory> {
        if self.directory.is_none() {
            let directory = self.fetch_directory().await?;
            self.directory = Some(directory);
        }
        
        Ok(self.directory.as_ref().unwrap())
    }
    
    /// 从服务器获取目录信息
    async fn fetch_directory(&self) -> AcmeResult<Directory> {
        let response = self.client
            .get(self.directory_url.clone())
            .send()
            .await
            .map_err(|e| AcmeError::HttpError(format!("获取目录失败: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(AcmeError::HttpError(
                format!("目录请求失败，状态码: {}", response.status())
            ));
        }
        
        let directory: Directory = response
            .json()
            .await
            .map_err(|e| AcmeError::JsonError(format!("解析目录失败: {}", e)))?;
        
        Ok(directory)
    }
    
    /// 获取新的 nonce
    pub async fn get_nonce(&mut self) -> AcmeResult<String> {
        if let Some(nonce) = self.nonce.take() {
            return Ok(nonce);
        }
        
        let directory = self.get_directory().await?;
        let new_nonce_url = directory.new_nonce.clone();
        
        let response = self.client
            .head(&new_nonce_url)
            .send()
            .await
            .map_err(|e| AcmeError::HttpError(format!("获取 nonce 失败: {}", e)))?;
        
        let nonce = response
            .headers()
            .get("replay-nonce")
            .ok_or_else(|| AcmeError::ProtocolError("缺少 replay-nonce 头".to_string()))?
            .to_str()
            .map_err(|e| AcmeError::ProtocolError(format!("无效的 nonce 头: {}", e)))?
            .to_string();
        
        Ok(nonce)
    }
    
    /// 设置 nonce（从响应头中提取）
    pub fn set_nonce_from_response(&mut self, response: &reqwest::Response) {
        if let Some(nonce) = response.headers().get("replay-nonce") {
            if let Ok(nonce_str) = nonce.to_str() {
                self.nonce = Some(nonce_str.to_string());
            }
        }
    }
    
    /// 检查是否为 dry-run 模式
    pub fn is_dry_run(&self) -> bool {
        self.dry_run
    }
    
    /// 设置 dry-run 模式
    pub fn set_dry_run(&mut self, dry_run: bool) {
        self.dry_run = dry_run;
    }
    
    /// 获取账户密钥
    pub fn account_key(&self) -> &KeyPair {
        &self.account_key
    }
    
    /// 获取账户 URL
    pub fn account_url(&self) -> Option<&str> {
        self.account_url.as_deref()
    }
    
    /// 设置账户 URL
    pub fn set_account_url(&mut self, url: String) {
        self.account_url = Some(url);
    }
    
    /// 获取 HTTP 客户端引用
    pub fn client(&self) -> &Client {
        &self.client
    }
    
    /// 获取授权信息
    pub async fn get_authorization(&mut self, auth_url: &str) -> AcmeResult<Authorization> {
        use crate::acme::jws::JwsBuilder;
        
        // 获取 nonce
        let nonce = self.get_nonce().await?;
        let account_url = self.account_url()
            .ok_or_else(|| AcmeError::ProtocolError("没有可用的账户 URL".to_string()))?
            .to_string();
        
        // 创建 JWS 构建器
        let jws_builder = JwsBuilder::new(self.account_key.clone());
        
        // 创建 POST-as-GET 请求
        let jws = jws_builder.create_post_as_get(
            &nonce,
            auth_url,
            &account_url,
        )?;
        
        // 发送请求
        let response = self.client
            .post(auth_url)
            .header("Content-Type", "application/jose+json")
            .json(&jws)
            .send()
            .await
            .map_err(|e| AcmeError::HttpError(format!("授权请求失败: {}", e)))?;
        
        // 更新 nonce
        self.set_nonce_from_response(&response);
        
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "未知错误".to_string());
            return Err(AcmeError::HttpError(
                format!("授权请求失败，状态码 {}: {}", status, error_text)
            ));
        }
        
        // 获取原始响应文本用于调试
        let response_text = response.text().await
            .map_err(|e| AcmeError::HttpError(format!("读取响应失败: {}", e)))?;
        
        // 在 DEBUG 级别打印原始 JSON 响应
        rat_logger::debug!("🔧 📋 授权 API 原始响应: {}", response_text);
        
        let authorization: Authorization = serde_json::from_str(&response_text)
            .map_err(|e| AcmeError::JsonError(format!("解析授权失败: {}", e)))?;
        
        Ok(authorization)
    }
    
    /// 完成订单（提交 CSR）
    pub async fn finalize_order(&mut self, finalize_url: &str, csr_der: &[u8]) -> AcmeResult<Order> {
        use crate::acme::jws::JwsBuilder;
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
        use serde_json::json;
        
        // Base64URL 编码 CSR
        let csr_b64 = URL_SAFE_NO_PAD.encode(csr_der);
        
        let finalize_request = json!({
            "csr": csr_b64
        });
        
        // 获取 nonce
        let nonce = self.get_nonce().await?;
        let account_url = self.account_url()
            .ok_or_else(|| AcmeError::ProtocolError("没有可用的账户 URL".to_string()))?
            .to_string();
        
        // 创建 JWS 构建器
        let jws_builder = JwsBuilder::new(self.account_key.clone());
        
        // 创建 JWS
        let jws = jws_builder.create_for_existing_account(
            &nonce,
            finalize_url,
            &account_url,
            &finalize_request,
        )?;
        
        // 发送请求
        let response = self.client
            .post(finalize_url)
            .header("Content-Type", "application/jose+json")
            .json(&jws)
            .send()
            .await
            .map_err(|e| AcmeError::HttpError(format!("订单完成请求失败: {}", e)))?;
        
        // 更新 nonce
        self.set_nonce_from_response(&response);
        
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "未知错误".to_string());
            return Err(AcmeError::HttpError(
                format!("订单完成请求失败，状态码 {}: {}", status, error_text)
            ));
        }
        
        let order: Order = response.json().await
            .map_err(|e| AcmeError::JsonError(format!("解析订单失败: {}", e)))?;
        
        Ok(order)
    }
    
    /// 等待订单准备就绪
    pub async fn wait_for_order_ready(
        &mut self,
        order_url: &str,
        max_attempts: u32,
        delay: std::time::Duration,
    ) -> AcmeResult<Order> {
        use crate::acme::jws::JwsBuilder;
        
        for attempt in 1..=max_attempts {
            // 获取订单状态
            let nonce = self.get_nonce().await?;
            let account_url = self.account_url()
                .ok_or_else(|| AcmeError::ProtocolError("没有可用的账户 URL".to_string()))?
                .to_string();
            
            let jws_builder = JwsBuilder::new(self.account_key.clone());
            let jws = jws_builder.create_post_as_get(
                &nonce,
                order_url,
                &account_url,
            )?;
            
            let response = self.client
                .post(order_url)
                .header("Content-Type", "application/jose+json")
                .json(&jws)
                .send()
                .await
                .map_err(|e| AcmeError::HttpError(format!("获取订单状态失败: {}", e)))?;
            
            self.set_nonce_from_response(&response);
            
            let status = response.status();
            if !status.is_success() {
                let error_text = response.text().await
                    .unwrap_or_else(|_| "未知错误".to_string());
                return Err(AcmeError::HttpError(
                    format!("获取订单状态失败，状态码 {}: {}", status, error_text)
                ));
            }
            
            let order: Order = response.json().await
                .map_err(|e| AcmeError::JsonError(format!("解析订单失败: {}", e)))?;
            
            match order.status {
                OrderStatus::Ready | OrderStatus::Valid => {
                    return Ok(order);
                }
                OrderStatus::Invalid => {
                    return Err(AcmeError::OrderFailed("订单无效".to_string()));
                }
                OrderStatus::Pending | OrderStatus::Processing => {
                    if attempt < max_attempts {
                        tokio::time::sleep(delay).await;
                    } else {
                        return Err(AcmeError::Timeout(
                            format!("经过 {} 次尝试后订单仍未准备就绪", max_attempts)
                        ));
                    }
                }
            }
        }
        
        Err(AcmeError::Timeout(
            format!("经过 {} 次尝试后订单仍未准备就绪", max_attempts)
        ))
    }
    
    /// 下载证书
    pub async fn download_certificate(&mut self, cert_url: &str) -> AcmeResult<String> {
        use crate::acme::jws::JwsBuilder;
        
        // 获取 nonce
        let nonce = self.get_nonce().await?;
        let account_url = self.account_url()
            .ok_or_else(|| AcmeError::ProtocolError("没有可用的账户 URL".to_string()))?;
        
        // 创建 POST-as-GET 请求
        let jws_builder = JwsBuilder::new(self.account_key.clone());
        let jws = jws_builder.create_post_as_get(
            &nonce,
            cert_url,
            account_url,
        )?;
        
        // 发送请求
        let response = self.client
            .post(cert_url)
            .header("Content-Type", "application/jose+json")
            .json(&jws)
            .send()
            .await
            .map_err(|e| AcmeError::HttpError(format!("证书下载失败: {}", e)))?;
        
        // 更新 nonce
        self.set_nonce_from_response(&response);
        
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "未知错误".to_string());
            return Err(AcmeError::HttpError(
                format!("证书下载失败，状态码 {}: {}", status, error_text)
            ));
        }
        
        let certificate = response.text().await
            .map_err(|e| AcmeError::HttpError(format!("读取证书失败: {}", e)))?;
        
        Ok(certificate)
    }
}

/// 从验证结果创建 ACME 配置
pub fn create_acme_config_from_validation(
    validation_result: &ValidationResult,
    contact_email: Option<String>,
    dry_run: bool,
) -> AcmeConfig {
    let mut config = AcmeConfig {
        contact_email,
        terms_of_service_agreed: true,
        dry_run,
        ..Default::default()
    };
    
    match validation_result {
        ValidationResult::Cloudflare { .. } => {
            // Cloudflare 使用 Let's Encrypt
            if dry_run {
                config.directory_url = "https://acme-staging-v02.api.letsencrypt.org/directory".to_string();
            } else {
                config.directory_url = "https://acme-v02.api.letsencrypt.org/directory".to_string();
            }
        }
        ValidationResult::ZeroSsl { eab_credentials, .. } => {
            // ZeroSSL 需要 EAB 凭证
            config.directory_url = "https://acme.zerossl.com/v2/DV90".to_string();
            config.eab_credentials = eab_credentials.clone();
        }
    }
    
    config
}

/// 验证域名格式
pub fn validate_domain(domain: &str) -> AcmeResult<()> {
    if domain.is_empty() {
        return Err(AcmeError::InvalidDomain("域名不能为空".to_string()));
    }
    
    if domain.len() > 253 {
        return Err(AcmeError::InvalidDomain("域名过长".to_string()));
    }
    
    // 基本的域名格式检查
    if !domain.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '*') {
        return Err(AcmeError::InvalidDomain("域名包含无效字符".to_string()));
    }
    
    // 检查是否以点开头或结尾
    if domain.starts_with('.') || domain.ends_with('.') {
        return Err(AcmeError::InvalidDomain("域名不能以点开头或结尾".to_string()));
    }
    
    // 检查连续的点
    if domain.contains("..") {
        return Err(AcmeError::InvalidDomain("域名不能包含连续的点".to_string()));
    }
    
    Ok(())
}

/// 创建标识符
pub fn create_identifier(domain: &str) -> AcmeResult<Identifier> {
    validate_domain(domain)?;
    
    // 检查是否为 IP 地址
    if domain.parse::<std::net::IpAddr>().is_ok() {
        Ok(Identifier {
            identifier_type: IdentifierType::Ip,
            value: domain.to_string(),
        })
    } else {
        Ok(Identifier {
            identifier_type: IdentifierType::Dns,
            value: domain.to_string(),
        })
    }
}
