use admin_config::AppConfig;
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(name = "admin-config")]
#[command(about = "配置管理工具", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 初始化配置文件
    Init {
        /// 配置文件路径
        #[arg(short, long, default_value = "config.toml")]
        path: String,

        /// 强制覆盖已存在的配置文件
        #[arg(short, long)]
        force: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path, force } => {
            init_config(&path, force)?;
        }
    }

    Ok(())
}

fn init_config(path: &str, force: bool) -> Result<()> {
    let config_path = Path::new(path);

    if config_path.exists() && !force {
        anyhow::bail!("配置文件已存在: {}\n使用 --force 参数强制覆盖", path);
    }

    let default_config = AppConfig::default();
    let config_content = toml::to_string_pretty(&default_config)?;

    fs::write(config_path, config_content)?;

    println!("✓ 配置文件已创建: {}", path);
    println!("\n⚠️  安全提示：");
    println!("   1. 系统已自动生成以下安全密钥：");
    println!("      - JWT Token 密钥 (128位十六进制)");
    println!("      - Session 密钥 (128位十六进制)");
    println!("      - AES 加密密钥 (64位十六进制)");
    println!("      - AES IV (32位十六进制)");
    println!("      - API 密钥加密密钥 (64位十六进制)");
    println!("      - 密码盐值 (32位十六进制)");
    println!("   2. 请妥善保管配置文件，不要泄露密钥");
    println!("   3. 修改配置文件中的其他参数（数据库、Redis等）");
    println!("   4. 不要将包含真实密钥的配置文件提交到 Git 仓库");
    println!("   5. 生产环境建议使用环境变量或密钥管理服务");

    Ok(())
}
