//! 调用者初始化日志示例
//!
//! 这个示例展示了如何作为调用者来正确初始化acme_commander的日志系统
//! 然后使用acme_commander进行ACME证书管理操作

use acme_commander::{
    LogLevel, LogOutput, LogConfig, init_logger,
    logger::utils::Timer, logger::AuditEvent,
};
use rat_logger::{LoggerBuilder, LevelFilter, handler::term::TermConfig, FormatConfig, LevelStyle};

/// 初始化acme_commander日志系统的推荐方式
fn init_logging_system() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 初始化ACME Commander日志系统 ===");

    // 创建ACME Commander日志配置
    let log_config = LogConfig {
        enabled: true,
        level: LogLevel::Debug,
        output: LogOutput::Terminal,
        use_colors: true,
        use_emoji: true,
        show_timestamp: true,
        show_module: true,
        enable_async: false,
        batch_size: 2048,
        batch_interval_ms: 25,
        buffer_size: 16 * 1024,
    };

    // 使用acme_commander的日志初始化函数
    init_logger(log_config)?;

    println!("✅ ACME Commander日志系统初始化成功");
    Ok(())
}

/// 创建不同的日志配置示例
fn create_log_configurations() -> Vec<(&'static str, LogConfig)> {
    vec![
        ("终端日志（默认）", LogConfig {
            enabled: true,
            level: LogLevel::Info,
            output: LogOutput::Terminal,
            use_colors: true,
            use_emoji: true,
            show_timestamp: true,
            show_module: true,
            enable_async: false,
            batch_size: 2048,
            batch_interval_ms: 25,
            buffer_size: 16 * 1024,
        }),
        ("文件日志示例", LogConfig::file("./logs")),
        ("禁用日志示例", LogConfig::disabled()),
    ]
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // === 调用者负责初始化日志系统 ===
    println!("=== ACME Commander 调用者初始化日志示例 ===\n");

    // 1. 首先初始化ACME Commander日志系统（调用者责任）
    init_logging_system()?;

    // 2. 测试ACME Commander的集成日志宏
    println!("\n=== 测试ACME Commander集成日志宏 ===");

    // 这些是ACME Commander提供的日志宏，使用我们的日志系统
    acme_commander::acme_info!("这是一个ACME信息日志");
    acme_commander::acme_warn!("这是一个ACME警告日志");
    acme_commander::acme_log_error!("这是一个ACME错误日志");
    acme_commander::cert_info!("这是一个证书信息日志");
    acme_commander::dns_info!("这是一个DNS信息日志");

    // 3. 展示不同的日志配置
    println!("\n=== 展示不同的日志配置 ===");
    let configs = create_log_configurations();

    for (name, config) in configs {
        println!("\n--- 测试配置: {} ---", name);
        println!("配置详情: {:?}", config);
        // 注意：这里只是展示配置，不实际切换日志系统
        // 在实际应用中，你会在启动时选择一个配置
    }

    // 4. 模拟ACME操作（使用ACME Commander的日志系统）
    println!("\n=== 模拟ACME操作 ===");

    acme_commander::acme_info!("🔧 开始模拟ACME操作流程");

    // 模拟密钥生成
    acme_commander::acme_info!("🔑 生成ECDSA P-384密钥对...");
    let timer = Timer::start("密钥对生成".to_string());
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let metrics = timer.finish(true);
    acme_commander::acme_info!("✅ 密钥对生成完成");

    // 模拟账户注册
    acme_commander::acme_info!("👤 注册ACME账户...");
    acme_commander::acme_info!("📧 联系邮箱: test@example.com");
    acme_commander::acme_info!("📋 同意服务条款");
    acme_commander::acme_info!("✅ 账户注册成功");

    // 模拟域名验证
    acme_commander::acme_info!("🌐 开始域名验证流程");
    acme_commander::acme_info!("📋 域名列表: example.com, www.example.com");
    acme_commander::acme_info!("🔍 选择DNS-01挑战验证");
    acme_commander::acme_info!("⏳ 等待DNS传播...");
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    acme_commander::acme_info!("✅ DNS验证成功");

    // 模拟证书签发
    acme_commander::acme_info!("📜 生成证书签名请求(CSR)...");
    acme_commander::acme_info!("🔐 提交CSR到ACME服务器");
    acme_commander::acme_info!("⏳ 等待证书签发...");
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    acme_commander::acme_info!("✅ 证书签发成功");

    acme_commander::acme_info!("🎉 ACME操作流程完成");

    // 5. 性能监控示例
    println!("\n=== 性能监控示例 ===");

    let timer = Timer::start("ACME完整流程".to_string());
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    let metrics = timer.finish(true);

    println!("性能指标记录完成");

    // 6. 审计日志示例
    println!("\n=== 审计日志示例 ===");

    let audit_event = AuditEvent::new(
        "证书申请".to_string(),
        "example.com".to_string(),
        "issue_certificate".to_string(),
        "成功".to_string(),
    ).with_user_id("test_user".to_string());

    audit_event.log();
    println!("审计事件记录完成");

    // 7. 条件性日志示例
    println!("\n=== 条件性日志示例 ===");

    println!("测试条件性日志（这些日志只有在日志系统初始化时才会显示）:");

    acme_commander::acme_info!("模拟证书保存到: /etc/ssl/certs/example.com.pem");
    acme_commander::cert_info!("证书有效期: 90天");
    acme_commander::dns_info!("DNS记录: _acme-challenge.example.com TXT = \"xxxx\"");

    // 8. 展示日志系统的安全性
    println!("\n=== 日志系统安全性演示 ===");
    println!("1. 如果调用者不初始化日志系统，所有日志宏会静默失败");
    println!("2. 不会因为日志未初始化而导致程序崩溃");
    println!("3. 适合作为库使用，不会干扰调用者的日志策略");
    println!("4. 调用者完全控制日志的格式、级别和输出目标");

    // 9. 不初始化日志系统的行为说明
    println!("\n=== 不初始化日志系统的行为 ===");
    println!("如果调用者不调用init_logger()，那么：");
    println!("• 所有acme_commander::acme_info!()等日志宏会静默失败");
    println!("• 不会产生任何输出，也不会导致程序崩溃");
    println!("• 核心功能完全正常工作，只是没有日志输出");
    println!("• 性能监控的Timer仍然可以工作，只是不会记录日志");
    println!("• 这种设计让调用者有完全的控制权");

    // 10. 最佳实践总结
    println!("\n=== 最佳实践总结 ===");
    println!("1. 作为库的ACME Commander不会自动初始化日志");
    println!("2. 调用者应该在main函数早期初始化日志系统");
    println!("3. 可以自定义日志格式、级别和输出目标");
    println!("4. 使用acme_commander::init_logger()函数进行初始化");
    println!("5. 在生产环境中可以配置文件日志或网络日志");
    println!("6. 可以启用异步模式以提高性能");

    println!("\n=== 示例完成 ===");
    println!("这个示例展示了：");
    println!("1. 调用者如何初始化ACME Commander的日志系统");
    println!("2. ACME Commander的集成日志宏使用");
    println!("3. 日志系统完全由调用者控制");
    println!("4. 条件性日志的安全性设计");
    println!("5. 性能监控和审计日志的使用");

    Ok(())
}