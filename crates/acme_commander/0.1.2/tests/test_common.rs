//! 测试辅助函数和通用配置
//!
//! 提供测试中常用的初始化、配置和辅助方法，减少重复代码

use acme_commander::logger::{init_logger, LogConfig, LogLevel, LogOutput};
use acme_commander::crypto::KeyPair;
use acme_commander::config;
use acme_commander::dns::cloudflare::CloudflareDnsManager;
use acme_commander::dns::{DnsManager, DnsChallengeManager};
use acme_commander::acme::{AcmeClient, AcmeConfig};
use std::time::Duration;

/// 常用测试域名
pub const TEST_DOMAIN: &str = "gs1.sukiyaki.su";

/// 默认测试配置
pub const DEFAULT_DNS_TTL: u32 = 300;
pub const DEFAULT_DNS_TIMEOUT: u64 = 600;
pub const DEFAULT_ACME_TIMEOUT: u32 = 30;

/// 初始化测试日志系统
pub fn init_test_logger() {
    let _ = init_logger(LogConfig {
        level: LogLevel::Debug,
        output: LogOutput::Terminal,
        ..Default::default()
    });
}

/// 创建测试用的账户密钥
pub fn create_test_account_key() -> KeyPair {
    KeyPair::generate().expect("无法生成账户密钥")
}

/// 创建测试用的证书密钥
pub fn create_test_certificate_key() -> KeyPair {
    KeyPair::generate().expect("无法生成证书密钥")
}

/// 创建测试用的Cloudflare DNS管理器
pub async fn create_test_cloudflare_manager() -> Result<Box<dyn DnsManager>, String> {
    match config::get_cloudflare_token(None)
        .map(|token| CloudflareDnsManager::new(token))
        .transpose() {
        Ok(Some(manager)) => {
            // 验证凭证
            match manager.validate_credentials().await {
                Ok(true) => Ok(Box::new(manager)),
                Ok(false) => Err("Cloudflare API Token 无效".to_string()),
                Err(e) => Err(format!("Cloudflare 凭证验证失败: {}", e)),
            }
        },
        Ok(None) => Err("未找到 Cloudflare API Token 配置".to_string()),
        Err(e) => Err(format!("创建 Cloudflare DNS 管理器失败: {}", e)),
    }
}

/// 创建测试用的DNS挑战管理器
pub fn create_test_dns_challenge_manager(dns_manager: Box<dyn DnsManager>) -> DnsChallengeManager {
    DnsChallengeManager::new(
        dns_manager,
        Some(DEFAULT_DNS_TTL),
        Some(DEFAULT_DNS_TIMEOUT),
    )
}

/// 创建自定义DNS挑战管理器（允许指定TTL和超时）
pub fn create_custom_dns_challenge_manager(dns_manager: Box<dyn DnsManager>, ttl: u32, timeout: u64) -> DnsChallengeManager {
    DnsChallengeManager::new(
        dns_manager,
        Some(ttl),
        Some(timeout),
    )
}

/// 创建测试用的ACME配置（沙盒模式）
pub fn create_test_acme_config(email: Option<String>) -> AcmeConfig {
    AcmeConfig {
        directory_url: acme_commander::directories::LETSENCRYPT_STAGING.to_string(),
        contact_email: email,
        terms_of_service_agreed: true,
        eab_credentials: None,
        timeout: Duration::from_secs(DEFAULT_ACME_TIMEOUT as u64),
        dry_run: false,
        user_agent: "acme-commander-test/0.1.1".to_string(),
    }
}

/// 创建测试用的ACME客户端
pub fn create_test_acme_client(config: AcmeConfig, account_key: KeyPair) -> Result<AcmeClient, acme_commander::error::AcmeError> {
    AcmeClient::new(config, account_key)
}

/// 测试用的域名列表
pub fn get_test_domains() -> Vec<String> {
    vec![TEST_DOMAIN.to_string()]
}

/// 加载测试配置文件
pub fn load_test_config() -> Result<config::AcmeConfig, Box<dyn std::error::Error>> {
    config::load_config(Some("config.toml".into()), None)
        .map_err(|e| format!("加载测试配置失败: {}", e).into())
}