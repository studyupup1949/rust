//! ACME 账户管理模块
//! 处理 ACME 账户的注册、查询和管理

use crate::acme::jws::JwsBuilder;
use crate::acme::{AcmeClient, Directory};
use crate::auth::EabCredentials;
use crate::error::{AcmeError, AcmeResult};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// ACME 账户状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AccountStatus {
    /// 有效状态
    Valid,
    /// 已停用
    Deactivated,
    /// 已撤销
    Revoked,
}

/// ACME 账户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// 账户状态
    pub status: AccountStatus,
    /// 联系方式
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<Vec<String>>,
    /// 是否同意服务条款
    #[serde(rename = "termsOfServiceAgreed", skip_serializing_if = "Option::is_none")]
    pub terms_of_service_agreed: Option<bool>,
    /// 账户订单列表 URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orders: Option<String>,
    /// 创建时间
    #[serde(rename = "createdAt", skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    /// 初始 IP 地址
    #[serde(rename = "initialIp", skip_serializing_if = "Option::is_none")]
    pub initial_ip: Option<String>,
}

/// 账户注册请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountRegistrationRequest {
    /// 是否同意服务条款
    #[serde(rename = "termsOfServiceAgreed")]
    pub terms_of_service_agreed: bool,
    /// 联系方式
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<Vec<String>>,
    /// 外部账户绑定（用于需要 EAB 的 CA）
    #[serde(rename = "externalAccountBinding", skip_serializing_if = "Option::is_none")]
    pub external_account_binding: Option<Value>,
    /// 仅查询现有账户（不创建新账户）
    #[serde(rename = "onlyReturnExisting", skip_serializing_if = "Option::is_none")]
    pub only_return_existing: Option<bool>,
}

/// 账户更新请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountUpdateRequest {
    /// 联系方式
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<Vec<String>>,
    /// 账户状态（用于停用账户）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<AccountStatus>,
}

/// 密钥更换请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyChangeRequest {
    /// 旧密钥签名的账户信息
    pub account: String,
    /// 新密钥签名的密钥更换信息
    #[serde(rename = "oldKey")]
    pub old_key: Value,
}

/// 账户管理器
#[derive(Debug)]
pub struct AccountManager<'a> {
    /// ACME 客户端引用
    client: &'a mut AcmeClient,
    /// JWS 构建器
    jws_builder: JwsBuilder,
}

impl<'a> AccountManager<'a> {
    /// 创建新的账户管理器
    pub fn new(client: &'a mut AcmeClient) -> Self {
        let jws_builder = JwsBuilder::new(client.account_key().clone());
        Self {
            client,
            jws_builder,
        }
    }
    
    /// 注册新账户
    pub async fn register_account(
        &mut self,
        contact_email: Option<&str>,
        terms_agreed: bool,
        eab_credentials: Option<&EabCredentials>,
    ) -> AcmeResult<(Account, String)> {
        let directory = self.client.get_directory().await?;
        let new_account_url = directory.new_account.clone();
        
        // 检查是否需要外部账户绑定
        let requires_eab = directory.meta
            .as_ref()
            .and_then(|m| m.external_account_required)
            .unwrap_or(false);
        
        if requires_eab && eab_credentials.is_none() {
            return Err(AcmeError::ProtocolError(
                "需要外部账户绑定但未提供".to_string()
            ));
        }
        
        // 构建注册请求
        let mut registration_request = AccountRegistrationRequest {
            terms_of_service_agreed: terms_agreed,
            contact: contact_email.map(|email| vec![format!("mailto:{}", email)]),
            external_account_binding: None,
            only_return_existing: None,
        };
        
        // 如果需要 EAB，创建外部账户绑定
        if let Some(eab) = eab_credentials {
            let account_jwk = self.client.account_key().to_jwk()
                .map_err(|e| AcmeError::CryptoError(format!("创建账户 JWK 失败: {}", e)))?;
            
            let eab_jws = crate::acme::jws::create_eab_jws(
                &eab.kid,
                &eab.hmac_key,
                &account_jwk,
                &new_account_url,
            )?;
            
            registration_request.external_account_binding = Some(eab_jws);
        }
        
        // 发送注册请求
        let (account, account_url) = self.send_account_request(
            &new_account_url,
            &json!(registration_request),
            true, // 使用 JWK
        ).await?;
        
        // 保存账户 URL
        self.client.set_account_url(account_url.clone());
        
        if self.client.is_dry_run() {
            crate::acme_info!("[演练模式] 账户将使用 URL 注册: {}", account_url);
        } else {
            crate::acme_info!("账户使用 URL 注册成功: {}", account_url);
        }
        
        Ok((account, account_url))
    }
    
    /// 查找现有账户
    pub async fn find_existing_account(
        &mut self,
        contact_email: Option<&str>,
    ) -> AcmeResult<Option<(Account, String)>> {
        let directory = self.client.get_directory().await?;
        let new_account_url = directory.new_account.clone();
        
        let registration_request = AccountRegistrationRequest {
            terms_of_service_agreed: false, // 查找时不需要同意条款
            contact: contact_email.map(|email| vec![format!("mailto:{}", email)]),
            external_account_binding: None,
            only_return_existing: Some(true),
        };
        
        let result = self.send_account_request(
            &new_account_url,
            &json!(registration_request),
            true, // 使用 JWK
        ).await;
        
        match result {
            Ok((account, account_url)) => {
                self.client.set_account_url(account_url.clone());
                Ok(Some((account, account_url)))
            }
            Err(AcmeError::AccountNotFound(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }
    
    /// 获取账户信息
    pub async fn get_account_info(&mut self) -> AcmeResult<Account> {
        let account_url = self.client.account_url()
            .ok_or_else(|| AcmeError::ProtocolError("没有可用的账户 URL".to_string()))?
            .to_string();
        
        let (account, _) = self.send_account_request(
            &account_url,
            &json!(""), // POST-as-GET 使用空载荷
            false, // 使用 kid
        ).await?;
        
        Ok(account)
    }
    
    /// 更新账户信息
    pub async fn update_account(
        &mut self,
        update_request: AccountUpdateRequest,
    ) -> AcmeResult<Account> {
        let account_url = self.client.account_url()
            .ok_or_else(|| AcmeError::ProtocolError("没有可用的账户 URL".to_string()))?
            .to_string();
        
        let is_dry_run = self.client.is_dry_run();
        let (account, _) = self.send_account_request(
            &account_url,
            &json!(update_request),
            false, // 使用 kid
        ).await?;
        
        if is_dry_run {
            crate::acme_info!("[演练模式] 账户将被更新");
        } else {
            crate::acme_info!("账户更新成功");
        }
        
        Ok(account)
    }
    
    /// 停用账户
    pub async fn deactivate_account(&mut self) -> AcmeResult<Account> {
        let account_url = self.client.account_url()
            .ok_or_else(|| AcmeError::ProtocolError("没有可用的账户 URL".to_string()))?
            .to_string();
        
        let deactivate_request = AccountUpdateRequest {
            contact: None,
            status: Some(AccountStatus::Deactivated),
        };
        
        let is_dry_run = self.client.is_dry_run();
        let (account, _) = self.send_account_request(
            &account_url,
            &json!(deactivate_request),
            false, // 使用 kid
        ).await?;
        
        if is_dry_run {
            crate::acme_info!("[演练模式] 账户将被停用");
        } else {
            crate::acme_info!("账户停用成功");
        }
        
        Ok(account)
    }
    
    /// 更换账户密钥
    pub async fn change_account_key(
        &mut self,
        new_key_pair: crate::crypto::KeyPair,
    ) -> AcmeResult<()> {
        // 提前获取所有需要的值
        let account_url = self.client.account_url()
            .ok_or_else(|| AcmeError::ProtocolError("没有可用的账户 URL".to_string()))?
            .to_string();
        let old_key_jwk = self.client.account_key().to_jwk()
            .map_err(|e| AcmeError::CryptoError(format!("创建旧密钥 JWK 失败: {}", e)))?;
        
        let directory = self.client.get_directory().await?;
        let key_change_url = directory.key_change
            .as_ref()
            .ok_or_else(|| AcmeError::ProtocolError("服务器不支持密钥更换".to_string()))?
            .clone();
        
        // 创建新密钥的 JWS 构建器
        let new_jws_builder = JwsBuilder::new(new_key_pair.clone());
        
        // 获取 nonce
        let nonce = self.client.get_nonce().await?;
        
        // 创建内部 JWS（新密钥签名）
        let inner_payload = json!({
            "account": &account_url,
            "oldKey": old_key_jwk
        });
        
        let inner_jws = new_jws_builder.create_for_new_account(
            &nonce,
            &key_change_url,
            &inner_payload,
        )?;
        
        // 创建外部 JWS（旧密钥签名）
        let outer_nonce = self.client.get_nonce().await?;
        let outer_jws = self.jws_builder.create_for_existing_account(
            &outer_nonce,
            &key_change_url,
            &account_url,
            &json!(inner_jws),
        )?;
        
        // 发送密钥更换请求
        let response = self.client.client()
            .post(&key_change_url)
            .header("Content-Type", "application/jose+json")
            .json(&outer_jws)
            .send()
            .await
            .map_err(|e| AcmeError::HttpError(format!("密钥更换请求失败: {}", e)))?;
        
        // 更新 nonce
        self.client.set_nonce_from_response(&response);
        
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "未知错误".to_string());
            return Err(AcmeError::HttpError(
                format!("密钥更换失败，状态码 {}: {}", status, error_text)
            ));
        }
        
        let is_dry_run = self.client.is_dry_run();
        if is_dry_run {
            crate::acme_info!("[演练模式] 账户密钥将被更改");
        } else {
            crate::acme_info!("账户密钥更改成功");
            // 在实际模式下，更新客户端的密钥
            // 注意：这里需要重新创建客户端或更新密钥引用
        }
        
        Ok(())
    }
    
    /// 获取账户订单列表
    pub async fn get_account_orders(&mut self) -> AcmeResult<Vec<String>> {
        let account = self.get_account_info().await?;
        
        let orders_url = account.orders
            .ok_or_else(|| AcmeError::ProtocolError("账户没有订单 URL".to_string()))?;
        
        // 获取 nonce
        let account_url = self.client.account_url().unwrap().to_string();
        let nonce = self.client.get_nonce().await?;
        
        // 创建 POST-as-GET 请求
        let jws = self.jws_builder.create_post_as_get(
            &nonce,
            &orders_url,
            &account_url,
        )?;
        
        // 发送请求
        let response = self.client.client()
            .post(&orders_url)
            .header("Content-Type", "application/jose+json")
            .json(&jws)
            .send()
            .await
            .map_err(|e| AcmeError::HttpError(format!("订单请求失败: {}", e)))?;
        
        // 更新 nonce
        self.client.set_nonce_from_response(&response);
        
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "未知错误".to_string());
            return Err(AcmeError::HttpError(
                format!("订单请求失败，状态码 {}: {}", status, error_text)
            ));
        }
        
        #[derive(Deserialize)]
        struct OrdersList {
            orders: Vec<String>,
        }
        
        let orders_list: OrdersList = response.json().await
            .map_err(|e| AcmeError::JsonError(format!("解析订单响应失败: {}", e)))?;
        
        Ok(orders_list.orders)
    }
    
    /// 发送账户相关请求的通用方法
    async fn send_account_request(
        &mut self,
        url: &str,
        payload: &Value,
        use_jwk: bool,
    ) -> AcmeResult<(Account, String)> {
        // 获取 nonce
        let nonce = self.client.get_nonce().await?;
        
        // 创建 JWS
        let jws = if use_jwk {
            self.jws_builder.create_for_new_account(&nonce, url, payload)?
        } else {
            let account_url = self.client.account_url()
                .ok_or_else(|| AcmeError::ProtocolError("没有可用的账户 URL".to_string()))?;
            self.jws_builder.create_for_existing_account(&nonce, url, account_url, payload)?
        };
        
        // 发送请求
        let response = self.client.client()
            .post(url)
            .header("Content-Type", "application/jose+json")
            .json(&jws)
            .send()
            .await
            .map_err(|e| AcmeError::HttpError(format!("账户请求失败: {}", e)))?;
        
        // 更新 nonce
        self.client.set_nonce_from_response(&response);
        
        // 处理响应
        let status = response.status();
        
        match status {
            StatusCode::OK | StatusCode::CREATED => {
                // 获取账户 URL
                let account_url = if status == StatusCode::CREATED {
                    // 新账户创建，从 Location 头部获取账户 URL
                    response.headers()
                        .get("location")
                        .ok_or_else(|| AcmeError::ProtocolError("缺少 Location 头部".to_string()))?
                        .to_str()
                        .map_err(|e| AcmeError::ProtocolError(format!("无效的 Location 头部: {}", e)))?
                        .to_string()
                } else {
                    // 现有账户，使用客户端中存储的账户 URL
                    if use_jwk {
                        // 如果使用 JWK，这可能是查找现有账户的请求，从 Location 头部获取
                        response.headers()
                            .get("location")
                            .ok_or_else(|| AcmeError::ProtocolError("缺少 Location 头部".to_string()))?
                            .to_str()
                            .map_err(|e| AcmeError::ProtocolError(format!("无效的 Location 头部: {}", e)))?
                            .to_string()
                    } else {
                        // 使用 KeyID，从客户端获取已存储的账户 URL
                        self.client.account_url()
                            .ok_or_else(|| AcmeError::ProtocolError("没有可用的账户 URL".to_string()))?
                            .to_string()
                    }
                };
                
                // 解析账户信息
                let account: Account = response.json().await
                    .map_err(|e| AcmeError::JsonError(format!("解析账户响应失败: {}", e)))?;
                
                Ok((account, account_url))
            }
            StatusCode::BAD_REQUEST => {
                // 可能是账户不存在
                let error_response: Result<crate::acme::AcmeErrorResponse, _> = response.json().await;
                if let Ok(error) = error_response {
                    if error.error_type.contains("accountDoesNotExist") {
                        return Err(AcmeError::AccountNotFound("账户不存在".to_string()));
                    }
                }
                Err(AcmeError::HttpError(format!("错误请求: {}", status)))
            }
            _ => {
                let error_text = response.text().await
                    .unwrap_or_else(|_| "未知错误".to_string());
                Err(AcmeError::HttpError(
                    format!("账户请求失败，状态码 {}: {}", status, error_text)
                ))
            }
        }
    }
}

/// 便捷函数：注册或查找账户
pub async fn register_or_find_account(
    client: &mut AcmeClient,
    contact_email: Option<&str>,
    terms_agreed: bool,
    eab_credentials: Option<&EabCredentials>,
) -> AcmeResult<(Account, String)> {
    let mut account_manager = AccountManager::new(client);
    
    // 首先尝试查找现有账户
    if let Some((account, account_url)) = account_manager.find_existing_account(contact_email).await? {
        if client.is_dry_run() {
            crate::acme_info!("[演练模式] 找到现有账户: {}", account_url);
        } else {
            crate::acme_info!("找到现有账户: {}", account_url);
        }
        return Ok((account, account_url));
    }
    
    // 如果没有找到现有账户，注册新账户
    account_manager.register_account(contact_email, terms_agreed, eab_credentials).await
}
