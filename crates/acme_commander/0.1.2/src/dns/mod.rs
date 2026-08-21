//! DNS 挑战处理模块
//! 支持多种 DNS 提供商的 TXT 记录管理

use crate::error::{AcmeError, AcmeResult};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

pub mod cloudflare;

/// DNS 提供商类型
#[derive(Debug, Clone, PartialEq)]
pub enum DnsProvider {
    /// Cloudflare DNS
    Cloudflare,
}

/// DNS 记录类型
#[derive(Debug, Clone, PartialEq)]
pub enum DnsRecordType {
    /// A 记录
    A,
    /// AAAA 记录
    AAAA,
    /// CNAME 记录
    CNAME,
    /// TXT 记录
    TXT,
    /// MX 记录
    MX,
    /// NS 记录
    NS,
}

/// DNS 记录
#[derive(Debug, Clone)]
pub struct DnsRecord {
    /// 记录名称
    pub name: String,
    /// 记录类型
    pub record_type: DnsRecordType,
    /// 记录值
    pub value: String,
    /// TTL（生存时间）
    pub ttl: u32,
    /// 优先级（用于 MX 记录）
    pub priority: Option<u16>,
    /// 记录 ID（由 DNS 提供商分配）
    pub id: Option<String>,
}

/// DNS 挑战记录
#[derive(Debug, Clone)]
pub struct DnsChallengeRecord {
    /// 域名
    pub domain: String,
    /// 挑战记录名称（通常是 _acme-challenge.domain）
    pub record_name: String,
    /// 挑战值
    pub challenge_value: String,
    /// TTL
    pub ttl: u32,
    /// 记录 ID（用于删除）
    pub record_id: Option<String>,
}

/// DNS 操作结果
#[derive(Debug, Clone)]
pub struct DnsOperationResult {
    /// 操作是否成功
    pub success: bool,
    /// 记录 ID
    pub record_id: Option<String>,
    /// 错误信息
    pub error_message: Option<String>,
    /// 操作耗时（毫秒）
    pub duration_ms: u64,
}

/// DNS 传播检查结果
#[derive(Debug, Clone)]
pub struct DnsPropagationResult {
    /// 是否已传播
    pub propagated: bool,
    /// 检查的 DNS 服务器
    pub checked_servers: Vec<String>,
    /// 成功解析的服务器
    pub successful_servers: Vec<String>,
    /// 失败的服务器及错误信息
    pub failed_servers: HashMap<String, String>,
    /// 检查耗时（毫秒）
    pub duration_ms: u64,
}

/// DNS 管理器特征
#[async_trait::async_trait]
pub trait DnsManager: Send + Sync {
    /// 添加 TXT 记录
    async fn add_txt_record(
        &self,
        domain: &str,
        name: &str,
        value: &str,
        ttl: u32,
    ) -> AcmeResult<DnsOperationResult>;
    
    /// 删除 TXT 记录
    async fn delete_txt_record(
        &self,
        domain: &str,
        record_id: &str,
    ) -> AcmeResult<DnsOperationResult>;
    
    /// 查找 TXT 记录
    async fn find_txt_record(
        &self,
        domain: &str,
        name: &str,
    ) -> AcmeResult<Option<DnsRecord>>;
    
    /// 列出域名的所有 TXT 记录
    async fn list_txt_records(
        &self,
        domain: &str,
    ) -> AcmeResult<Vec<DnsRecord>>;
    
    /// 验证 DNS 提供商凭证
    async fn validate_credentials(&self) -> AcmeResult<bool>;
    
    /// 获取提供商名称
    fn provider_name(&self) -> &str;
}

/// DNS 挑战管理器
pub struct DnsChallengeManager {
    /// DNS 管理器
    dns_manager: Box<dyn DnsManager>,
    /// 默认 TTL
    default_ttl: u32,
    /// 传播检查超时时间（秒）
    propagation_timeout: u64,
    /// 传播检查间隔（秒）
    propagation_interval: u64,
    /// DNS 服务器列表（用于传播检查）
    dns_servers: Vec<String>,
}

impl DnsChallengeManager {
    /// 创建新的 DNS 挑战管理器
    pub fn new(
        dns_manager: Box<dyn DnsManager>,
        default_ttl: Option<u32>,
        propagation_timeout: Option<u64>,
    ) -> Self {
        Self {
            dns_manager,
            default_ttl: default_ttl.unwrap_or(60), // 默认 60 秒
            propagation_timeout: propagation_timeout.unwrap_or(300), // 默认 5 分钟
            propagation_interval: 10, // 每 10 秒检查一次
            dns_servers: vec![
                "8.8.8.8".to_string(),      // Google DNS
                "1.1.1.1".to_string(),      // Cloudflare DNS
                "208.67.222.222".to_string(), // OpenDNS
                "9.9.9.9".to_string(),      // Quad9 DNS
            ],
        }
    }
    
    /// 添加 DNS 挑战记录
    pub async fn add_challenge_record(
        &self,
        domain: &str,
        challenge_value: &str,
        dry_run: bool,
    ) -> AcmeResult<DnsChallengeRecord> {
        let record_name = format!("_acme-challenge.{}", domain);
        
        if dry_run {
            println!("[演练模式] 将添加 DNS TXT 记录:");
            println!("  名称: {}", record_name);
            println!("  值: {}", challenge_value);
            println!("  TTL: {}", self.default_ttl);
            
            return Ok(DnsChallengeRecord {
                domain: domain.to_string(),
                record_name,
                challenge_value: challenge_value.to_string(),
                ttl: self.default_ttl,
                record_id: Some("dry-run-record-id".to_string()),
            });
        }
        
        let result = self.dns_manager.add_txt_record(
            domain,
            &record_name,
            challenge_value,
            self.default_ttl,
        ).await?;
        
        if !result.success {
            return Err(AcmeError::DnsError(
                result.error_message.unwrap_or("添加DNS记录失败".to_string())
            ));
        }
        
        Ok(DnsChallengeRecord {
            domain: domain.to_string(),
            record_name,
            challenge_value: challenge_value.to_string(),
            ttl: self.default_ttl,
            record_id: result.record_id,
        })
    }
    
    /// 删除 DNS 挑战记录
    pub async fn delete_challenge_record(
        &self,
        challenge_record: &DnsChallengeRecord,
        dry_run: bool,
    ) -> AcmeResult<()> {
        if dry_run {
            println!("[演练模式] 将删除 DNS TXT 记录:");
            println!("  名称: {}", challenge_record.record_name);
            if let Some(record_id) = &challenge_record.record_id {
                println!("  记录 ID: {}", record_id);
            }
            return Ok(());
        }
        
        if let Some(record_id) = &challenge_record.record_id {
            let result = self.dns_manager.delete_txt_record(
                &challenge_record.domain,
                record_id,
            ).await?;
            
            if !result.success {
                return Err(AcmeError::DnsError(
                    result.error_message.unwrap_or("删除DNS记录失败".to_string())
                ));
            }
        }
        
        Ok(())
    }
    
    /// 等待 DNS 记录传播
    pub async fn wait_for_propagation(
        &self,
        challenge_record: &DnsChallengeRecord,
        dry_run: bool,
    ) -> AcmeResult<DnsPropagationResult> {
        if dry_run {
            println!("[演练模式] 将等待 DNS 记录传播: {}", challenge_record.record_name);
            return Ok(DnsPropagationResult {
                propagated: true,
                checked_servers: self.dns_servers.clone(),
                successful_servers: self.dns_servers.clone(),
                failed_servers: HashMap::new(),
                duration_ms: 0,
            });
        }
        
        let start_time = std::time::Instant::now();
        let timeout = Duration::from_secs(self.propagation_timeout);
        let interval = Duration::from_secs(self.propagation_interval);
        
        loop {
            let check_result = self.check_dns_propagation(challenge_record).await?;
            
            if check_result.propagated {
                return Ok(check_result);
            }
            
            if start_time.elapsed() >= timeout {
                return Err(AcmeError::DnsError(format!(
                    "DNS传播超时，等待{}秒后仍未完成",
                    self.propagation_timeout
                )));
            }
            
            sleep(interval).await;
        }
    }
    
    /// 检查 DNS 记录传播状态
    pub async fn check_dns_propagation(
        &self,
        challenge_record: &DnsChallengeRecord,
    ) -> AcmeResult<DnsPropagationResult> {
        let start_time = std::time::Instant::now();
        let mut successful_servers = Vec::new();
        let mut failed_servers = HashMap::new();
        
        for dns_server in &self.dns_servers {
            match self.query_txt_record(dns_server, &challenge_record.record_name).await {
                Ok(values) => {
                    if values.contains(&challenge_record.challenge_value) {
                        successful_servers.push(dns_server.clone());
                    } else {
                        failed_servers.insert(
                            dns_server.clone(),
                            "TXT记录中未找到挑战值".to_string(),
                        );
                    }
                }
                Err(e) => {
                    failed_servers.insert(dns_server.clone(), e.to_string());
                }
            }
        }
        
        let propagated = !successful_servers.is_empty() && 
                        successful_servers.len() >= (self.dns_servers.len() / 2); // 至少一半的服务器成功
        
        Ok(DnsPropagationResult {
            propagated,
            checked_servers: self.dns_servers.clone(),
            successful_servers,
            failed_servers,
            duration_ms: start_time.elapsed().as_millis() as u64,
        })
    }
    
    /// 查询 TXT 记录
    async fn query_txt_record(
        &self,
        dns_server: &str,
        record_name: &str,
    ) -> AcmeResult<Vec<String>> {
        use rat_quickdns::{DnsResolverBuilder, QueryStrategy, DnsQueryRequest};
        use rat_quickdns::builder::DnsRecordType;
        use std::time::Duration;
        
        // 创建DNS解析器（禁用日志初始化，避免与acme_commander日志系统冲突）
        let resolver = DnsResolverBuilder::new(
            QueryStrategy::Smart,
            true,  // 启用 EDNS
            "acme_dns_check".to_string(),
        )
        .with_cache(false) // DNS传播检查不需要缓存
        .with_timeout(Duration::from_secs(5))
        .with_retry_count(2)
        .add_udp_upstream("custom_server", dns_server)
        .disable_logger_init() // 禁用rat_quickdns的自动日志初始化
        .build()
        .await
        .map_err(|e| AcmeError::DnsError(format!("创建DNS解析器失败: {}", e)))?;
        
        // 查询 TXT 记录
        let request = DnsQueryRequest::new(record_name, DnsRecordType::TXT)
            .with_timeout(5000); // 5秒超时
        
        let response = resolver.query(request).await
            .map_err(|e| AcmeError::DnsError(format!("DNS查询失败: {}", e)))?;
        
        if !response.success {
            return Err(AcmeError::DnsError("DNS查询未返回有效结果".to_string()));
        }
        
        // 提取TXT记录值
        let txt_values = response.texts();
        
        Ok(txt_values)
    }
    
    /// 验证 DNS 管理器凭证
    pub async fn validate_credentials(&self) -> AcmeResult<bool> {
        self.dns_manager.validate_credentials().await
    }
    
    /// 获取提供商名称
    pub fn provider_name(&self) -> &str {
        self.dns_manager.provider_name()
    }
    
    /// 设置 DNS 服务器列表
    pub fn set_dns_servers(&mut self, servers: Vec<String>) {
        self.dns_servers = servers;
    }
    
    /// 设置传播超时时间
    pub fn set_propagation_timeout(&mut self, timeout_seconds: u64) {
        self.propagation_timeout = timeout_seconds;
    }
    
    /// 设置传播检查间隔
    pub fn set_propagation_interval(&mut self, interval_seconds: u64) {
        self.propagation_interval = interval_seconds;
    }
    
    /// 清理域名的所有 ACME 挑战记录
    pub async fn cleanup_challenge_records(
        &self,
        domain: &str,
        dry_run: bool,
    ) -> AcmeResult<Vec<DnsOperationResult>> {
        let challenge_name = format!("_acme-challenge.{}", domain);
        
        if dry_run {
            println!("[演练模式] 将清理域名 {} 的 ACME 挑战记录", domain);
            println!("  查找记录: {}", challenge_name);
            return Ok(vec![DnsOperationResult {
                success: true,
                record_id: Some("dry-run-cleanup".to_string()),
                error_message: None,
                duration_ms: 0,
            }]);
        }
        
        println!("🧹 开始清理域名 {} 的 ACME 挑战记录...", domain);
        println!("🔍 查找记录: {}", challenge_name);
        
        // 获取所有 TXT 记录
        let all_records = self.dns_manager.list_txt_records(domain).await?;
        
        // 过滤出 ACME 挑战记录
        let challenge_records: Vec<_> = all_records
            .into_iter()
            .filter(|record| record.name == challenge_name)
            .collect();
        
        if challenge_records.is_empty() {
            println!("✅ 没有找到需要清理的记录");
            return Ok(Vec::new());
        }
        
        println!("📋 找到 {} 条记录需要清理:", challenge_records.len());
        for record in &challenge_records {
            println!("  - ID: {:?}, 名称: {}, 值: {}", record.id, record.name, record.value);
        }
        
        let mut results = Vec::new();
        
        // 删除所有找到的记录
        for record in challenge_records {
            if let Some(record_id) = &record.id {
                println!("🗑️  删除记录: {} ({})", record.name, record_id);
                
                let result = self.dns_manager.delete_txt_record(domain, record_id).await?;
                
                if result.success {
                    println!("✅ 记录已删除");
                } else {
                    println!("❌ 删除失败: {:?}", result.error_message);
                }
                
                results.push(result);
            } else {
                println!("⚠️  跳过无 ID 的记录: {}", record.name);
                results.push(DnsOperationResult {
                    success: false,
                    record_id: None,
                    error_message: Some("记录缺少 ID".to_string()),
                    duration_ms: 0,
                });
            }
        }
        
        let successful_count = results.iter().filter(|r| r.success).count();
        println!("🎉 清理完成！成功删除 {} 条记录", successful_count);
        
        Ok(results)
    }
    
    /// 列出域名的所有 ACME 挑战记录
    pub async fn list_challenge_records(
        &self,
        domain: &str,
    ) -> AcmeResult<Vec<DnsRecord>> {
        let challenge_name = format!("_acme-challenge.{}", domain);
        
        // 获取所有 TXT 记录
        let all_records = self.dns_manager.list_txt_records(domain).await?;
        
        // 过滤出 ACME 挑战记录
        let challenge_records: Vec<_> = all_records
            .into_iter()
            .filter(|record| record.name == challenge_name)
            .collect();
        
        Ok(challenge_records)
    }
}

/// 便捷函数：创建 ACME 挑战记录名称
pub fn create_acme_challenge_name(domain: &str) -> String {
    format!("_acme-challenge.{}", domain)
}

/// 便捷函数：验证域名格式
pub fn validate_domain_name(domain: &str) -> AcmeResult<()> {
    if domain.is_empty() {
        return Err(AcmeError::InvalidDomain("域名不能为空".to_string()));
    }
    
    if domain.len() > 253 {
        return Err(AcmeError::InvalidDomain("域名过长".to_string()));
    }
    
    // 基本的域名格式检查
    let parts: Vec<&str> = domain.split('.').collect();
    if parts.len() < 2 {
        return Err(AcmeError::InvalidDomain("无效的域名格式".to_string()));
    }
    
    for part in parts {
        if part.is_empty() || part.len() > 63 {
            return Err(AcmeError::InvalidDomain("无效的域名标签".to_string()));
        }
        
        if !part.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return Err(AcmeError::InvalidDomain("域名包含无效字符".to_string()));
        }
        
        if part.starts_with('-') || part.ends_with('-') {
            return Err(AcmeError::InvalidDomain("域名标签不能以连字符开头或结尾".to_string()));
        }
    }
    
    Ok(())
}

/// 便捷函数：提取根域名
pub fn extract_root_domain(domain: &str) -> AcmeResult<String> {
    validate_domain_name(domain)?;
    
    let parts: Vec<&str> = domain.split('.').collect();
    if parts.len() >= 2 {
        Ok(format!("{}.{}", parts[parts.len() - 2], parts[parts.len() - 1]))
    } else {
        Ok(domain.to_string())
    }
}
