//! acme_commander 库的完整证书申请测试
//! 使用 gs1.sukiyaki.su 域名在沙盒模式下运行，真实申请并保存证书

mod test_common;
use test_common::*;
use acme_commander::acme::{AcmeClient, AcmeConfig, OrderManager, AccountManager, ChallengeType};
use acme_commander::acme::challenge::ChallengeManager;
use acme_commander::crypto::KeyPair;
use acme_commander::logger::{init_logger, LogConfig, LogLevel, LogOutput};
use acme_commander::error::AcmeResult;
use acme_commander::config;
use acme_commander::dns::{DnsProvider, DnsManager, DnsChallengeManager};
use acme_commander::dns::cloudflare::CloudflareDnsManager;
use acme_commander::{acme_info, acme_debug, acme_warn, acme_error};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// 测试证书输出目录
const TEST_OUTPUT_DIR: &str = "./test_certs";

// 注释：extract_der_from_pem 功能已移至框架层 CertificateManager::extract_der_from_pem

/// 配置文件路径常量
const CONFIG_FILE: &str = "config.toml";

/// 主测试函数 - 完整的证书申请流程测试
#[tokio::test]
async fn test_complete_certificate_issuance() -> AcmeResult<()> {
    // 初始化acme_commander集成日志系统 - Debug级别
    init_logger(LogConfig {
        level: LogLevel::Debug,
        output: LogOutput::Terminal,
        ..Default::default()
    }).expect("初始化日志失败");

    acme_info!("=== 开始完整证书申请测试 ===");
    acme_info!("测试模式: 沙盒模式 (Let's Encrypt 测试环境)");
    acme_info!("输出目录: {}", TEST_OUTPUT_DIR);

    // 第一步：加载配置
    acme_info!("\n[步骤 1] 加载配置文件");
    let app_config = config::load_config(Some(CONFIG_FILE.into()), None)
        .map_err(|e| acme_commander::error::AcmeError::ConfigError(
            format!("加载配置文件失败: {}", e)
        ))?;
    acme_info!("✅ 成功加载配置文件");
    acme_info!("测试域名: {:?}", app_config.certificate.domains);
    acme_debug!("  ACME服务器: {}", app_config.acme.server_name());
    acme_debug!("  DNS提供商: {}", app_config.dns.provider);

    // 第二步：准备输出目录
    acme_info!("\n[步骤 2] 准备输出目录");
    prepare_output_directory()?;
    acme_info!("✅ 成功准备输出目录");

    // 第三步：创建账户密钥
    acme_info!("\n[步骤 3] 创建账户密钥");
    let account_key = create_test_account_key();
    acme_debug!("✅ 成功生成账户密钥");
    acme_debug!("  密钥类型: ECDSA (P-256)");

    // 第四步：创建ACME客户端配置
    acme_info!("\n[步骤 4] 创建ACME客户端");
    let acme_config = AcmeConfig {
        directory_url: app_config.acme.directory_url().to_string(),
        contact_email: Some(app_config.account.email.clone()),
        terms_of_service_agreed: true,
        eab_credentials: None,
        timeout: Duration::from_secs(app_config.acme.timeout_seconds),
        dry_run: false,
        user_agent: app_config.acme.user_agent.clone(),
    };

    let mut acme_client = AcmeClient::new(acme_config, account_key.clone())?;
    acme_debug!("✅ 成功创建ACME客户端");

    // 第五步：注册账户
    acme_info!("\n[步骤 5] 注册ACME账户");
    let mut account_manager = AccountManager::new(&mut acme_client);
    let (account, _) = account_manager.register_account(None, true, None).await?;
    acme_debug!("✅ 成功注册ACME账户");

    // 第六步：创建证书密钥
    acme_info!("\n[步骤 6] 创建证书密钥");
    let cert_key = create_test_certificate_key();
    acme_debug!("✅ 成功生成证书密钥");
    acme_debug!("  密钥类型: ECDSA (P-256)");

    // 第七步：申请新证书
    acme_info!("\n[步骤 7] 申请新证书");
    let (mut order, order_url) = {
        let mut order_manager = OrderManager::new(&mut acme_client);
        order_manager.create_order(
            &app_config.certificate.domains,
            None, // not_before
            None, // not_after
        ).await?
    };
    acme_debug!("✅ 成功创建订单");
    acme_debug!("  订单状态: {:?}", order.status);
    acme_debug!("  订单URL: {}", order_url);

    // 第八步：处理授权和挑战
    acme_info!("\n[步骤 8] 处理授权和DNS挑战");
    process_authorizations(&mut acme_client, &mut order, &app_config).await?;
    acme_info!("✅ 成功完成所有DNS挑战");

    // 第九步：等待订单就绪
    acme_info!("\n[步骤 9] 等待订单就绪");
    let ready_order = {
        let mut order_manager = OrderManager::new(&mut acme_client);
        order_manager.wait_for_order_ready(
            &order_url,
            30, // 最多等待30次
            Duration::from_secs(2) // 每次等待2秒
        ).await?
    };
    acme_debug!("✅ 订单已就绪，可以完成证书签发");
    acme_debug!("  最终订单状态: {:?}", ready_order.status);

    // 第十步：完成订单（发送CSR）
    acme_info!("\n[步骤 10] 完成订单（发送CSR）");

    // 使用框架层的CSR处理方法
    let cert_manager = acme_commander::acme::certificate::CertificateManager::new(cert_key.clone());
    let (csr_der, csr_pem) = cert_manager.prepare_domain_csr(
        &app_config.certificate.csr_file,
        &app_config.certificate.domains
    )?;

    // 记录CSR处理方式
    if let Some(ref csr_file) = app_config.certificate.csr_file {
        if csr_file.exists() {
            acme_debug!("📁 使用预生成的CSR文件: {}", csr_file.display());
        } else {
            acme_debug!("📝 CSR文件不存在，已生成并保存到: {}", csr_file.display());
        }
    } else {
        acme_debug!("📝 未配置CSR文件路径，已生成内存中的CSR");
    }
    acme_debug!("✅ CSR准备完成");

    // 完成订单
    let finalized_order = {
        let mut order_manager = OrderManager::new(&mut acme_client);
        order_manager.finalize_order(&ready_order, &csr_der).await?
    };
    acme_debug!("✅ 订单完成请求发送成功");
    acme_debug!("  订单状态: {:?}", finalized_order.status);

    // 等待订单变为valid状态
    let valid_order = {
        let mut order_manager = OrderManager::new(&mut acme_client);
        order_manager.wait_for_order_ready(
            &order_url,
            30, // 最多等待30次
            Duration::from_secs(2) // 每次等待2秒
        ).await?
    };
    acme_debug!("✅ 订单已生效，可以下载证书");
    acme_debug!("  最终订单状态: {:?}", valid_order.status);

    // 第十一步：下载并保存证书
    acme_info!("\n[步骤 11] 下载并保存证书");
    let cert_url = valid_order.certificate
        .ok_or_else(|| acme_commander::error::AcmeError::ProtocolError(
            "订单完成但未提供证书下载URL".to_string()
        ))?;
    acme_debug!("  证书下载URL: {}", cert_url);

    let certificate_pem = acme_client.download_certificate(&cert_url).await?;
    acme_debug!("✅ 证书下载成功");
    acme_debug!("  证书PEM长度: {} 字节", certificate_pem.len());

    // 解析证书链并保存
    save_certificate_pem_files(&certificate_pem, &cert_key, &app_config.certificate.domains, &app_config.certificate.csr_file)?;
    acme_info!("✅ 成功下载并保存证书文件");

    // 第十二步：验证证书文件
    acme_info!("\n[步骤 12] 验证证书文件");
    verify_certificate_files(&app_config.certificate.domains)?;
    acme_info!("✅ 所有证书文件验证通过");

    acme_info!("\n=== 🎉 完整证书申请测试成功 ===");
    acme_info!("证书文件已保存到: {}", TEST_OUTPUT_DIR);

    Ok(())
}

/// 准备输出目录
fn prepare_output_directory() -> AcmeResult<()> {
    let path = Path::new(TEST_OUTPUT_DIR);

    // 如果目录存在，先删除
    if path.exists() {
        fs::remove_dir_all(path)
            .map_err(|e| acme_commander::error::AcmeError::IoError(
                format!("删除输出目录失败: {}", e)
            ))?;
    }

    // 创建目录
    fs::create_dir_all(path)
        .map_err(|e| acme_commander::error::AcmeError::IoError(
            format!("创建输出目录失败: {}", e)
        ))?;

    Ok(())
}

/// 处理授权和DNS挑战
async fn process_authorizations(
    acme_client: &mut AcmeClient,
    order: &mut acme_commander::acme::order::Order,
    app_config: &config::AcmeConfig,
) -> AcmeResult<()> {
    // 创建OrderManager来获取授权信息
    let mut order_manager = OrderManager::new(acme_client);
    let authorizations = order_manager.get_order_authorizations(order).await?;
    acme_debug!("获取到 {} 个授权", authorizations.len());

    for authorization in authorizations {
        acme_debug!("处理域名 {} 的授权", authorization.identifier.value);
        acme_debug!("  授权状态: {:?}", authorization.status);

        // 查找DNS-01挑战
        let dns_challenge = authorization.challenges.iter()
            .find(|c| c.challenge_type == ChallengeType::Dns01)
            .ok_or_else(|| acme_commander::error::AcmeError::ProtocolError(
                "未找到DNS-01挑战".to_string()
            ))?;

        acme_debug!("✅ 找到DNS-01挑战");
        acme_debug!("  挑战类型: {:?}", dns_challenge.challenge_type);
        acme_debug!("  挑战状态: {:?}", dns_challenge.status);

        // 使用ChallengeManager处理挑战
        let mut challenge_manager = ChallengeManager::new(acme_client);

        // 准备挑战信息
        let challenge_info = challenge_manager.prepare_challenge(dns_challenge)?;
        acme_debug!("✅ 准备挑战信息成功");

        // 获取DNS记录值
        let dns_record_value = if let acme_commander::acme::challenge::ChallengeInfo::Dns01(dns01) = challenge_info {
            dns01.record_value
        } else {
            return Err(acme_commander::error::AcmeError::ProtocolError(
                "挑战类型不匹配".to_string()
            ));
        };

        acme_debug!("  DNS记录值: {}", dns_record_value);

        // 创建DNS挑战管理器
        let dns_manager = create_dns_manager(app_config).await?;
        let mut dns_challenge_manager = create_custom_dns_challenge_manager(dns_manager, 300, 600);

        // 添加DNS记录
        let challenge_record = dns_challenge_manager.add_challenge_record(
            &authorization.identifier.value,
            &dns_record_value,
            false, // 非dry-run模式
        ).await?;
        acme_debug!("✅ DNS记录添加成功");

        // 等待DNS传播
        acme_debug!("等待DNS传播...");
        dns_challenge_manager.wait_for_propagation(&challenge_record, false).await?;

        // 响应挑战
        let updated_challenge = challenge_manager.respond_to_challenge(dns_challenge).await?;
        acme_debug!("✅ 挑战响应成功");
        acme_debug!("  挑战状态: {:?}", updated_challenge.status);

        // 等待挑战完成
        challenge_manager.wait_for_challenge_completion(
            &dns_challenge.url,
            30, // 最多尝试30次
            Duration::from_secs(5), // 每次等待5秒
        ).await?;
        acme_debug!("✅ 挑战完成并验证成功");

        // 清理DNS记录（添加错误处理，确保即使清理失败也不会影响整体流程）
        if let Err(cleanup_error) = dns_challenge_manager.delete_challenge_record(&challenge_record, false).await {
            acme_warn!("⚠️  DNS记录清理失败: {}", cleanup_error);
            acme_debug!("  域名: {}", authorization.identifier.value);
            acme_debug!("  记录值: {}", dns_record_value);
            acme_warn!("  建议：请手动清理DNS记录以避免资源泄漏");
        } else {
            acme_debug!("✅ DNS记录清理完成");
        }
    }

    Ok(())
}

/// 创建DNS管理器
async fn create_dns_manager(app_config: &config::AcmeConfig) -> AcmeResult<Box<dyn DnsManager>> {
    match app_config.dns.provider.as_str() {
        "cloudflare" => {
            let cloudflare_token = config::get_cloudflare_token(Some(CONFIG_FILE.into()))
                .ok_or_else(|| acme_commander::error::AcmeError::ConfigError(
                    "未配置Cloudflare API Token".to_string()
                ))?;

            let dns_manager = CloudflareDnsManager::new(cloudflare_token)?;

            // 验证凭据
            if dns_manager.validate_credentials().await? {
                acme_debug!("✅ Cloudflare DNS 凭据验证成功");
                Ok(Box::new(dns_manager))
            } else {
                Err(acme_commander::error::AcmeError::ConfigError(
                    "Cloudflare DNS 凭据验证失败".to_string()
                ))
            }
        },
        _ => {
            Err(acme_commander::error::AcmeError::ConfigError(
                format!("不支持的DNS提供商: {}", app_config.dns.provider)
            ))
        }
    }
}

/// 保存PEM格式证书文件
fn save_certificate_pem_files(
    certificate_pem: &str,
    cert_key: &KeyPair,
    domains: &[String],
    csr_file: &Option<PathBuf>,
) -> AcmeResult<()> {
    let output_dir = Path::new(TEST_OUTPUT_DIR);

    acme_debug!("开始保存证书文件到目录: {}", TEST_OUTPUT_DIR);

    // 使用第一个域名作为文件名
    let primary_domain = domains.first().cloned().unwrap_or_else(|| "unknown".to_string());

    // 保存私钥
    let key_path = output_dir.join(format!("{}.key", primary_domain));
    let key_pem = cert_key.private_key_pem();
    fs::write(&key_path, key_pem)
        .map_err(|e| acme_commander::error::AcmeError::IoError(
            format!("保存私钥失败: {}", e)
        ))?;
    acme_debug!("  📁 私钥: {}", key_path.display());
    acme_debug!("  私钥长度: {} 字节", key_pem.len());

    // 尝试分离证书和证书链
    let cert_parts: Vec<&str> = certificate_pem.split("-----END CERTIFICATE-----").collect();

    if cert_parts.len() > 1 {
        // 第一个证书是叶子证书
        let leaf_cert = format!("{}-----END CERTIFICATE-----", cert_parts[0].trim());

        // 保存叶子证书（单独证书）
        let cert_path = output_dir.join(format!("{}.pem", primary_domain));
        fs::write(&cert_path, leaf_cert)
            .map_err(|e| acme_commander::error::AcmeError::IoError(
                format!("保存证书失败: {}", e)
            ))?;
        acme_debug!("  📁 证书: {}", cert_path.display());

        // 其余的是中间证书
        let chain_content: String = cert_parts[1..].iter()
            .filter_map(|part| {
                let trimmed = part.trim();
                if !trimmed.is_empty() {
                    Some(format!("{}-----END CERTIFICATE-----", trimmed))
                } else {
                    None
                }
            })
            .collect::<String>();

        if !chain_content.is_empty() {
            let chain_path = output_dir.join(format!("{}.chain.pem", primary_domain));
            fs::write(&chain_path, chain_content)
                .map_err(|e| acme_commander::error::AcmeError::IoError(
                    format!("保存证书链失败: {}", e)
                ))?;
            acme_debug!("  📁 证书链: {}", chain_path.display());
        }
    }

    // 保存完整证书链
    let fullchain_path = output_dir.join(format!("{}.fullchain.pem", primary_domain));
    fs::write(&fullchain_path, certificate_pem)
        .map_err(|e| acme_commander::error::AcmeError::IoError(
            format!("保存完整证书失败: {}", e)
        ))?;
    acme_debug!("  📁 完整证书: {}", fullchain_path.display());
    acme_debug!("  完整证书长度: {} 字节", certificate_pem.len());

    // 如果配置了CSR文件，复制到测试输出目录
    if let Some(csr_file) = csr_file {
        if csr_file.exists() {
            let csr_dest_path = output_dir.join(format!("{}.csr", primary_domain));
            fs::copy(csr_file, &csr_dest_path)
                .map_err(|e| acme_commander::error::AcmeError::IoError(
                    format!("复制CSR文件失败: {}", e)
                ))?;
            acme_debug!("  📁 CSR文件: {}", csr_dest_path.display());
        }
    }

    Ok(())
}

/// 验证证书文件（使用框架层的验证方法）
fn verify_certificate_files(domains: &[String]) -> AcmeResult<()> {
    use acme_commander::acme::certificate::CertificateManager;

    let output_dir = Path::new(TEST_OUTPUT_DIR);
    let primary_domain = domains.first().cloned().unwrap_or_else(|| "unknown".to_string());

    acme_debug!("开始验证证书文件...");

    // 使用框架层的详细验证方法
    let cert_manager = CertificateManager::new(KeyPair::generate()?); // 临时创建用于验证
    let validation_result = cert_manager.validate_certificate_files(
        output_dir,
        &primary_domain,
        true // 包含CSR验证
    )?;

    // 输出详细验证结果
    acme_debug!("📋 证书文件验证结果:");
    acme_debug!("  私钥文件: {}", if validation_result.private_key_valid { "✅ 有效" } else { "❌ 无效" });
    acme_debug!("  完整证书链: {}", if validation_result.full_chain_valid { "✅ 有效" } else { "❌ 无效" });
    acme_debug!("  单独证书: {}", if validation_result.certificate_valid { "✅ 有效" } else { "ℹ️ 不存在" });
    acme_debug!("  证书链文件: {}", if validation_result.chain_valid { "✅ 有效" } else { "ℹ️ 不存在" });
    acme_debug!("  CSR文件: {}", if validation_result.csr_valid { "✅ 有效" } else { "ℹ️ 不存在" });
    acme_debug!("  证书总数: {}", validation_result.certificate_count);

    // 输出文件大小信息
    if !validation_result.file_sizes.is_empty() {
        acme_debug!("📁 文件大小信息:");
        for (file_type, size) in &validation_result.file_sizes {
            acme_debug!("  {}: {} 字节", file_type, size);
        }
    }

    // 检查必需文件是否有效
    if !validation_result.is_all_valid() {
        return Err(acme_commander::error::AcmeError::IoError(
            format!("证书文件验证失败: {}", validation_result.summary())
        ));
    }

    acme_info!("🎉 所有证书文件验证通过！");
    Ok(())
}

/// 测试配置文件加载
#[test]
fn test_config_loading() {
    // 初始化acme_commander集成日志系统 - Debug级别
    if let Err(_) = init_logger(LogConfig {
        level: LogLevel::Debug,
        output: LogOutput::Terminal,
        ..Default::default()
    }) {
        // 如果初始化失败，继续执行但不记录日志
    }

    acme_info!("=== 测试配置文件加载 ===");

    let result = config::load_config(Some("config.toml".into()), None);
    match result {
        Ok(config) => {
            acme_info!("✅ 成功加载配置文件");
            acme_debug!("  ACME服务器: {}", config.acme.directory_url());
            acme_debug!("  DNS提供商: {}", config.dns.provider);

            if let Some(cloudflare) = &config.dns.cloudflare {
                if let Some(token) = &cloudflare.api_token {
                    if !token.is_empty() {
                        acme_debug!("  Cloudflare Token: 已配置");
                    } else {
                        acme_warn!("  ⚠️  Cloudflare Token: 未配置");
                    }
                } else {
                    acme_warn!("  ⚠️  Cloudflare Token: 未配置");
                }
            } else {
                acme_warn!("  ⚠️  Cloudflare配置: 未找到");
            }
        }
        Err(e) => {
            acme_error!(ConfigError, "配置文件加载失败");
            panic!("配置文件加载失败: {}", e);
        }
    }
}