//! Cloudflare DNS 管理器实现
//! 支持通过 Cloudflare API 管理 DNS 记录

use crate::dns::{DnsManager, DnsOperationResult, DnsRecord, DnsRecordType};
use crate::error::{AcmeError, AcmeResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Cloudflare DNS 管理器
#[derive(Debug, Clone)]
pub struct CloudflareDnsManager {
    /// API Token
    api_token: String,
    /// HTTP 客户端
    client: Client,
    /// API 基础 URL
    base_url: String,
    /// 请求超时时间（秒）
    timeout_seconds: u64,
}

/// Cloudflare API 响应
#[derive(Debug, Deserialize)]
struct CloudflareResponse<T> {
    success: bool,
    errors: Vec<CloudflareError>,
    messages: Vec<String>,
    result: Option<T>,
    result_info: Option<CloudflareResultInfo>,
}

/// Cloudflare API 错误
#[derive(Debug, Deserialize)]
struct CloudflareError {
    code: u32,
    message: String,
}

/// Cloudflare 结果信息
#[derive(Debug, Deserialize)]
struct CloudflareResultInfo {
    page: u32,
    per_page: u32,
    count: u32,
    total_count: u32,
}

/// Cloudflare Zone 信息
#[derive(Debug, Deserialize)]
struct CloudflareZone {
    id: String,
    name: String,
    status: String,
}

/// Cloudflare DNS 记录
#[derive(Debug, Serialize, Deserialize)]
struct CloudflareDnsRecord {
    id: Option<String>,
    #[serde(rename = "type")]
    record_type: String,
    name: String,
    content: String,
    ttl: u32,
    priority: Option<u16>,
    proxied: Option<bool>,
    zone_id: Option<String>,
    zone_name: Option<String>,
    created_on: Option<String>,
    modified_on: Option<String>,
}

/// 创建 DNS 记录请求
#[derive(Debug, Serialize)]
struct CreateDnsRecordRequest {
    #[serde(rename = "type")]
    record_type: String,
    name: String,
    content: String,
    ttl: u32,
    priority: Option<u16>,
    proxied: Option<bool>,
}

/// 更新 DNS 记录请求
#[derive(Debug, Serialize)]
struct UpdateDnsRecordRequest {
    #[serde(rename = "type")]
    record_type: String,
    name: String,
    content: String,
    ttl: u32,
    priority: Option<u16>,
    proxied: Option<bool>,
}

impl CloudflareDnsManager {
    /// 创建新的 Cloudflare DNS 管理器
    pub fn new(api_token: String) -> AcmeResult<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| AcmeError::HttpError(format!("Failed to create HTTP client: {}", e)))?;
        
        Ok(Self {
            api_token,
            client,
            base_url: "https://api.cloudflare.com/client/v4".to_string(),
            timeout_seconds: 30,
        })
    }
    
    /// 设置请求超时时间
    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds;
        self
    }
    
    /// 设置自定义 API 基础 URL
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }
    
    /// 获取域名的 Zone ID
    async fn get_zone_id(&self, domain: &str) -> AcmeResult<String> {
        let url = format!("{}/zones?name={}", self.base_url, domain);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| AcmeError::HttpError(format!("Failed to get zone: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(AcmeError::HttpError(format!(
                "HTTP error {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }
        
        let cf_response: CloudflareResponse<Vec<CloudflareZone>> = response
            .json()
            .await
            .map_err(|e| AcmeError::HttpError(format!("Failed to parse response: {}", e)))?;
        
        if !cf_response.success {
            let error_msg = cf_response.errors
                .into_iter()
                .map(|e| format!("{}: {}", e.code, e.message))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(AcmeError::DnsError(format!("Cloudflare API error: {}", error_msg)));
        }
        
        let zones = cf_response.result.unwrap_or_default();
        if zones.is_empty() {
            return Err(AcmeError::DnsError(format!("Zone not found for domain: {}", domain)));
        }
        
        Ok(zones[0].id.clone())
    }
    
    /// 查找根域名
    async fn find_root_domain(&self, domain: &str) -> AcmeResult<String> {
        // 尝试不同的域名层级，从最具体到最通用
        let parts: Vec<&str> = domain.split('.').collect();
        
        for i in 0..parts.len() {
            let test_domain = parts[i..].join(".");
            if let Ok(_) = self.get_zone_id(&test_domain).await {
                return Ok(test_domain);
            }
        }
        
        Err(AcmeError::DnsError(format!("No managed zone found for domain: {}", domain)))
    }
    
    /// 发送 API 请求
    async fn send_request<T: for<'de> Deserialize<'de>>(
        &self,
        method: reqwest::Method,
        url: &str,
        body: Option<&impl Serialize>,
    ) -> AcmeResult<CloudflareResponse<T>> {
        let mut request = self.client
            .request(method, url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json");
        
        if let Some(body) = body {
            request = request.json(body);
        }
        
        let response = request
            .send()
            .await
            .map_err(|e| AcmeError::HttpError(format!("Request failed: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(AcmeError::HttpError(format!(
                "HTTP error {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }
        
        let cf_response: CloudflareResponse<T> = response
            .json()
            .await
            .map_err(|e| AcmeError::HttpError(format!("Failed to parse response: {}", e)))?;
        
        Ok(cf_response)
    }
    
    /// 转换 Cloudflare DNS 记录为通用格式
    fn convert_dns_record(&self, cf_record: CloudflareDnsRecord) -> DnsRecord {
        let record_type = match cf_record.record_type.as_str() {
            "A" => DnsRecordType::A,
            "AAAA" => DnsRecordType::AAAA,
            "CNAME" => DnsRecordType::CNAME,
            "TXT" => DnsRecordType::TXT,
            "MX" => DnsRecordType::MX,
            "NS" => DnsRecordType::NS,
            _ => DnsRecordType::TXT, // 默认为 TXT
        };
        
        DnsRecord {
            name: cf_record.name,
            record_type,
            value: cf_record.content,
            ttl: cf_record.ttl,
            priority: cf_record.priority,
            id: cf_record.id,
        }
    }
}

#[async_trait::async_trait]
impl DnsManager for CloudflareDnsManager {
    async fn add_txt_record(
        &self,
        domain: &str,
        name: &str,
        value: &str,
        ttl: u32,
    ) -> AcmeResult<DnsOperationResult> {
        let start_time = Instant::now();
        
        // 查找根域名和 Zone ID
        let root_domain = self.find_root_domain(domain).await?;
        let zone_id = self.get_zone_id(&root_domain).await?;
        
        // 创建 DNS 记录请求
        let request = CreateDnsRecordRequest {
            record_type: "TXT".to_string(),
            name: name.to_string(),
            content: value.to_string(),
            ttl,
            priority: None,
            proxied: Some(false), // TXT 记录不能被代理
        };
        
        // 发送创建请求
        let url = format!("{}/zones/{}/dns_records", self.base_url, zone_id);
        let response: CloudflareResponse<CloudflareDnsRecord> = self
            .send_request(reqwest::Method::POST, &url, Some(&request))
            .await?;
        
        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        if response.success {
            let record_id = response.result
                .and_then(|r| r.id)
                .unwrap_or_default();
            
            Ok(DnsOperationResult {
                success: true,
                record_id: Some(record_id),
                error_message: None,
                duration_ms,
            })
        } else {
            let error_msg = response.errors
                .into_iter()
                .map(|e| format!("{}: {}", e.code, e.message))
                .collect::<Vec<_>>()
                .join(", ");
            
            Ok(DnsOperationResult {
                success: false,
                record_id: None,
                error_message: Some(error_msg),
                duration_ms,
            })
        }
    }
    
    async fn delete_txt_record(
        &self,
        domain: &str,
        record_id: &str,
    ) -> AcmeResult<DnsOperationResult> {
        let start_time = Instant::now();
        
        // 查找根域名和 Zone ID
        let root_domain = self.find_root_domain(domain).await?;
        let zone_id = self.get_zone_id(&root_domain).await?;
        
        // 发送删除请求
        let url = format!("{}/zones/{}/dns_records/{}", self.base_url, zone_id, record_id);
        let response: CloudflareResponse<serde_json::Value> = self
            .send_request(reqwest::Method::DELETE, &url, None::<&()>)
            .await?;
        
        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        if response.success {
            Ok(DnsOperationResult {
                success: true,
                record_id: Some(record_id.to_string()),
                error_message: None,
                duration_ms,
            })
        } else {
            let error_msg = response.errors
                .into_iter()
                .map(|e| format!("{}: {}", e.code, e.message))
                .collect::<Vec<_>>()
                .join(", ");
            
            Ok(DnsOperationResult {
                success: false,
                record_id: Some(record_id.to_string()),
                error_message: Some(error_msg),
                duration_ms,
            })
        }
    }
    
    async fn find_txt_record(
        &self,
        domain: &str,
        name: &str,
    ) -> AcmeResult<Option<DnsRecord>> {
        // 查找根域名和 Zone ID
        let root_domain = self.find_root_domain(domain).await?;
        let zone_id = self.get_zone_id(&root_domain).await?;
        
        // 查询 DNS 记录
        let url = format!(
            "{}/zones/{}/dns_records?type=TXT&name={}",
            self.base_url, zone_id, name
        );
        
        let response: CloudflareResponse<Vec<CloudflareDnsRecord>> = self
            .send_request(reqwest::Method::GET, &url, None::<&()>)
            .await?;
        
        if response.success {
            if let Some(records) = response.result {
                if let Some(cf_record) = records.into_iter().next() {
                    return Ok(Some(self.convert_dns_record(cf_record)));
                }
            }
        }
        
        Ok(None)
    }
    
    async fn list_txt_records(
        &self,
        domain: &str,
    ) -> AcmeResult<Vec<DnsRecord>> {
        // 查找根域名和 Zone ID
        let root_domain = self.find_root_domain(domain).await?;
        let zone_id = self.get_zone_id(&root_domain).await?;
        
        // 查询所有 TXT 记录
        let url = format!(
            "{}/zones/{}/dns_records?type=TXT",
            self.base_url, zone_id
        );
        
        let response: CloudflareResponse<Vec<CloudflareDnsRecord>> = self
            .send_request(reqwest::Method::GET, &url, None::<&()>)
            .await?;
        
        if response.success {
            if let Some(cf_records) = response.result {
                let records = cf_records
                    .into_iter()
                    .map(|cf_record| self.convert_dns_record(cf_record))
                    .collect();
                return Ok(records);
            }
        }
        
        Ok(Vec::new())
    }
    
    async fn validate_credentials(&self) -> AcmeResult<bool> {
        // 使用与auth模块相同的验证端点来验证凭证
        let url = format!("{}/user/tokens/verify", self.base_url);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| AcmeError::HttpError(format!("Failed to validate credentials: {}", e)))?;
        
        if !response.status().is_success() {
            return Ok(false);
        }
        
        // 解析响应以确保token验证成功
        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AcmeError::HttpError(format!("Failed to parse response: {}", e)))?;
        
        // 检查Cloudflare API响应格式
        if let Some(success) = body.get("success").and_then(|v| v.as_bool()) {
            Ok(success)
        } else {
            Ok(false)
        }
    }
    
    fn provider_name(&self) -> &str {
        "Cloudflare"
    }
}

/// 便捷函数：创建 Cloudflare DNS 管理器
pub fn create_cloudflare_dns_manager(api_token: String) -> AcmeResult<CloudflareDnsManager> {
    CloudflareDnsManager::new(api_token)
}
