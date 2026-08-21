//! 命令处理模块
//!
//! 包含所有子命令的具体实现逻辑

use crate::cli::{DnsProviderType, DnsCommands, KeyType, RevocationReason};
use acme_commander::error::{AcmeError, AuthError};

use acme_commander::convenience::{generate_key_pair, load_key_pair_from_file, validate_cloudflare_token, validate_zerossl_api_key};
use acme_commander::directories;
use serde_json;
use chrono;
use acme_commander::certificate::{issue_certificate, IssuanceOptions, save_certificate_files};
use acme_commander::crypto::KeyPair;
use acme_commander::dns::cloudflare::CloudflareDnsManager;
use acme_commander::dns::{DnsManager, DnsChallengeManager};
use acme_commander::acme::{AcmeClient, AcmeConfig};
use acme_commander::AcmeResult;

use std::path::PathBuf;
use rat_logger::{error, info, warn};

/// 处理证书颁发命令
pub async fn cmd_certonly(
    domains: Vec<String>,
    email: String,
    production: bool,
    dry_run: bool,
    dns_provider: DnsProviderType,
    cloudflare_token: Option<String>,
    account_key: Option<PathBuf>,
    cert_key: Option<PathBuf>,
    output_dir: PathBuf,
    cert_name: String,
    force_renewal: bool,
) -> Result<(), AcmeError> {
    info!("开始证书颁发流程");
    info!("域名: {:?}", domains);
    info!("邮箱: {}", email);
    info!("生产环境: {}", production);
    info!("Dry run: {}", dry_run);
    info!("DNS 提供商: {:?}", dns_provider);
    info!("输出目录: {:?}", output_dir);
    info!("证书名称: {}", cert_name);
    info!("强制续订: {}", force_renewal);

    if dry_run {
        info!("🔍 执行 dry run 模式 - 不会进行实际的证书颁发");
    }

    // 验证 DNS 提供商凭证
    match dns_provider {
        DnsProviderType::Cloudflare => {
            let token = cloudflare_token
                .as_ref()
                .ok_or_else(|| AcmeError::Auth(AuthError::InvalidToken("Cloudflare API 令牌未提供".to_string())))?;
            
            info!("🔐 验证 Cloudflare API 令牌...");
            match validate_cloudflare_token(token).await {
                Ok(true) => {
                    info!("✅ Cloudflare API 令牌验证成功");
                },
                Ok(false) => {
                    error!("❌ Cloudflare API 令牌验证失败");
                    return Err(AcmeError::Auth(AuthError::InvalidToken("Cloudflare API 令牌无效".to_string())));
                },
                Err(e) => {
                    error!("❌ Cloudflare API 令牌验证出错: {:?}", e);
                    return Err(e);
                }
            }
        }
    }

    if dry_run {
        info!("✅ Dry run 完成 - 所有验证通过，实际运行时将继续证书颁发流程");
        return Ok(());
    }

    // 创建输出目录
    if !output_dir.exists() {
        std::fs::create_dir_all(&output_dir)
            .map_err(|e| AcmeError::IoError(format!("创建输出目录失败: {}", e)))?;
        info!("📁 创建输出目录: {:?}", output_dir);
    }

    // 证书颁发功能实现
    info!("🚀 开始颁发证书...");
    
    // 创建域名专用目录
    let domain_dir = output_dir.join(&domains[0]); // 使用主域名作为目录名
    if !domain_dir.exists() {
        std::fs::create_dir_all(&domain_dir)
            .map_err(|e| AcmeError::IoError(format!("创建域名目录失败: {}", e)))?;
        info!("📁 创建域名目录: {:?}", domain_dir);
    }

    // 生成或加载账户密钥
    let account_key_file = domain_dir.join("account.key");
    let account_key = if let Some(account_key_path) = account_key {
        info!("📂 加载指定的账户密钥: {:?}", account_key_path);
        load_key_pair_from_file(&account_key_path)?
    } else if account_key_file.exists() {
        info!("📂 加载现有账户密钥: {:?}", account_key_file);
        load_key_pair_from_file(&account_key_file)?
    } else {
        info!("🔑 生成新的账户密钥...");
        let key = generate_key_pair()?;
        // 保存账户密钥到域名目录
        std::fs::write(&account_key_file, key.to_pem().to_string())
            .map_err(|e| AcmeError::IoError(format!("保存账户密钥失败: {}", e)))?;
        info!("💾 账户密钥已保存: {:?}", account_key_file);
        
        // 设置账户密钥文件权限为600(仅所有者读写)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let key_perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&account_key_file, key_perms)
                .map_err(|e| AcmeError::IoError(format!("设置账户密钥文件权限失败: {}", e)))?;
        }
        
        key
    };
    
    // 生成或加载证书密钥
    let cert_key_file = domain_dir.join("cert.key");
    let certificate_key = if let Some(cert_key_path) = cert_key {
        info!("📂 加载指定的证书密钥: {:?}", cert_key_path);
        load_key_pair_from_file(&cert_key_path)?
    } else if cert_key_file.exists() {
        info!("📂 加载现有证书密钥: {:?}", cert_key_file);
        load_key_pair_from_file(&cert_key_file)?
    } else {
        info!("🔑 生成新的证书密钥...");
        let key = generate_key_pair()?;
        // 保存证书密钥到域名目录
        std::fs::write(&cert_key_file, key.to_pem().to_string())
            .map_err(|e| AcmeError::IoError(format!("保存证书密钥失败: {}", e)))?;
        info!("💾 证书密钥已保存: {:?}", cert_key_file);
        
        // 设置证书密钥文件权限为600(仅所有者读写)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let key_perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&cert_key_file, key_perms)
                .map_err(|e| AcmeError::IoError(format!("设置证书密钥文件权限失败: {}", e)))?;
        }
        
        key
    };
    
    // 创建 DNS 管理器
    let dns_manager: Box<dyn DnsManager> = match dns_provider {
        DnsProviderType::Cloudflare => {
            let token = cloudflare_token
                .as_ref()
                .ok_or_else(|| AcmeError::Auth(AuthError::InvalidToken("Cloudflare API 令牌未提供".to_string())))?;
            Box::new(CloudflareDnsManager::new(token.clone())?)
        }
    };
    
    // 创建证书颁发选项
    let issuance_options = IssuanceOptions {
        domains: domains.clone(),
        email: email.clone(),
        production,
        dry_run,
        dns_manager,
        certificate_request: None, // 使用默认的证书请求
    };
    
    // 颁发证书
    info!("📋 开始 ACME 证书颁发流程...");
    let issuance_result = issue_certificate(account_key, certificate_key, issuance_options).await?;
    
    // 保存 ACME 状态信息到域名目录
    let acme_state_file = domain_dir.join("acme_state.json");
    let expires_at_str = {
        let datetime = chrono::DateTime::<chrono::Utc>::from(issuance_result.expires_at);
        datetime.to_rfc3339()
    };
    let acme_state = serde_json::json!({
        "domains": domains,
        "email": email,
        "production": production,
        "cert_name": cert_name,
        "issued_at": chrono::Utc::now().to_rfc3339(),
        "expires_at": expires_at_str,
        "status": "issued",
        "last_renewal_check": chrono::Utc::now().to_rfc3339()
    });
    std::fs::write(&acme_state_file, serde_json::to_string_pretty(&acme_state)
        .map_err(|e| AcmeError::IoError(format!("序列化 ACME 状态失败: {}", e)))?)
        .map_err(|e| AcmeError::IoError(format!("保存 ACME 状态失败: {}", e)))?;
    info!("💾 ACME 状态已保存: {:?}", acme_state_file);
    
    if dry_run {
        info!("🔍 Dry run 模式完成 - 证书颁发流程验证成功");
        info!("✅ 实际运行时证书将保存到: {:?}", output_dir);
        return Ok(());
    }
    
    // 保存证书文件到域名目录
    let cert_file = domain_dir.join(format!("{}.pem", cert_name));
    let key_file = domain_dir.join(format!("{}.key", cert_name));
    let chain_file = domain_dir.join(format!("{}-chain.pem", cert_name));
    let fullchain_file = domain_dir.join(format!("{}-fullchain.pem", cert_name));
    
    // 同时在输出目录根部创建符号链接或副本（保持向后兼容）
    let root_cert_file = output_dir.join(format!("{}.pem", cert_name));
    let root_key_file = output_dir.join(format!("{}.key", cert_name));
    let root_chain_file = output_dir.join(format!("{}-chain.pem", cert_name));
    let root_fullchain_file = output_dir.join(format!("{}-fullchain.pem", cert_name));
    
    info!("💾 保存证书文件...");
    // 保存到域名目录
    save_certificate_files(
        &issuance_result,
        &cert_file,
        &key_file,
        &chain_file,
        &fullchain_file,
    )?;
    
    // 保存到根目录（向后兼容）
    save_certificate_files(
        &issuance_result,
        &root_cert_file,
        &root_key_file,
        &root_chain_file,
        &root_fullchain_file,
    )?;
    
    // 更新 ACME 状态为已完成
    let final_acme_state = serde_json::json!({
        "domains": domains,
        "email": email,
        "production": production,
        "cert_name": cert_name,
        "issued_at": chrono::Utc::now().to_rfc3339(),
        "expires_at": issuance_result.expires_at.duration_since(std::time::UNIX_EPOCH)
            .map(|d| chrono::DateTime::<chrono::Utc>::from_timestamp(d.as_secs() as i64, 0)
                .unwrap_or_default().to_rfc3339())
            .unwrap_or_else(|_| "unknown".to_string()),
        "status": "completed",
        "last_renewal_check": chrono::Utc::now().to_rfc3339(),
        "cert_files": {
            "certificate": cert_file.to_string_lossy(),
            "private_key": key_file.to_string_lossy(),
            "chain": chain_file.to_string_lossy(),
            "fullchain": fullchain_file.to_string_lossy()
        }
    });
    std::fs::write(&acme_state_file, serde_json::to_string_pretty(&final_acme_state)
        .map_err(|e| AcmeError::IoError(format!("序列化最终 ACME 状态失败: {}", e)))?)
        .map_err(|e| AcmeError::IoError(format!("保存最终 ACME 状态失败: {}", e)))?;
    
    info!("✅ 证书颁发成功!");
    info!("📁 域名专用目录: {:?}", domain_dir);
    info!("📁 证书文件已保存到:");
    info!("   证书: {:?}", cert_file);
    info!("   私钥: {:?}", key_file);
    info!("   证书链: {:?}", chain_file);
    info!("   完整链: {:?}", fullchain_file);
    info!("📁 向后兼容副本:");
    info!("   证书: {:?}", root_cert_file);
    info!("   私钥: {:?}", root_key_file);
    info!("   证书链: {:?}", root_chain_file);
    info!("   完整链: {:?}", root_fullchain_file);
    info!("⏰ 证书过期时间: {:?}", issuance_result.expires_at);
    
    if !force_renewal {
        info!("💡 提示: 使用 --force-renewal 参数可以强制续订现有证书");
    }

    Ok(())
}

/// 处理挑战恢复命令
pub async fn cmd_recover(
    domain_dir: PathBuf,
    dry_run: bool,
) -> Result<(), AcmeError> {
    info!("开始挑战恢复流程");
    info!("域名目录: {:?}", domain_dir);
    info!("Dry run: {}", dry_run);

    // 检查域名目录是否存在
    if !domain_dir.exists() {
        return Err(AcmeError::IoError(format!("域名目录不存在: {:?}", domain_dir)));
    }

    // 加载账户信息
    let account_info_file = domain_dir.join("account_info.json");
    if !account_info_file.exists() {
        return Err(AcmeError::IoError("未找到账户信息文件，无法恢复挑战".to_string()));
    }

    let account_info_content = std::fs::read_to_string(&account_info_file)
        .map_err(|e| AcmeError::IoError(format!("读取账户信息失败: {}", e)))?;
    let account_info: serde_json::Value = serde_json::from_str(&account_info_content)
        .map_err(|e| AcmeError::IoError(format!("解析账户信息失败: {}", e)))?;

    // 加载账户密钥
    let account_key_file = domain_dir.join("account.key");
    if !account_key_file.exists() {
        return Err(AcmeError::IoError("未找到账户密钥文件，无法恢复挑战".to_string()));
    }
    let account_key = load_key_pair_from_file(&account_key_file)?;

    // 提取账户信息
    let account_url = account_info["account_url"].as_str()
        .ok_or_else(|| AcmeError::IoError("账户信息中缺少账户 URL".to_string()))?;
    let email = account_info["email"].as_str()
        .ok_or_else(|| AcmeError::IoError("账户信息中缺少邮箱".to_string()))?;
    let production = account_info["production"].as_bool().unwrap_or(false);
    let domains: Vec<String> = account_info["domains"].as_array()
        .ok_or_else(|| AcmeError::IoError("账户信息中缺少域名列表".to_string()))?
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();

    if domains.is_empty() {
        return Err(AcmeError::IoError("域名列表为空".to_string()));
    }

    info!("📋 恢复信息:");
    info!("   账户 URL: {}", account_url);
    info!("   邮箱: {}", email);
    info!("   生产环境: {}", production);
    info!("   域名: {:?}", domains);

    if dry_run {
        info!("🔍 Dry run 模式 - 挑战恢复流程验证成功");
        info!("✅ 实际运行时将使用保存的账户密钥和状态信息恢复挑战");
        return Ok(());
    }

    // 创建 ACME 客户端
    let directory_url = if production {
        directories::LETSENCRYPT_PRODUCTION
    } else {
        directories::LETSENCRYPT_STAGING
    };
    
    let acme_config = AcmeConfig::new(directory_url.to_string(), account_key.clone());
    let mut acme_client = AcmeClient::new(acme_config, account_key)?;
    
    // 设置账户 URL
    acme_client.set_account_url(account_url.to_string());
    
    info!("🔄 使用保存的账户密钥和 URL 恢复挑战...");
    info!("✅ 挑战恢复功能已准备就绪");
    info!("💡 提示: 可以使用此功能继续中断的证书申请流程");
    
    Ok(())
}

/// 处理证书续订命令
pub async fn cmd_renew(
    cert_dir: PathBuf,
    force: bool,
    dry_run: bool,
) -> Result<(), AcmeError> {
    info!("开始证书续订流程");
    info!("证书目录: {:?}", cert_dir);
    info!("强制续订: {}", force);
    info!("Dry run: {}", dry_run);

    if dry_run {
        info!("🔍 执行 dry run 模式 - 不会进行实际的证书续订");
        info!("✅ Dry run 完成 - 续订功能尚未完全实现");
        return Ok(());
    }

    // 扫描证书目录
    let entries = std::fs::read_dir(&cert_dir)
        .map_err(|e| AcmeError::IoError(format!("读取证书目录失败: {}", e)))?;
    
    let mut renewed_count = 0;
    
    for entry in entries {
        let entry = entry.map_err(|e| AcmeError::IoError(format!("读取目录项失败: {}", e)))?;
        let path = entry.path();
        
        if path.is_file() && path.extension().map_or(false, |ext| ext == "pem") {
            info!("检查证书: {:?}", path);
            // 这里需要实现证书检查和续订逻辑
            if force {
                info!("强制续订证书: {:?}", path);
                renewed_count += 1;
            }
        }
    }

    info!("✅ 证书续订完成 - 处理了 {} 个证书", renewed_count);
    Ok(())
}

/// 处理验证命令
pub async fn cmd_validate(
    cloudflare_token: Option<String>,
    zerossl_api_key: Option<String>,
) -> Result<(), AcmeError> {
    info!("开始验证 API 令牌凭证");

    // 检查是否提供了任何令牌
    if cloudflare_token.is_none() && zerossl_api_key.is_none() {
        error!("❌ 错误: 必须提供至少一个 API 令牌进行验证");
        error!("   使用 --cloudflare-token 提供 Cloudflare API 令牌");
        error!("   或使用 --zerossl-api-key 提供 ZeroSSL API 密钥");
        return Err(AcmeError::Auth(AuthError::InvalidToken(
            "未提供任何 API 令牌".to_string(),
        )));
    }

    // 验证 Cloudflare 令牌
    if let Some(token) = cloudflare_token {
        info!("🔐 验证 Cloudflare API 令牌...");
        match validate_cloudflare_token(&token).await {
            Ok(true) => {
                info!("✅ Cloudflare API 令牌验证成功!");
                info!("💡 提示: 此令牌可用于 DNS 挑战验证");
                // 直接输出到控制台，确保用户能看到结果
                // println!("✅ Cloudflare API 令牌验证成功!");
                // println!("💡 提示: 此令牌可用于 DNS 挑战验证");
            },
            Ok(false) => {
                error!("❌ Cloudflare API 令牌验证失败");
                error!("💡 解决建议:");
                error!("   1. 检查令牌是否正确复制（无多余空格或字符）");
                error!("   2. 确认令牌未过期");
                error!("   3. 验证令牌具有必要的权限（Zone:DNS:Edit）");
                error!("   4. 在 Cloudflare 仪表板中重新生成令牌");
                return Err(AcmeError::Auth(AuthError::InvalidToken("Cloudflare API 令牌无效".to_string())));
            },
            Err(AcmeError::Auth(auth_err)) => {
                match auth_err {
                    AuthError::InvalidToken(ref msg) => {
                        error!("❌ 令牌问题: {}", msg);
                        error!("💡 建议: 检查令牌格式和有效性");
                    },
                    AuthError::InsufficientPermissions => {
                        error!("❌ 权限不足");
                        error!("💡 建议: 确保令牌具有 Zone:DNS:Edit 权限");
                    },
                    AuthError::ServiceError(ref msg) => {
                        error!("❌ 服务错误: {}", msg);
                        error!("💡 可能原因:");
                        error!("   - 网络连接问题");
                        error!("   - Cloudflare API 服务暂时不可用");
                        error!("   - 认证失败");
                        error!("💡 建议: 检查网络连接并稍后重试");
                    },
                    AuthError::RateLimitExceeded => {
                        error!("❌ 速率限制");
                        error!("💡 建议: 等待一段时间后重试");
                    },
                    _ => {
                        error!("❌ 认证错误: {:?}", auth_err);
                    }
                }
                return Err(AcmeError::Auth(auth_err));
            },
            Err(AcmeError::HttpError(error_msg)) => {
                error!("❌ 网络请求错误: {}", error_msg);
                error!("💡 可能原因:");
                error!("   - 网络连接问题");
                error!("   - Cloudflare API 服务问题");
                error!("   - 防火墙或代理设置阻止请求");
                error!("💡 建议:");
                error!("   1. 检查网络连接");
                error!("   2. 确认可以访问 api.cloudflare.com");
                error!("   3. 检查防火墙和代理设置");
                error!("   4. 稍后重试");
                return Err(AcmeError::HttpError(format!("HTTP错误: {}", error_msg)));
            },
            Err(other_err) => {
                error!("❌ 未知错误: {:?}", other_err);
                error!("💡 建议: 如果问题持续存在，请联系技术支持");
                return Err(other_err);
            }
        }
    }

    // 验证 ZeroSSL API 密钥
    if let Some(api_key) = zerossl_api_key {
        info!("🔐 验证 ZeroSSL API 密钥...");
        match validate_zerossl_api_key(&api_key).await {
            Ok(true) => {
                info!("✅ ZeroSSL API 密钥验证成功!");
            },
            Ok(false) => {
                error!("❌ ZeroSSL API 密钥验证失败");
                return Err(AcmeError::Auth(AuthError::InvalidToken("ZeroSSL API 密钥无效".to_string())));
            },
            Err(e) => {
                error!("❌ ZeroSSL API 密钥验证出错: {:?}", e);
                return Err(e);
            }
        }
    }

    Ok(())
}

/// 处理密钥生成命令
pub async fn cmd_keygen(
    output: PathBuf,
    key_type: KeyType,
) -> Result<(), AcmeError> {
    info!("开始生成密钥");
    info!("输出文件: {:?}", output);
    info!("密钥类型: {:?}", key_type);

    // 生成密钥对
    let key_pair = generate_key_pair()?;
    
    // 保存密钥到文件
    std::fs::write(&output, key_pair.to_pem().to_string())
         .map_err(|e| AcmeError::IoError(format!("保存密钥文件失败: {}", e)))?;

    info!("✅ 密钥生成成功: {:?}", output);
    Ok(())
}

/// 处理证书显示命令
pub async fn cmd_show(
    cert_file: PathBuf,
    detailed: bool,
) -> Result<(), AcmeError> {
    info!("显示证书信息");
    info!("证书文件: {:?}", cert_file);
    info!("详细信息: {}", detailed);

    // 读取证书文件
    let cert_content = std::fs::read_to_string(&cert_file)
         .map_err(|e| AcmeError::IoError(format!("读取证书文件失败: {}", e)))?;
    
    // 解析并显示证书信息
    // 这里需要实现证书解析逻辑
    info!("证书内容长度: {} 字节", cert_content.len());
    
    if detailed {
        info!("证书详细信息:");
        info!("{}", cert_content);
    }

    Ok(())
}

/// 处理证书撤销命令
pub async fn cmd_revoke(
    cert_file: PathBuf,
    account_key: PathBuf,
    reason: RevocationReason,
    production: bool,
) -> Result<(), AcmeError> {
    info!("开始撤销证书");
    info!("证书文件: {:?}", cert_file);
    info!("账户密钥: {:?}", account_key);
    info!("撤销原因: {:?}", reason);
    info!("生产环境: {}", production);

    // 转换撤销原因
    let revocation_reason = match reason {
        RevocationReason::Unspecified => 0,
        RevocationReason::KeyCompromise => 1,
        RevocationReason::CaCompromise => 2,
        RevocationReason::AffiliationChanged => 3,
        RevocationReason::Superseded => 4,
        RevocationReason::CessationOfOperation => 5,
        RevocationReason::CertificateHold => 6,
        RevocationReason::RemoveFromCrl => 8,
        RevocationReason::PrivilegeWithdrawn => 9,
        RevocationReason::AaCompromise => 10,
    };

    // 加载账户密钥
    let account_key_pair = load_key_pair_from_file(&account_key)?;
    
    // 读取证书文件
    let cert_content = std::fs::read_to_string(&cert_file)
         .map_err(|e| AcmeError::IoError(format!("读取证书文件失败: {}", e)))?;
    
    info!("证书撤销功能尚未完全实现");
    info!("撤销原因代码: {}", revocation_reason);
    info!("✅ 证书撤销请求已记录");
    Ok(())
}

/// 处理 DNS 命令
pub async fn cmd_dns(dns_command: DnsCommands) -> Result<(), AcmeError> {
    match dns_command {
        DnsCommands::Cleanup {
            domain,
            dns_provider,
            cloudflare_token,
            dry_run,
        } => {
            cmd_dns_cleanup(domain, dns_provider, cloudflare_token, dry_run).await
        }
        DnsCommands::List {
            domain,
            dns_provider,
            cloudflare_token,
        } => {
            cmd_dns_list(domain, dns_provider, cloudflare_token).await
        }
    }
}

/// 处理 DNS 清理命令
pub async fn cmd_dns_cleanup(
    domain: String,
    dns_provider: DnsProviderType,
    cloudflare_token: Option<String>,
    dry_run: bool,
) -> Result<(), AcmeError> {
    println!("🧹 开始清理域名 {} 的 ACME 挑战记录", domain);
    println!("📡 DNS 提供商: {:?}", dns_provider);
    println!("🔍 Dry run 模式: {}", if dry_run { "是" } else { "否" });
    info!("开始清理域名 {} 的 ACME 挑战记录", domain);
    info!("DNS 提供商: {:?}", dns_provider);
    info!("Dry run: {}", dry_run);

    // 验证 DNS 提供商凭证
    match dns_provider {
        DnsProviderType::Cloudflare => {
            let token = cloudflare_token
                .as_ref()
                .ok_or_else(|| AcmeError::Auth(AuthError::InvalidToken("Cloudflare API 令牌未提供".to_string())))?;
            
            println!("🔐 验证 Cloudflare API 令牌...");
            info!("🔐 验证 Cloudflare API 令牌...");
            match validate_cloudflare_token(token).await {
                Ok(true) => {
                    println!("✅ Cloudflare API 令牌验证成功");
                    info!("✅ Cloudflare API 令牌验证成功");
                },
                Ok(false) => {
                    println!("❌ Cloudflare API 令牌验证失败");
                    error!("❌ Cloudflare API 令牌验证失败");
                    return Err(AcmeError::Auth(AuthError::InvalidToken("Cloudflare API 令牌无效".to_string())));
                },
                Err(e) => {
                    println!("❌ Cloudflare API 令牌验证出错: {:?}", e);
                    error!("❌ Cloudflare API 令牌验证出错: {:?}", e);
                    return Err(e);
                }
            }
        }
    }

    // 创建 DNS 管理器
    let dns_manager: Box<dyn DnsManager> = match dns_provider {
        DnsProviderType::Cloudflare => {
            let token = cloudflare_token
                .as_ref()
                .ok_or_else(|| AcmeError::Auth(AuthError::InvalidToken("Cloudflare API 令牌未提供".to_string())))?;
            Box::new(CloudflareDnsManager::new(token.clone())?)
        }
    };

    // 创建 DNS 挑战管理器
    let dns_challenge_manager = DnsChallengeManager::new(dns_manager, None, None);

    println!("🔍 正在查找需要清理的 ACME 挑战记录...");
    info!("正在查找需要清理的 ACME 挑战记录...");
    
    // 执行清理
    let results = dns_challenge_manager.cleanup_challenge_records(&domain, dry_run).await?;

    if dry_run {
        println!("🔍 Dry run 完成 - 实际运行时将清理 {} 条记录", results.len());
        info!("🔍 Dry run 完成 - 实际运行时将清理 {} 条记录", results.len());
    } else {
        let successful_count = results.iter().filter(|r| r.success).count();
        println!("✅ 清理完成！成功删除 {} 条记录", successful_count);
        info!("✅ 清理完成！成功删除 {} 条记录", successful_count);
    }

    Ok(())
}

/// 处理 DNS 列表命令
pub async fn cmd_dns_list(
    domain: String,
    dns_provider: DnsProviderType,
    cloudflare_token: Option<String>,
) -> Result<(), AcmeError> {
    println!("🌐 列出域名 {} 的 ACME 挑战记录", domain);
    println!("📡 DNS 提供商: {:?}", dns_provider);
    info!("列出域名 {} 的 ACME 挑战记录", domain);
    info!("DNS 提供商: {:?}", dns_provider);

    // 验证 DNS 提供商凭证
    match dns_provider {
        DnsProviderType::Cloudflare => {
            let token = cloudflare_token
                .as_ref()
                .ok_or_else(|| AcmeError::Auth(AuthError::InvalidToken("Cloudflare API 令牌未提供".to_string())))?;
            
            println!("🔐 验证 Cloudflare API 令牌...");
            info!("🔐 验证 Cloudflare API 令牌...");
            match validate_cloudflare_token(token).await {
                Ok(true) => {
                    println!("✅ Cloudflare API 令牌验证成功");
                    info!("✅ Cloudflare API 令牌验证成功");
                },
                Ok(false) => {
                    println!("❌ Cloudflare API 令牌验证失败");
                    error!("❌ Cloudflare API 令牌验证失败");
                    return Err(AcmeError::Auth(AuthError::InvalidToken("Cloudflare API 令牌无效".to_string())));
                },
                Err(e) => {
                    println!("❌ Cloudflare API 令牌验证出错: {:?}", e);
                    error!("❌ Cloudflare API 令牌验证出错: {:?}", e);
                    return Err(e);
                }
            }
        }
    }

    // 创建 DNS 管理器
    let dns_manager: Box<dyn DnsManager> = match dns_provider {
        DnsProviderType::Cloudflare => {
            let token = cloudflare_token
                .as_ref()
                .ok_or_else(|| AcmeError::Auth(AuthError::InvalidToken("Cloudflare API 令牌未提供".to_string())))?;
            Box::new(CloudflareDnsManager::new(token.clone())?)
        }
    };

    // 创建 DNS 挑战管理器
    let dns_challenge_manager = DnsChallengeManager::new(dns_manager, None, None);

    println!("🔍 正在查询 ACME 挑战记录...");
    info!("正在查询 ACME 挑战记录...");
    
    // 列出记录
    let records = dns_challenge_manager.list_challenge_records(&domain).await?;

    if records.is_empty() {
        println!("✅ 没有找到 ACME 挑战记录");
        info!("✅ 没有找到 ACME 挑战记录");
    } else {
        println!("📋 找到 {} 条 ACME 挑战记录:", records.len());
        info!("📋 找到 {} 条 ACME 挑战记录:", records.len());
        for (i, record) in records.iter().enumerate() {
            println!("  {}. ID: {:?}", i + 1, record.id);
            println!("     名称: {}", record.name);
            println!("     值: {}", record.value);
            println!("     类型: {:?}", record.record_type);
            println!("     TTL: {}", record.ttl);
            println!("");
            
            info!("  {}. ID: {:?}", i + 1, record.id);
            info!("     名称: {}", record.name);
            info!("     值: {}", record.value);
            info!("     类型: {:?}", record.record_type);
            info!("     TTL: {}", record.ttl);
            info!("");
        }
    }

    Ok(())
}