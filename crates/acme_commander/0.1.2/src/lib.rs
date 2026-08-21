//! ACME Commander 库
//! 
//! 一个全面的 ACME (自动证书管理环境) 客户端库
//! 用于从 ACME 兼容的证书颁发机构获取和管理 SSL/TLS 证书。
//! 
//! 该库提供了完整的 ACME 协议实现，支持以下功能：
//! - ECDSA P-384 (secp384r1) 密钥生成和管理
//! - 支持多种 DNS 提供商的 DNS-01 挑战验证
//! - 证书签名请求(CSR)生成
//! - 证书链管理和验证
//! - 全面的日志记录和错误处理
//! - 测试用的 dry-run 模式
//! - Cloudflare 和 ZeroSSL 的令牌验证

pub mod error;
pub mod logger;
pub mod crypto;
pub mod auth;
pub mod acme;
pub mod dns;
pub mod i18n;
pub mod i18n_logger;
pub mod config;

// 重新导出常用类型
pub use error::{AcmeError, AcmeResult};
pub use logger::{LogLevel, LogOutput, LogConfig, init_logger};
pub use crypto::{KeyPair, Algorithm, PemData, PemType};
pub use auth::{Provider, ValidationResult, SecureCredential};
pub use acme::{AcmeClient, AcmeConfig, OrderStatus, ChallengeType, ChallengeRecoveryManager, recover_challenge_from_authorization};
pub use dns::{DnsProvider, DnsManager, DnsChallengeManager};
pub use config::{AcmeConfig as Config, ConfigManager};

/// 库版本
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 默认 ACME 目录 URL
pub mod directories {
    /// Let's Encrypt 生产环境目录
    pub const LETSENCRYPT_PRODUCTION: &str = "https://acme-v02.api.letsencrypt.org/directory";
    
    /// Let's Encrypt 测试环境目录
    pub const LETSENCRYPT_STAGING: &str = "https://acme-staging-v02.api.letsencrypt.org/directory";
    
    /// ZeroSSL 生产环境目录
    pub const ZEROSSL_PRODUCTION: &str = "https://acme.zerossl.com/v2/DV90";
}

/// 常用操作的便捷函数
pub mod convenience {
    use super::*;
    use crate::crypto::KeyPair;
    use crate::acme::AcmeClient;
    use crate::dns::cloudflare::CloudflareDnsManager;

    use std::path::Path;
    
    /// 创建新的 ECDSA P-384 密钥对
    pub fn generate_key_pair() -> AcmeResult<KeyPair> {
        KeyPair::generate().map_err(|e| AcmeError::CryptoError(e.to_string()))
    }
    
    /// 从 PEM 文件加载密钥对
    pub fn load_key_pair_from_file<P: AsRef<Path>>(path: P) -> AcmeResult<KeyPair> {
        let pem_content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| AcmeError::IoError(format!("读取密钥文件失败: {}", e)))?;
        KeyPair::from_private_key_pem(&pem_content).map_err(|e| AcmeError::CryptoError(e.to_string()))
    }
    
    /// 创建使用 Let's Encrypt 测试环境的 ACME 客户端
    pub fn create_staging_client(account_key: KeyPair) -> AcmeResult<AcmeClient> {
        let config = AcmeConfig::new(
            directories::LETSENCRYPT_STAGING.to_string(),
            account_key.clone(),
        );
        AcmeClient::new(config, account_key)
    }
    
    /// 创建使用 Let's Encrypt 生产环境的 ACME 客户端
    pub fn create_production_client(account_key: KeyPair) -> AcmeResult<AcmeClient> {
        let config = AcmeConfig::new(
            directories::LETSENCRYPT_PRODUCTION.to_string(),
            account_key.clone(),
        );
        AcmeClient::new(config, account_key)
    }
    
    /// 创建 Cloudflare DNS 管理器
    pub fn create_cloudflare_dns(api_token: String) -> AcmeResult<Box<dyn DnsManager>> {
        let manager = CloudflareDnsManager::new(api_token)?;
        Ok(Box::new(manager))
    }
    

    
    /// 验证 Cloudflare API 令牌
    pub async fn validate_cloudflare_token(api_token: &str) -> AcmeResult<bool> {
        use crate::auth::cloudflare::verify_cloudflare_token;
        let result = verify_cloudflare_token(api_token).await
            .map_err(|e| AcmeError::HttpError(e.to_string()))?;
        match result {
            crate::auth::ValidationResult::Cloudflare { .. } => Ok(true),
            _ => Ok(false),
        }
    }
    
    /// 验证 ZeroSSL API 密钥
    pub async fn validate_zerossl_api_key(api_key: &str) -> AcmeResult<bool> {
        use crate::auth::zerossl::verify_zerossl_api_key;
        let result = verify_zerossl_api_key(api_key).await
            .map_err(|e| AcmeError::HttpError(e.to_string()))?;
        match result {
            crate::auth::ValidationResult::ZeroSsl { .. } => Ok(true),
            _ => Ok(false),
        }
    }
}

/// 高级证书管理 API
pub mod certificate {
    use super::*;
    use crate::crypto::KeyPair;
    use crate::acme::{AcmeClient, AcmeConfig};
    use crate::dns::DnsChallengeManager;
    use crate::acme::certificate::{CertificateManager, CertificateRequest};
    use std::path::Path;
    
    /// 证书颁发选项
    pub struct IssuanceOptions {
        /// 包含在证书中的域名
        pub domains: Vec<String>,
        /// 账户邮箱地址
        pub email: String,
        /// 是否使用生产环境
        pub production: bool,
        /// 是否为测试运行
        pub dry_run: bool,
        /// DNS挑战管理器
        pub dns_manager: Box<dyn DnsManager>,
        /// 证书请求详情
        pub certificate_request: Option<CertificateRequest>,
    }
    
    /// 证书颁发结果
    #[derive(Debug, Clone)]
    pub struct IssuanceResult {
        /// 证书PEM数据
        pub certificate_pem: String,
        /// 证书链PEM数据
        pub chain_pem: String,
        /// 完整链PEM数据(证书+证书链)
        pub fullchain_pem: String,
        /// 私钥PEM数据
        pub private_key_pem: String,
        /// 证书过期时间
        pub expires_at: std::time::SystemTime,
    }
    
    /// 颁发新证书
    pub async fn issue_certificate(
        account_key: KeyPair,
        certificate_key: KeyPair,
        options: IssuanceOptions,
    ) -> AcmeResult<IssuanceResult> {
        // 创建ACME客户端
        let directory_url = if options.production {
            crate::directories::LETSENCRYPT_PRODUCTION
        } else {
            crate::directories::LETSENCRYPT_STAGING
        };
        
        let acme_config = AcmeConfig::new(directory_url.to_string(), account_key.clone());
        let mut acme_client = AcmeClient::new(acme_config, account_key)?;
        
        // 注册或查找账户
        let _account = crate::acme::register_or_find_account(&mut acme_client, Some(&options.email), true, None).await?;
        
        // 创建证书管理器
        let cert_manager = CertificateManager::new(certificate_key.clone());
        
        // 生成CSR
        let cert_request = options.certificate_request.unwrap_or_else(|| {
            crate::acme::certificate::create_domain_certificate_request(
                options.domains[0].clone(),
                options.domains[1..].to_vec(),
            )
        });
        
        let csr_der = cert_manager.generate_csr(&cert_request)?;
        
        // 创建订单
        let (order, order_url) = crate::acme::order::create_domain_order(&mut acme_client, &options.domains).await?;
        
        // 处理挑战
        let dns_challenge_manager = DnsChallengeManager::new(
            options.dns_manager,
            Some(60), // TTL(生存时间)
            Some(300), // 传播超时时间
        );
        
        for authorization_url in &order.authorizations {
            let authorization = acme_client.get_authorization(authorization_url).await?;
            
            // 查找DNS-01挑战
            let dns_challenge = authorization.challenges
                .iter()
                .find(|c| c.challenge_type == crate::acme::ChallengeType::Dns01)
                .ok_or_else(|| AcmeError::AcmeProtocolError("未找到 DNS-01 挑战".to_string()))?;
            
            // 使用挑战管理器处理挑战
            let mut challenge_manager = crate::acme::challenge::ChallengeManager::new(&mut acme_client);
            
            // 准备挑战信息
            let challenge_info = challenge_manager.prepare_challenge(dns_challenge)?;
            let dns_record_value = if let crate::acme::challenge::ChallengeInfo::Dns01(dns01) = challenge_info {
                dns01.record_value
            } else {
                return Err(AcmeError::AcmeProtocolError("挑战类型不匹配".to_string()));
            };
            
            // 添加DNS记录
            let challenge_record = dns_challenge_manager.add_challenge_record(
                &authorization.identifier.value,
                &dns_record_value, // 使用正确的DNS挑战值
                options.dry_run,
            ).await?;
            
            if !options.dry_run {
                // 等待传播完成
                dns_challenge_manager.wait_for_propagation(&challenge_record, false).await?;
                
                // 响应挑战
                let _updated_challenge = challenge_manager.respond_to_challenge(dns_challenge).await?;
                
                // 等待挑战完成
                challenge_manager.wait_for_challenge_completion(
                    &dns_challenge.url,
                    30, // 最大尝试次数
                    std::time::Duration::from_secs(5), // 检查间隔
                ).await?;
                
                // 清理DNS记录
                dns_challenge_manager.delete_challenge_record(&challenge_record, false).await?;
            }
        }
        
        if options.dry_run {
            return Ok(IssuanceResult {
                certificate_pem: "[演练模式] 证书将在此处颁发".to_string(),
                chain_pem: "[演练模式] 证书链将在此处提供".to_string(),
                fullchain_pem: "[演练模式] 完整证书链将在此处提供".to_string(),
                private_key_pem: certificate_key.to_pem().to_string(),
                expires_at: std::time::SystemTime::now() + std::time::Duration::from_secs(90 * 24 * 3600),
            });
        }
        
        // 完成订单
        let finalized_order = acme_client.finalize_order(&order.finalize, &csr_der).await?;
        
        // 等待订单完成并获取证书URL
        let ready_order = acme_client.wait_for_order_ready(
            &order_url, 
            30, // 最多等待30次
            std::time::Duration::from_secs(2) // 每次等待2秒
        ).await?;
        
        // 下载证书
        let cert_url = ready_order.certificate
            .ok_or_else(|| AcmeError::AcmeProtocolError("订单完成但未提供证书下载URL".to_string()))?;
        let certificate_pem = acme_client.download_certificate(&cert_url).await?;
        
        // 解析证书链
        let cert_chain = cert_manager.parse_certificate_chain(&certificate_pem)?;
        
        // 提取仅包含中间证书的证书链
        let intermediate_chain_pem = cert_manager.extract_intermediate_chain_pem(&certificate_pem)?;
        
        Ok(IssuanceResult {
            certificate_pem: cert_chain.certificate_pem,
            chain_pem: intermediate_chain_pem, // 修复：仅中间证书链
            fullchain_pem: cert_chain.full_chain_pem,
            private_key_pem: certificate_key.to_pem().to_string(),
            expires_at: cert_chain.leaf_certificate.not_after,
        })
    }
    
    /// 将证书文件保存到磁盘
    pub fn save_certificate_files<P: AsRef<Path>>(
        result: &IssuanceResult,
        cert_file: P,
        key_file: P,
        chain_file: P,
        fullchain_file: P,
    ) -> AcmeResult<()> {
        use std::fs;
        
        fs::write(&cert_file, &result.certificate_pem)
            .map_err(|e| AcmeError::IoError(format!("写入证书文件失败: {}", e)))?;
        
        fs::write(&key_file, &result.private_key_pem)
            .map_err(|e| AcmeError::IoError(format!("写入密钥文件失败: {}", e)))?;
        
        fs::write(&chain_file, &result.chain_pem)
            .map_err(|e| AcmeError::IoError(format!("写入证书链文件失败: {}", e)))?;
        
        fs::write(&fullchain_file, &result.fullchain_pem)
            .map_err(|e| AcmeError::IoError(format!("写入完整证书链文件失败: {}", e)))?;
        
        // 设置适当的文件权限
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            
            // 设置密钥文件权限为600(仅所有者读写)
            let key_perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&key_file, key_perms)
                .map_err(|e| AcmeError::IoError(format!("设置密钥文件权限失败: {}", e)))?;
            
            // 设置证书文件权限为644(所有者读写，组/其他用户只读)
            let cert_perms = std::fs::Permissions::from_mode(0o644);
            std::fs::set_permissions(&cert_file, cert_perms.clone())
                .map_err(|e| AcmeError::IoError(format!("设置证书文件权限失败: {}", e)))?;
            std::fs::set_permissions(&chain_file, cert_perms.clone())
                .map_err(|e| AcmeError::IoError(format!("设置证书链文件权限失败: {}", e)))?;
            std::fs::set_permissions(&fullchain_file, cert_perms)
                .map_err(|e| AcmeError::IoError(format!("设置完整证书链文件权限失败: {}", e)))?;
        }
        
        Ok(())
    }
}
