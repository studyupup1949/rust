//! ACME 挑战处理模块
//! 处理 HTTP-01、DNS-01 和 TLS-ALPN-01 挑战

use crate::acme::jws::JwsBuilder;
use crate::acme::{AcmeClient, Authorization, AuthorizationStatus};
use crate::error::{AcmeError, AcmeResult};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::Duration;
use url::Url;

/// 挑战类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChallengeType {
    /// HTTP-01 挑战
    #[serde(rename = "http-01")]
    Http01,
    /// DNS-01 挑战
    #[serde(rename = "dns-01")]
    Dns01,
    /// TLS-ALPN-01 挑战
    #[serde(rename = "tls-alpn-01")]
    TlsAlpn01,
}

/// 挑战状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChallengeStatus {
    /// 待处理
    Pending,
    /// 处理中
    Processing,
    /// 有效
    Valid,
    /// 无效
    Invalid,
}

/// 挑战
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Challenge {
    /// 挑战类型
    #[serde(rename = "type")]
    pub challenge_type: ChallengeType,
    /// 挑战 URL
    pub url: String,
    /// 挑战状态
    pub status: ChallengeStatus,
    /// 挑战令牌
    pub token: String,
    /// 验证时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validated: Option<String>,
    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<serde_json::Value>,
}

/// 挑战响应请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResponse {
    /// 空对象，表示准备好接受挑战
    #[serde(flatten)]
    pub _empty: HashMap<String, Value>,
}

/// HTTP-01 挑战信息
#[derive(Debug, Clone)]
pub struct Http01Challenge {
    /// 挑战令牌
    pub token: String,
    /// 密钥授权
    pub key_authorization: String,
    /// 挑战 URL
    pub url: String,
    /// 验证文件路径
    pub file_path: String,
    /// 验证文件内容
    pub file_content: String,
}

/// DNS-01 挑战信息
#[derive(Debug, Clone)]
pub struct Dns01Challenge {
    /// 挑战令牌
    pub token: String,
    /// 密钥授权
    pub key_authorization: String,
    /// 挑战 URL
    pub url: String,
    /// DNS 记录名称
    pub record_name: String,
    /// DNS 记录值
    pub record_value: String,
    /// DNS 记录类型（始终为 TXT）
    pub record_type: String,
}

/// TLS-ALPN-01 挑战信息
#[derive(Debug, Clone)]
pub struct TlsAlpn01Challenge {
    /// 挑战令牌
    pub token: String,
    /// 密钥授权
    pub key_authorization: String,
    /// 挑战 URL
    pub url: String,
    /// 证书指纹
    pub certificate_thumbprint: String,
}

/// 挑战管理器
#[derive(Debug)]
pub struct ChallengeManager<'a> {
    /// ACME 客户端引用
    client: &'a mut AcmeClient,
    /// JWS 构建器
    jws_builder: JwsBuilder,
}

impl<'a> ChallengeManager<'a> {
    /// 创建新的挑战管理器
    pub fn new(client: &'a mut AcmeClient) -> Self {
        let jws_builder = JwsBuilder::new(client.account_key().clone());
        Self {
            client,
            jws_builder,
        }
    }
    
    /// 处理授权的所有挑战
    pub async fn process_authorization(
        &mut self,
        authorization: &Authorization,
        preferred_challenge_type: Option<ChallengeType>,
    ) -> AcmeResult<Challenge> {
        // 选择挑战类型
        let challenge = self.select_challenge(authorization, preferred_challenge_type)?;
        
        // 准备挑战
        let challenge_info = self.prepare_challenge(&challenge)?;
        
        // 在 dry-run 模式下，只显示挑战信息
        if self.client.is_dry_run() {
            self.display_challenge_info(&challenge_info)?;
            return Ok(challenge);
        }
        
        // 通知用户设置挑战
        self.display_challenge_setup_instructions(&challenge_info)?;
        
        // 等待用户确认（在实际应用中可能需要自动化）
        crate::acme_info!("请设置挑战并按 Enter 键继续...");
        
        // 响应挑战
        let updated_challenge = self.respond_to_challenge(&challenge).await?;
        
        Ok(updated_challenge)
    }
    
    /// 选择合适的挑战类型
    fn select_challenge(
        &self,
        authorization: &Authorization,
        preferred_type: Option<ChallengeType>,
    ) -> AcmeResult<Challenge> {
        // 如果指定了首选类型，优先选择
        if let Some(preferred) = preferred_type {
            for challenge in &authorization.challenges {
                if challenge.challenge_type == preferred {
                    return Ok(challenge.clone());
                }
            }
        }
        
        // 按优先级选择：DNS-01 > HTTP-01 > TLS-ALPN-01
        let priority_order = [ChallengeType::Dns01, ChallengeType::Http01, ChallengeType::TlsAlpn01];
        
        for challenge_type in &priority_order {
            for challenge in &authorization.challenges {
                if challenge.challenge_type == *challenge_type {
                    return Ok(challenge.clone());
                }
            }
        }
        
        Err(AcmeError::ProtocolError(
            "No supported challenge type found".to_string()
        ))
    }
    
    /// 准备挑战信息
    pub fn prepare_challenge(&self, challenge: &Challenge) -> AcmeResult<ChallengeInfo> {
        let key_authorization = self.create_key_authorization(&challenge.token)?;
        
        match challenge.challenge_type {
            ChallengeType::Http01 => {
                let http01 = Http01Challenge {
                    token: challenge.token.clone(),
                    key_authorization: key_authorization.clone(),
                    url: challenge.url.clone(),
                    file_path: format!("/.well-known/acme-challenge/{}", challenge.token),
                    file_content: key_authorization,
                };
                Ok(ChallengeInfo::Http01(http01))
            }
            ChallengeType::Dns01 => {
                let dns_value = self.create_dns_challenge_value(&key_authorization)?;
                let dns01 = Dns01Challenge {
                    token: challenge.token.clone(),
                    key_authorization,
                    url: challenge.url.clone(),
                    record_name: "_acme-challenge".to_string(),
                    record_value: dns_value,
                    record_type: "TXT".to_string(),
                };
                Ok(ChallengeInfo::Dns01(dns01))
            }
            ChallengeType::TlsAlpn01 => {
                let thumbprint = self.create_certificate_thumbprint(&key_authorization)?;
                let tls_alpn01 = TlsAlpn01Challenge {
                    token: challenge.token.clone(),
                    key_authorization,
                    url: challenge.url.clone(),
                    certificate_thumbprint: thumbprint,
                };
                Ok(ChallengeInfo::TlsAlpn01(tls_alpn01))
            }
        }
    }
    
    /// 创建密钥授权
    fn create_key_authorization(&self, token: &str) -> AcmeResult<String> {
        let account_jwk = self.client.account_key().to_jwk()
            .map_err(|e| AcmeError::CryptoError(format!("Failed to create JWK: {}", e)))?;
        
        let jwk_thumbprint = self.create_jwk_thumbprint(&account_jwk)?;
        
        Ok(format!("{}.{}", token, jwk_thumbprint))
    }
    
    /// 创建 JWK 指纹
    fn create_jwk_thumbprint(&self, jwk: &Value) -> AcmeResult<String> {
        // 提取 JWK 的关键字段并排序
        let mut thumbprint_data = serde_json::Map::new();
        
        if let Some(obj) = jwk.as_object() {
            // 按字母顺序添加必需字段
            if let Some(crv) = obj.get("crv") {
                thumbprint_data.insert("crv".to_string(), crv.clone());
            }
            if let Some(kty) = obj.get("kty") {
                thumbprint_data.insert("kty".to_string(), kty.clone());
            }
            if let Some(x) = obj.get("x") {
                thumbprint_data.insert("x".to_string(), x.clone());
            }
            if let Some(y) = obj.get("y") {
                thumbprint_data.insert("y".to_string(), y.clone());
            }
        }
        
        let thumbprint_json = serde_json::to_string(&thumbprint_data)
            .map_err(|e| AcmeError::JsonError(format!("Failed to serialize JWK thumbprint: {}", e)))?;
        
        let mut hasher = Sha256::new();
        hasher.update(thumbprint_json.as_bytes());
        let hash = hasher.finalize();
        
        Ok(URL_SAFE_NO_PAD.encode(&hash))
    }
    
    /// 创建 DNS 挑战值
    fn create_dns_challenge_value(&self, key_authorization: &str) -> AcmeResult<String> {
        let mut hasher = Sha256::new();
        hasher.update(key_authorization.as_bytes());
        let hash = hasher.finalize();
        
        Ok(URL_SAFE_NO_PAD.encode(&hash))
    }
    
    /// 创建证书指纹（用于 TLS-ALPN-01）
    fn create_certificate_thumbprint(&self, key_authorization: &str) -> AcmeResult<String> {
        let mut hasher = Sha256::new();
        hasher.update(key_authorization.as_bytes());
        let hash = hasher.finalize();
        
        Ok(hex::encode(&hash))
    }
    
    /// 响应挑战
    pub async fn respond_to_challenge(&mut self, challenge: &Challenge) -> AcmeResult<Challenge> {
        // 获取 nonce
        let nonce = self.client.get_nonce().await?;
        let account_url = self.client.account_url()
            .ok_or_else(|| AcmeError::ProtocolError("No account URL available".to_string()))?;
        
        // 创建挑战响应
        let response_payload = json!({});
        
        // 创建 JWS
        let jws = self.jws_builder.create_for_existing_account(
            &nonce,
            &challenge.url,
            account_url,
            &response_payload,
        )?;
        
        // 发送挑战响应
        let response = self.client.client()
            .post(&challenge.url)
            .header("Content-Type", "application/jose+json")
            .json(&jws)
            .send()
            .await
            .map_err(|e| AcmeError::HttpError(format!("Challenge response failed: {}", e)))?;
        
        // 更新 nonce
        self.client.set_nonce_from_response(&response);
        
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AcmeError::HttpError(
                format!("Challenge response failed with status {}: {}", status, error_text)
            ));
        }
        
        let updated_challenge: Challenge = response.json().await
            .map_err(|e| AcmeError::JsonError(format!("Failed to parse challenge response: {}", e)))?;
        
        if self.client.is_dry_run() {
            crate::acme_info!("[演练模式] 挑战将被响应");
        } else {
            crate::acme_info!("挑战响应发送成功");
        }
        
        Ok(updated_challenge)
    }
    
    /// 等待挑战完成
    pub async fn wait_for_challenge_completion(
        &mut self,
        challenge_url: &str,
        max_attempts: u32,
        delay: Duration,
    ) -> AcmeResult<Challenge> {
        for attempt in 1..=max_attempts {
            let challenge = self.get_challenge_status(challenge_url).await?;
            
            match challenge.status {
                ChallengeStatus::Valid => {
                    if self.client.is_dry_run() {
                        crate::acme_info!("[演练模式] 挑战将有效");
                    } else {
                        crate::acme_info!("挑战完成成功");
                    }
                    return Ok(challenge);
                }
                ChallengeStatus::Invalid => {
                    let error_msg = challenge.error
                        .map(|e| format!("Challenge failed: {:?}", e))
                        .unwrap_or_else(|| "Challenge failed with unknown error".to_string());
                    return Err(AcmeError::ChallengeValidationFailed(error_msg));
                }
                ChallengeStatus::Pending | ChallengeStatus::Processing => {
                    if attempt < max_attempts {
                        crate::acme_info!(
                            "Challenge status: {:?}, waiting {} seconds before retry (attempt {}/{})",
                            challenge.status, delay.as_secs(), attempt, max_attempts
                        );
                        tokio::time::sleep(delay).await;
                    } else {
                        return Err(AcmeError::Timeout(
                            format!("Challenge not completed after {} attempts", max_attempts)
                        ));
                    }
                }
            }
        }
        
        Err(AcmeError::Timeout(
            format!("Challenge not completed after {} attempts", max_attempts)
        ))
    }
    
    /// 获取挑战状态
    async fn get_challenge_status(&mut self, challenge_url: &str) -> AcmeResult<Challenge> {
        // 获取 nonce
        let nonce = self.client.get_nonce().await?;
        let account_url = self.client.account_url()
            .ok_or_else(|| AcmeError::ProtocolError("No account URL available".to_string()))?;
        
        // 创建 POST-as-GET 请求
        let jws = self.jws_builder.create_post_as_get(
            &nonce,
            challenge_url,
            account_url,
        )?;
        
        // 发送请求
        let response = self.client.client()
            .post(challenge_url)
            .header("Content-Type", "application/jose+json")
            .json(&jws)
            .send()
            .await
            .map_err(|e| AcmeError::HttpError(format!("Challenge status request failed: {}", e)))?;
        
        // 更新 nonce
        self.client.set_nonce_from_response(&response);
        
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AcmeError::HttpError(
                format!("Challenge status request failed with status {}: {}", status, error_text)
            ));
        }
        
        let challenge: Challenge = response.json().await
            .map_err(|e| AcmeError::JsonError(format!("Failed to parse challenge status: {}", e)))?;
        
        Ok(challenge)
    }
    
    /// 显示挑战信息（dry-run 模式）
    fn display_challenge_info(&self, challenge_info: &ChallengeInfo) -> AcmeResult<()> {
        match challenge_info {
            ChallengeInfo::Http01(http01) => {
                crate::acme_info!("[演练模式] HTTP-01 挑战信息:");
            crate::acme_info!("  文件路径: {}", http01.file_path);
            crate::acme_info!("  文件内容: {}", http01.file_content);
            crate::acme_info!("  挑战 URL: {}", http01.url);
            }
            ChallengeInfo::Dns01(dns01) => {
                crate::acme_info!("[演练模式] DNS-01 挑战信息:");
            crate::acme_info!("  记录名称: {}", dns01.record_name);
            crate::acme_info!("  记录类型: {}", dns01.record_type);
            crate::acme_info!("  记录值: {}", dns01.record_value);
            crate::acme_info!("  挑战 URL: {}", dns01.url);
            }
            ChallengeInfo::TlsAlpn01(tls_alpn01) => {
                crate::acme_info!("[演练模式] TLS-ALPN-01 挑战信息:");
            crate::acme_info!("  证书指纹: {}", tls_alpn01.certificate_thumbprint);
            crate::acme_info!("  挑战 URL: {}", tls_alpn01.url);
            }
        }
        Ok(())
    }
    
    /// 显示挑战设置说明
    fn display_challenge_setup_instructions(&self, challenge_info: &ChallengeInfo) -> AcmeResult<()> {
        match challenge_info {
            ChallengeInfo::Http01(http01) => {
                crate::acme_info!("HTTP-01 挑战设置说明:");
            crate::acme_info!("1. 在以下位置创建文件: {}", http01.file_path);
            crate::acme_info!("2. 文件内容应为: {}", http01.file_content);
            crate::acme_info!("3. 确保文件可通过 HTTP 访问");
            }
            ChallengeInfo::Dns01(dns01) => {
                crate::acme_info!("DNS-01 挑战设置说明:");
            crate::acme_info!("1. 创建 TXT 记录: {}", dns01.record_name);
            crate::acme_info!("2. 记录值: {}", dns01.record_value);
            crate::acme_info!("3. 等待 DNS 传播");
            }
            ChallengeInfo::TlsAlpn01(tls_alpn01) => {
                crate::acme_info!("TLS-ALPN-01 挑战设置说明:");
            crate::acme_info!("1. 配置带有 ALPN 扩展的 TLS 服务器");
            crate::acme_info!("2. 证书指纹: {}", tls_alpn01.certificate_thumbprint);
            crate::acme_info!("3. 确保 TLS 服务器正确响应");
            }
        }
        Ok(())
    }
}

/// 挑战信息枚举
#[derive(Debug, Clone)]
pub enum ChallengeInfo {
    Http01(Http01Challenge),
    Dns01(Dns01Challenge),
    TlsAlpn01(TlsAlpn01Challenge),
}

/// 便捷函数：处理单个授权的挑战
pub async fn process_single_authorization(
    client: &mut AcmeClient,
    authorization: &Authorization,
    challenge_type: Option<ChallengeType>,
) -> AcmeResult<Challenge> {
    let mut challenge_manager = ChallengeManager::new(client);
    challenge_manager.process_authorization(authorization, challenge_type).await
}

/// 便捷函数：等待挑战完成
pub async fn wait_for_challenge(
    client: &mut AcmeClient,
    challenge_url: &str,
    max_attempts: u32,
    delay: Duration,
) -> AcmeResult<Challenge> {
    let mut challenge_manager = ChallengeManager::new(client);
    challenge_manager.wait_for_challenge_completion(challenge_url, max_attempts, delay).await
}

/// 挑战恢复管理器
/// 用于从保存的授权信息中恢复中断的挑战流程
#[derive(Debug)]
pub struct ChallengeRecoveryManager<'a> {
    /// ACME 客户端引用
    client: &'a mut AcmeClient,
    /// JWS 构建器
    jws_builder: JwsBuilder,
}

impl<'a> ChallengeRecoveryManager<'a> {
    /// 创建新的挑战恢复管理器
    pub fn new(client: &'a mut AcmeClient) -> Self {
        let jws_builder = JwsBuilder::new(client.account_key().clone());
        Self {
            client,
            jws_builder,
        }
    }
    
    /// 从保存的授权信息恢复挑战
    /// 
    /// # 参数
    /// * `authorization` - 保存的授权信息
    /// * `preferred_challenge_type` - 首选挑战类型
    /// * `auto_setup` - 是否自动设置挑战（如果支持）
    /// 
    /// # 返回
    /// 返回恢复的挑战信息和是否需要手动设置
    pub async fn recover_challenge(
        &mut self,
        authorization: &Authorization,
        preferred_challenge_type: Option<ChallengeType>,
        auto_setup: bool,
    ) -> AcmeResult<(Challenge, bool)> {
        crate::acme_info!("🔄 开始恢复挑战流程...");
        crate::acme_info!("📋 域名: {}", authorization.identifier.value);
        crate::acme_info!("📅 授权过期时间: {:?}", authorization.expires);
        
        // 检查授权状态
        match authorization.status {
            AuthorizationStatus::Valid => {
                crate::acme_info!("✅ 授权已经有效，无需恢复挑战");
                // 返回第一个可用的挑战作为占位符
                if let Some(challenge) = authorization.challenges.first() {
                    return Ok((challenge.clone(), false));
                } else {
                    return Err(AcmeError::ProtocolError("授权中没有可用的挑战".to_string()));
                }
            }
            AuthorizationStatus::Invalid => {
                return Err(AcmeError::ChallengeValidationFailed(
                    "授权已失效，无法恢复".to_string()
                ));
            }
            AuthorizationStatus::Expired => {
                return Err(AcmeError::ChallengeValidationFailed(
                    "授权已过期，无法恢复".to_string()
                ));
            }
            _ => {
                crate::acme_info!("📝 授权状态: {:?}，继续恢复流程", authorization.status);
            }
        }
        
        // 选择合适的挑战
        let challenge = self.select_best_challenge(authorization, preferred_challenge_type)?;
        crate::acme_info!("🎯 选择挑战类型: {:?}", challenge.challenge_type);
        crate::acme_info!("🔗 挑战 URL: {}", challenge.url);
        
        // 检查挑战当前状态
        let current_challenge = self.get_challenge_status(&challenge.url).await?;
        crate::acme_info!("📊 当前挑战状态: {:?}", current_challenge.status);
        
        match current_challenge.status {
            ChallengeStatus::Valid => {
                crate::acme_info!("✅ 挑战已经完成，无需恢复");
                return Ok((current_challenge, false));
            }
            ChallengeStatus::Invalid => {
                crate::acme_info!("❌ 挑战已失效，尝试重新设置");
                if let Some(error) = &current_challenge.error {
                    crate::acme_info!("❌ 失效原因: {:?}", error);
                }
            }
            ChallengeStatus::Pending => {
                crate::acme_info!("⏳ 挑战待处理，准备设置");
            }
            ChallengeStatus::Processing => {
                crate::acme_info!("🔄 挑战处理中，等待完成");
                // 直接等待处理完成
                return self.wait_for_existing_challenge(&challenge.url).await;
            }
        }
        
        // 准备挑战信息
        let challenge_info = self.prepare_challenge_info(&current_challenge)?;
        
        // 显示挑战设置信息
        self.display_recovery_instructions(&challenge_info)?;
        
        // 如果支持自动设置，尝试自动设置
        let needs_manual_setup = if auto_setup {
            !self.try_auto_setup(&challenge_info).await?
        } else {
            true
        };
        
        if needs_manual_setup {
            crate::acme_info!("⚠️  需要手动设置挑战，请按照上述说明完成设置");
            crate::acme_info!("💡 设置完成后，程序将自动继续验证流程");
        }
        
        Ok((current_challenge, needs_manual_setup))
    }
    
    /// 完成恢复的挑战验证
    pub async fn complete_recovered_challenge(
        &mut self,
        challenge: &Challenge,
        wait_for_setup: bool,
    ) -> AcmeResult<Challenge> {
        if wait_for_setup {
            crate::acme_info!("⏳ 等待挑战设置完成...");
            crate::acme_info!("💡 请确认挑战已正确设置，然后按 Enter 键继续...");
            
            // 在实际应用中，这里可能需要更智能的等待机制
            // 比如定期检查 DNS 记录或 HTTP 端点
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).ok();
        }
        
        // 响应挑战
        crate::acme_info!("📤 发送挑战响应...");
        let responded_challenge = self.respond_to_challenge(challenge).await?;
        
        // 等待挑战完成
        crate::acme_info!("⏳ 等待挑战验证完成...");
        let completed_challenge = self.wait_for_challenge_completion(
            &challenge.url,
            30, // 最多等待 30 次
            Duration::from_secs(10), // 每次等待 10 秒
        ).await?;
        
        match completed_challenge.status {
            ChallengeStatus::Valid => {
                crate::acme_info!("✅ 挑战恢复并验证成功！");
            }
            ChallengeStatus::Invalid => {
                if let Some(error) = &completed_challenge.error {
                    crate::acme_info!("❌ 挑战验证失败: {:?}", error);
                }
                return Err(AcmeError::ChallengeValidationFailed(
                    "挑战恢复后验证失败".to_string()
                ));
            }
            _ => {
                return Err(AcmeError::ProtocolError(
                    format!("意外的挑战状态: {:?}", completed_challenge.status)
                ));
            }
        }
        
        Ok(completed_challenge)
    }
    
    /// 选择最佳挑战类型
    fn select_best_challenge(
        &self,
        authorization: &Authorization,
        preferred_type: Option<ChallengeType>,
    ) -> AcmeResult<Challenge> {
        // 如果指定了首选类型，优先选择
        if let Some(preferred) = preferred_type {
            for challenge in &authorization.challenges {
                if challenge.challenge_type == preferred {
                    return Ok(challenge.clone());
                }
            }
            crate::acme_info!("⚠️  首选挑战类型 {:?} 不可用，自动选择其他类型", preferred);
        }
        
        // 按优先级选择：DNS-01 > HTTP-01 > TLS-ALPN-01
        let priority_order = [ChallengeType::Dns01, ChallengeType::Http01, ChallengeType::TlsAlpn01];
        
        for challenge_type in &priority_order {
            for challenge in &authorization.challenges {
                if challenge.challenge_type == *challenge_type {
                    return Ok(challenge.clone());
                }
            }
        }
        
        Err(AcmeError::ProtocolError(
            "没有找到支持的挑战类型".to_string()
        ))
    }
    
    /// 准备挑战信息
    fn prepare_challenge_info(&self, challenge: &Challenge) -> AcmeResult<ChallengeInfo> {
        let key_authorization = self.create_key_authorization(&challenge.token)?;
        
        match challenge.challenge_type {
            ChallengeType::Http01 => {
                let http01 = Http01Challenge {
                    token: challenge.token.clone(),
                    key_authorization: key_authorization.clone(),
                    url: challenge.url.clone(),
                    file_path: format!("/.well-known/acme-challenge/{}", challenge.token),
                    file_content: key_authorization,
                };
                Ok(ChallengeInfo::Http01(http01))
            }
            ChallengeType::Dns01 => {
                let dns_value = self.create_dns_challenge_value(&key_authorization)?;
                let dns01 = Dns01Challenge {
                    token: challenge.token.clone(),
                    key_authorization,
                    url: challenge.url.clone(),
                    record_name: "_acme-challenge".to_string(),
                    record_value: dns_value,
                    record_type: "TXT".to_string(),
                };
                Ok(ChallengeInfo::Dns01(dns01))
            }
            ChallengeType::TlsAlpn01 => {
                let thumbprint = self.create_certificate_thumbprint(&key_authorization)?;
                let tls_alpn01 = TlsAlpn01Challenge {
                    token: challenge.token.clone(),
                    key_authorization,
                    url: challenge.url.clone(),
                    certificate_thumbprint: thumbprint,
                };
                Ok(ChallengeInfo::TlsAlpn01(tls_alpn01))
            }
        }
    }
    
    /// 显示恢复说明
    fn display_recovery_instructions(&self, challenge_info: &ChallengeInfo) -> AcmeResult<()> {
        crate::acme_info!("🔧 挑战恢复设置说明:");
        
        match challenge_info {
            ChallengeInfo::Http01(http01) => {
                crate::acme_info!("📁 HTTP-01 挑战恢复:");
                crate::acme_info!("   1. 在 Web 服务器创建文件: {}", http01.file_path);
                crate::acme_info!("   2. 文件内容: {}", http01.file_content);
                crate::acme_info!("   3. 确保文件可通过 HTTP 访问");
                crate::acme_info!("   4. 测试 URL: http://<domain>{}", http01.file_path);
            }
            ChallengeInfo::Dns01(dns01) => {
                crate::acme_info!("🌐 DNS-01 挑战恢复:");
                crate::acme_info!("   1. 创建 TXT 记录: {}", dns01.record_name);
                crate::acme_info!("   2. 记录值: {}", dns01.record_value);
                crate::acme_info!("   3. 等待 DNS 传播（通常需要 1-5 分钟）");
                crate::acme_info!("   4. 可使用 'dig TXT _acme-challenge.<domain>' 验证");
            }
            ChallengeInfo::TlsAlpn01(tls_alpn01) => {
                crate::acme_info!("🔒 TLS-ALPN-01 挑战恢复:");
                crate::acme_info!("   1. 配置 TLS 服务器支持 ALPN 扩展");
                crate::acme_info!("   2. 证书指纹: {}", tls_alpn01.certificate_thumbprint);
                crate::acme_info!("   3. 确保端口 443 可访问");
            }
        }
        
        Ok(())
    }
    
    /// 尝试自动设置挑战（如果支持）
    async fn try_auto_setup(&mut self, challenge_info: &ChallengeInfo) -> AcmeResult<bool> {
        match challenge_info {
            ChallengeInfo::Dns01(_dns01) => {
                // 这里可以集成 DNS 提供商 API 进行自动设置
                // 目前返回 false 表示需要手动设置
                crate::acme_info!("💡 DNS-01 自动设置功能待实现，需要手动设置");
                Ok(false)
            }
            ChallengeInfo::Http01(_http01) => {
                // 这里可以尝试在本地 Web 服务器创建文件
                // 目前返回 false 表示需要手动设置
                crate::acme_info!("💡 HTTP-01 自动设置功能待实现，需要手动设置");
                Ok(false)
            }
            ChallengeInfo::TlsAlpn01(_tls_alpn01) => {
                // TLS-ALPN-01 通常需要特殊的服务器配置
                crate::acme_info!("💡 TLS-ALPN-01 需要手动配置 TLS 服务器");
                Ok(false)
            }
        }
    }
    
    /// 等待现有挑战完成
    async fn wait_for_existing_challenge(&mut self, challenge_url: &str) -> AcmeResult<(Challenge, bool)> {
        crate::acme_info!("⏳ 检测到挑战正在处理中，等待完成...");
        
        let completed_challenge = self.wait_for_challenge_completion(
            challenge_url,
            30, // 最多等待 30 次
            Duration::from_secs(10), // 每次等待 10 秒
        ).await?;
        
        Ok((completed_challenge, false))
    }
    
    // 复用现有的方法
    fn create_key_authorization(&self, token: &str) -> AcmeResult<String> {
        let account_jwk = self.client.account_key().to_jwk()
            .map_err(|e| AcmeError::CryptoError(format!("Failed to create JWK: {}", e)))?;
        
        let jwk_thumbprint = self.create_jwk_thumbprint(&account_jwk)?;
        
        Ok(format!("{}.{}", token, jwk_thumbprint))
    }
    
    fn create_jwk_thumbprint(&self, jwk: &Value) -> AcmeResult<String> {
        let mut thumbprint_data = serde_json::Map::new();
        
        if let Some(obj) = jwk.as_object() {
            if let Some(crv) = obj.get("crv") {
                thumbprint_data.insert("crv".to_string(), crv.clone());
            }
            if let Some(kty) = obj.get("kty") {
                thumbprint_data.insert("kty".to_string(), kty.clone());
            }
            if let Some(x) = obj.get("x") {
                thumbprint_data.insert("x".to_string(), x.clone());
            }
            if let Some(y) = obj.get("y") {
                thumbprint_data.insert("y".to_string(), y.clone());
            }
        }
        
        let thumbprint_json = serde_json::to_string(&thumbprint_data)
            .map_err(|e| AcmeError::JsonError(format!("Failed to serialize JWK thumbprint: {}", e)))?;
        
        let mut hasher = Sha256::new();
        hasher.update(thumbprint_json.as_bytes());
        let hash = hasher.finalize();
        
        Ok(URL_SAFE_NO_PAD.encode(&hash))
    }
    
    fn create_dns_challenge_value(&self, key_authorization: &str) -> AcmeResult<String> {
        let mut hasher = Sha256::new();
        hasher.update(key_authorization.as_bytes());
        let hash = hasher.finalize();
        
        Ok(URL_SAFE_NO_PAD.encode(&hash))
    }
    
    fn create_certificate_thumbprint(&self, key_authorization: &str) -> AcmeResult<String> {
        let mut hasher = Sha256::new();
        hasher.update(key_authorization.as_bytes());
        let hash = hasher.finalize();
        
        Ok(hex::encode(&hash))
    }
    
    async fn respond_to_challenge(&mut self, challenge: &Challenge) -> AcmeResult<Challenge> {
        let nonce = self.client.get_nonce().await?;
        let account_url = self.client.account_url()
            .ok_or_else(|| AcmeError::ProtocolError("No account URL available".to_string()))?;
        
        let response_payload = json!({});
        
        let jws = self.jws_builder.create_for_existing_account(
            &nonce,
            &challenge.url,
            account_url,
            &response_payload,
        )?;
        
        let response = self.client.client()
            .post(&challenge.url)
            .header("Content-Type", "application/jose+json")
            .json(&jws)
            .send()
            .await
            .map_err(|e| AcmeError::HttpError(format!("Challenge response failed: {}", e)))?;
        
        self.client.set_nonce_from_response(&response);
        
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AcmeError::HttpError(
                format!("Challenge response failed with status {}: {}", status, error_text)
            ));
        }
        
        let updated_challenge: Challenge = response.json().await
            .map_err(|e| AcmeError::JsonError(format!("Failed to parse challenge response: {}", e)))?;
        
        Ok(updated_challenge)
    }
    
    async fn wait_for_challenge_completion(
        &mut self,
        challenge_url: &str,
        max_attempts: u32,
        delay: Duration,
    ) -> AcmeResult<Challenge> {
        for attempt in 1..=max_attempts {
            let challenge = self.get_challenge_status(challenge_url).await?;
            
            match challenge.status {
                ChallengeStatus::Valid => {
                    return Ok(challenge);
                }
                ChallengeStatus::Invalid => {
                    let error_msg = challenge.error
                        .map(|e| format!("Challenge failed: {:?}", e))
                        .unwrap_or_else(|| "Challenge failed with unknown error".to_string());
                    return Err(AcmeError::ChallengeValidationFailed(error_msg));
                }
                ChallengeStatus::Pending | ChallengeStatus::Processing => {
                    if attempt < max_attempts {
                        crate::acme_info!(
                            "Challenge status: {:?}, waiting {} seconds before retry (attempt {}/{})",
                            challenge.status, delay.as_secs(), attempt, max_attempts
                        );
                        tokio::time::sleep(delay).await;
                    } else {
                        return Err(AcmeError::Timeout(
                            format!("Challenge not completed after {} attempts", max_attempts)
                        ));
                    }
                }
            }
        }
        
        Err(AcmeError::Timeout(
            format!("Challenge not completed after {} attempts", max_attempts)
        ))
    }
    
    async fn get_challenge_status(&mut self, challenge_url: &str) -> AcmeResult<Challenge> {
        let nonce = self.client.get_nonce().await?;
        let account_url = self.client.account_url()
            .ok_or_else(|| AcmeError::ProtocolError("No account URL available".to_string()))?;
        
        let jws = self.jws_builder.create_post_as_get(
            &nonce,
            challenge_url,
            account_url,
        )?;
        
        let response = self.client.client()
            .post(challenge_url)
            .header("Content-Type", "application/jose+json")
            .json(&jws)
            .send()
            .await
            .map_err(|e| AcmeError::HttpError(format!("Challenge status request failed: {}", e)))?;
        
        self.client.set_nonce_from_response(&response);
        
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AcmeError::HttpError(
                format!("Challenge status request failed with status {}: {}", status, error_text)
            ));
        }
        
        let challenge: Challenge = response.json().await
            .map_err(|e| AcmeError::JsonError(format!("Failed to parse challenge status: {}", e)))?;
        
        Ok(challenge)
    }
}

/// 便捷函数：恢复挑战
pub async fn recover_challenge_from_authorization(
    client: &mut AcmeClient,
    authorization: &Authorization,
    preferred_challenge_type: Option<ChallengeType>,
    auto_setup: bool,
) -> AcmeResult<Challenge> {
    let mut recovery_manager = ChallengeRecoveryManager::new(client);
    
    let (challenge, needs_manual_setup) = recovery_manager
        .recover_challenge(authorization, preferred_challenge_type, auto_setup)
        .await?;
    
    let completed_challenge = recovery_manager
        .complete_recovered_challenge(&challenge, needs_manual_setup)
        .await?;
    
    Ok(completed_challenge)
}
