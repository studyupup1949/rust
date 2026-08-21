//! 命令行接口定义模块
//!
//! 包含所有的 CLI 参数定义、子命令和相关的枚举类型

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// ACME Commander - SSL/TLS 证书管理工具
#[derive(Parser)]
#[command(name = "acme-commander")]
#[command(version = acme_commander::VERSION)]
#[command(about = "用于SSL/TLS证书管理的综合ACME客户端")]
#[command(long_about = "ACME Commander是一个现代化的ACME客户端，支持从Let's Encrypt和ZeroSSL等兼容ACME的证书颁发机构自动颁发和续期SSL/TLS证书。")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
    
    /// 启用详细日志（调试级别）
    #[arg(short, long, global = true)]
    pub verbose: bool,
    
    /// 日志输出格式
    #[arg(long, global = true, value_enum, default_value = "terminal")]
    pub log_output: LogOutputType,
    
    /// 日志文件路径（使用文件输出时）
    #[arg(long, global = true)]
    pub log_file: Option<PathBuf>,
    
    /// 配置文件路径
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum LogOutputType {
    Terminal,
    File,
    Both,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 获取新证书
    Certonly {
        /// 要包含在证书中的域名（如果配置文件中未提供则必需）
        #[arg(long)]
        domains: Option<Vec<String>>,

        /// 账户注册邮箱地址（如果配置文件中未提供则必需）
        #[arg(short, long)]
        email: Option<String>,
        
        /// 使用生产环境（默认：测试环境）
        #[arg(long)]
        production: bool,
        
        /// 执行 dry run 而不做任何更改
        #[arg(long)]
        dry_run: bool,
        
        /// 用于挑战验证的 DNS 提供商
        #[arg(long, value_enum, default_value = "cloudflare")]
        dns_provider: DnsProviderType,
        
        /// Cloudflare API 令牌（Cloudflare DNS 必需）
        #[arg(long, env = "CLOUDFLARE_API_TOKEN")]
        cloudflare_token: Option<String>,
        
        /// 账户密钥文件路径
        #[arg(long)]
        account_key: Option<PathBuf>,
        
        /// 证书密钥文件路径
        #[arg(long)]
        cert_key: Option<PathBuf>,
        
        /// 证书文件输出目录
        #[arg(short, long, default_value = "./certs")]
        output_dir: PathBuf,
        
        /// 证书文件名前缀
        #[arg(long, default_value = "cert")]
        cert_name: String,
        
        /// 强制证书续订，即使未接近过期
        #[arg(long)]
        force_renewal: bool,
    },
    
    /// 续订现有证书
    Renew {
        /// 要扫描续订的证书目录
        #[arg(short, long, default_value = "./certs")]
        cert_dir: PathBuf,
        
        /// 强制续订所有证书
        #[arg(long)]
        force: bool,
        
        /// 执行 dry run 而不做任何更改
        #[arg(long)]
        dry_run: bool,
    },
    
    /// 恢复中断的挑战流程
    Recover {
        /// 域名目录路径（包含保存的状态文件）
        #[arg(short, long, required = true)]
        domain_dir: PathBuf,
        
        /// 执行 dry run 而不做任何更改
        #[arg(long)]
        dry_run: bool,
    },
    
    /// 验证 API 令牌凭证
    Validate {
        /// Cloudflare API 令牌
        #[arg(long, env = "CLOUDFLARE_API_TOKEN")]
        cloudflare_token: Option<String>,
        
        /// ZeroSSL API 密钥
        #[arg(long, env = "ZEROSSL_API_KEY")]
        zerossl_api_key: Option<String>,
    },
    
    /// 生成加密密钥
    Keygen {
        /// 私钥输出文件
        #[arg(short, long, required = true)]
        output: PathBuf,
        
        /// 密钥类型（账户或证书）
        #[arg(long, value_enum, default_value = "certificate")]
        key_type: KeyType,
    },
    
    /// 显示证书信息
    Show {
        /// 证书文件路径
        #[arg(required = true)]
        cert_file: PathBuf,
        
        /// 显示详细证书信息
        #[arg(long)]
        detailed: bool,
    },
    
    /// 撤销证书
    Revoke {
        /// 证书文件路径
        #[arg(required = true)]
        cert_file: PathBuf,
        
        /// 账户密钥文件路径
        #[arg(long, required = true)]
        account_key: PathBuf,
        
        /// 撤销原因
        #[arg(long, value_enum, default_value = "unspecified")]
        reason: RevocationReason,
        
        /// 使用生产环境
        #[arg(long)]
        production: bool,
    },
    
    /// DNS 管理命令
    Dns {
        #[command(subcommand)]
        command: DnsCommands,
    },
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum DnsProviderType {
    Cloudflare,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum KeyType {
    Account,
    Certificate,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum RevocationReason {
    Unspecified,
    KeyCompromise,
    CaCompromise,
    AffiliationChanged,
    Superseded,
    CessationOfOperation,
    CertificateHold,
    RemoveFromCrl,
    PrivilegeWithdrawn,
    AaCompromise,
}

#[derive(Subcommand)]
pub enum DnsCommands {
    /// 清理域名的 ACME 挑战记录
    Cleanup {
        /// 要清理的域名
        #[arg(required = true)]
        domain: String,
        
        /// DNS 提供商
        #[arg(long, value_enum, default_value = "cloudflare")]
        dns_provider: DnsProviderType,
        
        /// Cloudflare API 令牌
        #[arg(long, env = "CLOUDFLARE_API_TOKEN")]
        cloudflare_token: Option<String>,
        
        /// 执行 dry run 而不做任何更改
        #[arg(long)]
        dry_run: bool,
    },
    
    /// 列出域名的 ACME 挑战记录
    List {
        /// 要查询的域名
        #[arg(required = true)]
        domain: String,
        
        /// DNS 提供商
        #[arg(long, value_enum, default_value = "cloudflare")]
        dns_provider: DnsProviderType,
        
        /// Cloudflare API 令牌
        #[arg(long, env = "CLOUDFLARE_API_TOKEN")]
        cloudflare_token: Option<String>,
    },
}