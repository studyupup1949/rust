//! ACME Commander 辅助工具函数
//!
//! 包含日志初始化、配置加载、版本信息显示等辅助功能

use std::path::PathBuf;
use std::error::Error;
use acme_commander::error::AcmeError;
use acme_commander::i18n;
use acme_commander::i18n_logger;
use acme_commander::logger::{LogConfig, LogLevel, LogOutput, init_logger};
use rat_logger::info;
use crate::cli::{LogOutputType, DnsProviderType};
/// 初始化日志系统
pub fn init_logging(
    verbose: bool,
    log_output: LogOutputType,
    log_file: Option<PathBuf>,
) -> Result<(), Box<dyn Error>> {
    let log_level = if verbose {
        LogLevel::Debug
    } else {
        LogLevel::Info  // 默认使用 Info 级别显示重要信息
    };
    
    let log_output = match log_output {
        LogOutputType::Terminal => LogOutput::Terminal,
        LogOutputType::File => {
            let log_file = log_file
                .ok_or_else(|| "文件输出需要日志文件路径")?;
            LogOutput::File {
                log_dir: log_file.parent().unwrap_or_else(|| std::path::Path::new(".")).to_path_buf(),
                max_file_size: 10 * 1024 * 1024, // 10MB
                max_compressed_files: 5,
            }
        },
        LogOutputType::Both => {
            let log_file = log_file
                .ok_or_else(|| "双重输出需要日志文件路径")?;
            LogOutput::File {
                log_dir: log_file.parent().unwrap_or_else(|| std::path::Path::new(".")).to_path_buf(),
                max_file_size: 10 * 1024 * 1024, // 10MB
                max_compressed_files: 5,
            }
        },
    };
    
    let config = LogConfig {
        level: log_level,
        output: log_output,
        enabled: true,
        use_colors: true,
        use_emoji: true,
        show_timestamp: true,
        show_module: verbose,
        enable_async: false, // CLI工具使用同步模式
        batch_size: 2048,
        batch_interval_ms: 25,
        buffer_size: 16 * 1024,
    };

    init_logger(config)
        .map_err(|e| format!("初始化日志器失败: {}", e).into())
}

/// 加载配置文件
pub fn load_app_config(config_path: Option<PathBuf>) -> Result<acme_commander::config::AcmeConfig, Box<dyn std::error::Error>> {
    use acme_commander::config::load_config;

    let config = load_config(config_path, None)?;
    Ok(config)
}

/// 显示版本信息
pub fn show_version_info() {
    use rat_logger::info;

    i18n_logger::log_info_format("log.version_info", &[&format!("ACME Commander v{}", env!("CARGO_PKG_VERSION"))]);
    info!("构建信息:");
    if let Some(git_hash) = option_env!("GIT_HASH") {
        i18n_logger::log_info_format("log.git_commit", &[git_hash]);
    }
    if let Some(build_time) = option_env!("BUILD_TIME") {
        i18n_logger::log_info_format("log.build_time", &[build_time]);
    }
    i18n_logger::log_info_format("log.target_platform", &[std::env::consts::ARCH]);
}



/// 格式化错误信息
pub fn format_error(error: &dyn std::error::Error) -> String {
    // 对于其他类型的错误，使用默认格式化
    let mut message = error.to_string();
    let mut source = error.source();

    while let Some(err) = source {
        message.push_str(&format!("\n  {}: {}", i18n::t("error.reason"), err));
        source = err.source();
    }

    message
}

/// 安全地显示令牌（隐藏敏感部分）
pub fn mask_token(token: &str) -> String {
    if token.len() <= 8 {
        "***".to_string()
    } else {
        format!("{}***{}", &token[..4], &token[token.len()-4..])
    }
}

/// 合并后的certonly配置
#[derive(Debug, Clone)]
pub struct CertonlyConfig {
    pub domains: Vec<String>,
    pub email: String,
    pub production: bool,
    pub dry_run: bool,
    pub dns_provider: DnsProviderType,
    pub cloudflare_token: Option<String>,
    pub account_key: Option<std::path::PathBuf>,
    pub cert_key: Option<std::path::PathBuf>,
    pub output_dir: std::path::PathBuf,
    pub cert_name: String,
    pub force_renewal: bool,
}

/// 合并配置文件和命令行参数
pub fn merge_config_with_cli_args(
    config_file: Option<std::path::PathBuf>,
    cli_domains: Option<Vec<String>>,
    cli_email: Option<String>,
    cli_production: bool,
    cli_dry_run: bool,
    cli_dns_provider: DnsProviderType,
    cli_cloudflare_token: Option<String>,
    cli_account_key: Option<std::path::PathBuf>,
    cli_cert_key: Option<std::path::PathBuf>,
    cli_output_dir: std::path::PathBuf,
    cli_cert_name: String,
    cli_force_renewal: bool,
) -> Result<CertonlyConfig, Box<dyn std::error::Error>> {
    use acme_commander::config::{load_config, get_cloudflare_token};

    // 尝试从配置文件加载
    let app_config = if config_file.is_some() {
        match load_config(config_file.clone(), None) {
            Ok(config) => Some(config),
            Err(e) => {
                rat_logger::warn!("配置文件加载失败: {}", e);
                None
            }
        }
    } else {
        None
    };

    // 获取域名（命令行优先）
    let domains = if let Some(domains) = cli_domains {
        if domains.is_empty() {
            // 如果命令行为空，尝试从配置文件获取
            app_config
                .as_ref()
                .map(|config| config.certificate.domains.clone())
                .unwrap_or_default()
        } else {
            domains
        }
    } else {
        // 没有命令行参数，从配置文件获取
        app_config
            .as_ref()
            .map(|config| config.certificate.domains.clone())
            .unwrap_or_default()
    };

    // 获取邮箱（命令行优先）
    let email = if let Some(email) = cli_email {
        email
    } else {
        app_config
            .as_ref()
            .map(|config| config.account.email.clone())
            .unwrap_or_default()
    };

    // 获取Cloudflare Token（优先级：命令行 > 环境变量 > 配置文件）
    let cloudflare_token = if let Some(token) = cli_cloudflare_token {
        Some(token)
    } else {
        get_cloudflare_token(config_file.clone())
    };

    // 获取生产环境设置（配置文件优先，然后命令行）
    let production = if let Some(config) = app_config.as_ref() {
        config.acme.environment == acme_commander::config::AcmeEnvironment::Production
    } else {
        cli_production
    };

    // 构建最终配置
    let config = CertonlyConfig {
        domains,
        email,
        production,
        dry_run: cli_dry_run,
        dns_provider: cli_dns_provider,
        cloudflare_token,
        account_key: cli_account_key,
        cert_key: cli_cert_key,
        output_dir: cli_output_dir,
        cert_name: cli_cert_name,
        force_renewal: cli_force_renewal,
    };

    Ok(config)
}

/// 验证certonly配置
pub fn validate_certonly_config(config: &CertonlyConfig) -> Result<(), Box<dyn std::error::Error>> {
    rat_logger::debug!("开始验证配置:");
    rat_logger::debug!("  域名: {:?}", config.domains);
    rat_logger::debug!("  邮箱: {}", config.email);
    rat_logger::debug!("  DNS提供商: {:?}", config.dns_provider);
    rat_logger::debug!("  Cloudflare Token: {:?}", config.cloudflare_token);

    // 验证域名
    if config.domains.is_empty() {
        rat_logger::error!("域名验证失败: 域名列表为空");
        return Err("必须提供至少一个域名。可以通过 --domains 参数或配置文件中的 domains 字段指定。".into());
    }

    // 验证邮箱
    if config.email.is_empty() || !config.email.contains('@') {
        rat_logger::error!("邮箱验证失败: 邮箱为空或格式无效");
        return Err("必须提供有效的邮箱地址。可以通过 --email 参数或配置文件中的 account.email 字段指定。".into());
    }

    // 验证Cloudflare Token（仅当使用Cloudflare时）
    if config.dns_provider == DnsProviderType::Cloudflare {
        if config.cloudflare_token.is_none() || config.cloudflare_token.as_ref().unwrap().is_empty() {
            rat_logger::error!("Cloudflare Token验证失败: Token为空");
            return Err("使用Cloudflare DNS时必须提供API Token。可以通过 --cloudflare-token 参数、CLOUDFLARE_API_TOKEN 环境变量或配置文件中的 dns.cloudflare.api_token 字段指定。".into());
        }
    }

    rat_logger::debug!("配置验证成功");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_token() {
        assert_eq!(mask_token("short"), "***");
        assert_eq!(mask_token("verylongtoken123456"), "very***3456");
        assert_eq!(mask_token("12345678"), "***");
        assert_eq!(mask_token("123456789"), "1234***6789");
    }

    #[test]
    fn test_format_error() {
        let error = std::io::Error::new(std::io::ErrorKind::NotFound, "文件未找到");
        let formatted = format_error(&error);
        assert!(formatted.contains("文件未找到"));
    }
}