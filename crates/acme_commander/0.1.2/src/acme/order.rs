//! ACME 订单管理模块
//! 处理证书订单的创建、查询和完成

use crate::acme::jws::JwsBuilder;
use crate::acme::{AcmeClient, Identifier};
use crate::error::{AcmeError, AcmeResult};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Utc};

/// ACME 订单状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    /// 待处理
    Pending,
    /// 准备就绪
    Ready,
    /// 处理中
    Processing,
    /// 有效（已完成）
    Valid,
    /// 无效
    Invalid,
}

/// ACME 订单信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// 订单状态
    pub status: OrderStatus,
    /// 过期时间
    pub expires: DateTime<Utc>,
    /// 标识符列表
    pub identifiers: Vec<Identifier>,
    /// 授权 URL 列表
    pub authorizations: Vec<String>,
    /// 完成 URL（用于提交 CSR）
    pub finalize: String,
    /// 证书 URL（订单完成后可用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate: Option<String>,
    /// 错误信息（如果订单失败）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
}

/// 订单创建请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    /// 标识符列表（域名或 IP）
    pub identifiers: Vec<Identifier>,
    /// 证书有效期开始时间（可选）
    #[serde(rename = "notBefore", skip_serializing_if = "Option::is_none")]
    pub not_before: Option<String>,
    /// 证书有效期结束时间（可选）
    #[serde(rename = "notAfter", skip_serializing_if = "Option::is_none")]
    pub not_after: Option<String>,
}

/// 订单完成请求（CSR）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalizeRequest {
    /// Base64URL 编码的 CSR
    pub csr: String,
}

/// 订单管理器
#[derive(Debug)]
pub struct OrderManager<'a> {
    /// ACME 客户端引用
    client: &'a mut AcmeClient,
    /// JWS 构建器
    jws_builder: JwsBuilder,
}

impl<'a> OrderManager<'a> {
    /// 创建新的订单管理器
    pub fn new(client: &'a mut AcmeClient) -> Self {
        let jws_builder = JwsBuilder::new(client.account_key().clone());
        Self {
            client,
            jws_builder,
        }
    }
    
    /// 创建新订单
    pub async fn create_order(
        &mut self,
        domains: &[String],
        not_before: Option<SystemTime>,
        not_after: Option<SystemTime>,
    ) -> AcmeResult<(Order, String)> {
        // 验证域名并创建标识符
        let mut identifiers = Vec::new();
        for domain in domains {
            let identifier = crate::acme::create_identifier(domain)?;
            identifiers.push(identifier);
        }
        
        if identifiers.is_empty() {
            return Err(AcmeError::InvalidDomain("至少需要一个域名".to_string()));
        }
        
        // 构建订单请求
        let order_request = OrderRequest {
            identifiers,
            not_before: not_before.map(|t| format_time(t)),
            not_after: not_after.map(|t| format_time(t)),
        };
        
        // 获取目录信息
        let directory = self.client.get_directory().await?;
        
        // 发送订单创建请求
        let new_order_url = directory.new_order.clone();
        let (order, order_url) = self.send_order_request(
            &new_order_url,
            &json!(order_request),
        ).await?;
        
        if self.client.is_dry_run() {
            crate::acme_info!("[演练模式] 将为域名创建订单: {:?}", domains);
            crate::acme_info!("[演练模式] 订单 URL: {}", order_url);
        } else {
            crate::acme_info!("已为域名创建订单: {:?}", domains);
            crate::acme_info!("订单 URL: {}", order_url);
        }
        
        Ok((order, order_url))
    }
    
    /// 获取订单信息
    pub async fn get_order(&mut self, order_url: &str) -> AcmeResult<Order> {
        let (order, _) = self.send_order_request(order_url, &json!("")).await?;
        Ok(order)
    }
    
    /// 等待订单准备就绪
    pub async fn wait_for_order_ready(
        &mut self,
        order_url: &str,
        max_attempts: u32,
        delay: Duration,
    ) -> AcmeResult<Order> {
        for attempt in 1..=max_attempts {
            let order = self.get_order(order_url).await?;
            
            match order.status {
                OrderStatus::Ready => {
                    if self.client.is_dry_run() {
                        crate::acme_info!("[演练模式] 订单已准备好完成");
                    } else {
                        crate::acme_info!("订单已准备好完成");
                    }
                    return Ok(order);
                }
                OrderStatus::Valid => {
                    if self.client.is_dry_run() {
                        crate::acme_info!("[演练模式] 订单已经有效");
                    } else {
                        crate::acme_info!("订单已经有效");
                    }
                    return Ok(order);
                }
                OrderStatus::Invalid => {
                    return Err(AcmeError::OrderFailed("订单无效".to_string()));
                }
                OrderStatus::Pending | OrderStatus::Processing => {
                    if attempt < max_attempts {
                        crate::acme_info!(
                            "Order status: {:?}, waiting {} seconds before retry (attempt {}/{})",
                            order.status, delay.as_secs(), attempt, max_attempts
                        );
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
    
    /// 完成订单（提交 CSR）
    pub async fn finalize_order(
        &mut self,
        order: &Order,
        csr_der: &[u8],
    ) -> AcmeResult<Order> {
        // Base64URL 编码 CSR
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
        let csr_b64 = URL_SAFE_NO_PAD.encode(csr_der);
        
        let finalize_request = FinalizeRequest {
            csr: csr_b64,
        };
        
        // 发送完成请求
        let (updated_order, _) = self.send_order_request(
            &order.finalize,
            &json!(finalize_request),
        ).await?;
        
        if self.client.is_dry_run() {
            crate::acme_info!("[演练模式] 订单将使用 CSR 完成");
        } else {
            crate::acme_info!("订单已使用 CSR 完成");
        }
        
        Ok(updated_order)
    }
    
    /// 等待订单完成并获取证书
    pub async fn wait_for_certificate(
        &mut self,
        order_url: &str,
        max_attempts: u32,
        delay: Duration,
    ) -> AcmeResult<(Order, Option<String>)> {
        for attempt in 1..=max_attempts {
            let order = self.get_order(order_url).await?;
            
            match order.status {
                OrderStatus::Valid => {
                    if let Some(cert_url) = &order.certificate {
                        let certificate = self.download_certificate(cert_url).await?;
                        
                        if self.client.is_dry_run() {
                            crate::acme_info!("[演练模式] 证书将可用");
                        } else {
                            crate::acme_info!("证书已准备好下载");
                        }
                        
                        return Ok((order, Some(certificate)));
                    } else {
                        return Err(AcmeError::ProtocolError(
                            "订单有效但未提供证书 URL".to_string()
                        ));
                    }
                }
                OrderStatus::Invalid => {
                    return Err(AcmeError::OrderFailed("订单无效".to_string()));
                }
                OrderStatus::Processing => {
                    if attempt < max_attempts {
                        crate::acme_info!(
                            "Certificate processing, waiting {} seconds before retry (attempt {}/{})",
                            delay.as_secs(), attempt, max_attempts
                        );
                        tokio::time::sleep(delay).await;
                    } else {
                        return Err(AcmeError::Timeout(
                            format!("经过 {} 次尝试后证书仍未准备就绪", max_attempts)
                        ));
                    }
                }
                _ => {
                    return Err(AcmeError::ProtocolError(
                        format!("意外的订单状态: {:?}", order.status)
                    ));
                }
            }
        }
        
        Err(AcmeError::Timeout(
            format!("经过 {} 次尝试后证书仍未准备就绪", max_attempts)
        ))
    }
    
    /// 下载证书
    pub async fn download_certificate(&mut self, cert_url: &str) -> AcmeResult<String> {
        // 获取 nonce
        let nonce = self.client.get_nonce().await?;
        let account_url = self.client.account_url()
            .ok_or_else(|| AcmeError::ProtocolError("没有可用的账户 URL".to_string()))?;
        
        // 创建 POST-as-GET 请求
        let jws = self.jws_builder.create_post_as_get(
            &nonce,
            cert_url,
            account_url,
        )?;
        
        // 发送请求
        let response = self.client.client()
            .post(cert_url)
            .header("Content-Type", "application/jose+json")
            .json(&jws)
            .send()
            .await
            .map_err(|e| AcmeError::HttpError(format!("证书下载失败: {}", e)))?;
        
        // 更新 nonce
        self.client.set_nonce_from_response(&response);
        
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
        
        if self.client.is_dry_run() {
            crate::acme_info!("[演练模式] 证书将被下载 ({} 字节)", certificate.len());
        } else {
            crate::acme_info!("证书已下载 ({} 字节)", certificate.len());
        }
        
        Ok(certificate)
    }
    
    /// 获取订单的授权列表
    pub async fn get_order_authorizations(
        &mut self,
        order: &Order,
    ) -> AcmeResult<Vec<crate::acme::Authorization>> {
        let mut authorizations = Vec::new();
        
        for auth_url in &order.authorizations {
            let authorization = self.get_authorization(auth_url).await?;
            authorizations.push(authorization);
        }
        
        Ok(authorizations)
    }
    
    /// 获取单个授权信息
    async fn get_authorization(&mut self, auth_url: &str) -> AcmeResult<crate::acme::Authorization> {
        // 获取 nonce
        let nonce = self.client.get_nonce().await?;
        let account_url = self.client.account_url()
            .ok_or_else(|| AcmeError::ProtocolError("没有可用的账户 URL".to_string()))?;
        
        // 创建 POST-as-GET 请求
        let jws = self.jws_builder.create_post_as_get(
            &nonce,
            auth_url,
            account_url,
        )?;
        
        // 发送请求
        let response = self.client.client()
            .post(auth_url)
            .header("Content-Type", "application/jose+json")
            .json(&jws)
            .send()
            .await
            .map_err(|e| AcmeError::HttpError(format!("授权请求失败: {}", e)))?;
        
        // 更新 nonce
        self.client.set_nonce_from_response(&response);
        
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "未知错误".to_string());
            return Err(AcmeError::HttpError(
                format!("授权请求失败，状态码 {}: {}", status, error_text)
            ));
        }
        
        let authorization: crate::acme::Authorization = response.json().await
            .map_err(|e| AcmeError::JsonError(format!("解析授权失败: {}", e)))?;
        
        Ok(authorization)
    }
    
    /// 发送订单相关请求的通用方法
    async fn send_order_request(
        &mut self,
        url: &str,
        payload: &Value,
    ) -> AcmeResult<(Order, String)> {
        // 获取 nonce
        let nonce = self.client.get_nonce().await?;
        let account_url = self.client.account_url()
            .ok_or_else(|| AcmeError::ProtocolError("没有可用的账户 URL".to_string()))?;
        
        // 创建 JWS
        let jws = if payload.is_string() && payload.as_str() == Some("") {
            // POST-as-GET 请求
            self.jws_builder.create_post_as_get(&nonce, url, account_url)?
        } else {
            // 普通 POST 请求
            self.jws_builder.create_for_existing_account(&nonce, url, account_url, payload)?
        };
        
        // 发送请求
        let response = self.client.client()
            .post(url)
            .header("Content-Type", "application/jose+json")
            .json(&jws)
            .send()
            .await
            .map_err(|e| AcmeError::HttpError(format!("订单请求失败: {}", e)))?;
        
        // 更新 nonce
        self.client.set_nonce_from_response(&response);
        
        // 处理响应
        let status = response.status();
        
        match status {
            StatusCode::OK | StatusCode::CREATED => {
                // 获取订单 URL
                let order_url = if status == StatusCode::CREATED {
                    response.headers()
                        .get("location")
                        .ok_or_else(|| AcmeError::ProtocolError("缺少 Location 头部".to_string()))?
                        .to_str()
                        .map_err(|e| AcmeError::ProtocolError(format!("无效的 Location 头部: {}", e)))?
                        .to_string()
                } else {
                    url.to_string()
                };
                
                // 解析订单信息
                let order: Order = response.json().await
                    .map_err(|e| AcmeError::JsonError(format!("解析订单响应失败: {}", e)))?;
                
                Ok((order, order_url))
            }
            _ => {
                let error_text = response.text().await
                    .unwrap_or_else(|_| "未知错误".to_string());
                Err(AcmeError::HttpError(
                    format!("订单请求失败，状态码 {}: {}", status, error_text)
                ))
            }
        }
    }
}

/// 格式化时间为 RFC3339 格式
fn format_time(time: SystemTime) -> String {
    let duration = time.duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0));
    
    let secs = duration.as_secs();
    let nanos = duration.subsec_nanos();
    
    // 简单的 RFC3339 格式化（实际应用中建议使用 chrono）
    format!("{}.{:09}Z", 
        chrono::DateTime::<chrono::Utc>::from_timestamp(secs as i64, nanos)
            .unwrap_or_default()
            .format("%Y-%m-%dT%H:%M:%S"),
        nanos
    )
}

/// 便捷函数：创建简单的域名订单
pub async fn create_domain_order(
    client: &mut AcmeClient,
    domains: &[String],
) -> AcmeResult<(Order, String)> {
    let mut order_manager = OrderManager::new(client);
    order_manager.create_order(domains, None, None).await
}

/// 便捷函数：完整的订单处理流程
pub async fn process_order_complete(
    client: &mut AcmeClient,
    domains: &[String],
    csr_der: &[u8],
    max_wait_attempts: u32,
    wait_delay: Duration,
) -> AcmeResult<(Order, String)> {
    let mut order_manager = OrderManager::new(client);
    
    // 1. 创建订单
    let (order, order_url) = order_manager.create_order(domains, None, None).await?;
    
    // 2. 等待订单准备就绪（需要先完成挑战验证）
    let ready_order = order_manager.wait_for_order_ready(
        &order_url,
        max_wait_attempts,
        wait_delay,
    ).await?;
    
    // 3. 完成订单
    let finalized_order = order_manager.finalize_order(&ready_order, csr_der).await?;
    
    // 4. 等待证书生成
    let (final_order, certificate) = order_manager.wait_for_certificate(
        &order_url,
        max_wait_attempts,
        wait_delay,
    ).await?;
    
    let cert_content = certificate.unwrap_or_default();
    
    Ok((final_order, cert_content))
}
