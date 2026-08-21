//! Cloudflare DNS 测试
//! 测试 acme_commander 库的 Cloudflare DNS 管理功能

mod test_common;
use test_common::*;
use acme_commander::dns::cloudflare::CloudflareDnsManager;
use acme_commander::dns::{DnsManager, DnsChallengeManager};

#[tokio::test]
async fn test_cloudflare_dns_operations() {
    init_test_logger();

    let domain = TEST_DOMAIN;

    // 创建第一个Cloudflare DNS管理器用于验证
    let dns_manager = match create_test_cloudflare_manager().await {
        Ok(manager) => manager,
        Err(e) => {
            println!("警告: 无法创建Cloudflare DNS管理器: {}", e);
            println!("跳过Cloudflare DNS测试");
            return;
        }
    };

    println!("✅ Cloudflare API Token 验证成功");

    // 创建第二个Cloudflare DNS管理器用于测试
    let test_dns_manager = match create_test_cloudflare_manager().await {
        Ok(manager) => manager,
        Err(_) => {
            println!("无法创建第二个DNS管理器，跳过测试");
            return;
        }
    };

    // 创建测试用的DNS挑战管理器
    let dns_challenge_manager = create_test_dns_challenge_manager(test_dns_manager);
    
    // 测试挑战值
    let challenge_value = "test-challenge-value";
    
    // 添加 DNS 记录
    let challenge_record = dns_challenge_manager.add_challenge_record(
        domain,
        challenge_value,
        false, // 不使用 dry-run 模式
    )
    .await
    .expect("添加 DNS 挑战记录失败");
    
    println!("✅ DNS 记录已添加: {}", challenge_record.record_name);
    
    // 验证记录是否存在
    let record = dns_manager.find_txt_record(
        domain,
        &format!("_acme-challenge.{}", domain),
    )
    .await
    .expect("查找 TXT 记录失败");
    
    assert!(record.is_some(), "未找到添加的 TXT 记录");
    if let Some(record) = record {
        assert_eq!(record.value, challenge_value, "TXT 记录值不匹配");
        println!("✅ 成功验证 TXT 记录: {} = {}", record.name, record.value);
    }
    
    // 列出所有挑战记录
    let records = dns_challenge_manager.list_challenge_records(domain)
        .await
        .expect("列出挑战记录失败");
    
    println!("✅ 找到 {} 条挑战记录", records.len());
    for (i, record) in records.iter().enumerate() {
        println!("  {}. ID: {:?}", i + 1, record.id);
        println!("     名称: {}", record.name);
        println!("     值: {}", record.value);
    }
    
    // 等待 DNS 传播
    let propagation_result = dns_challenge_manager.wait_for_propagation(
        &challenge_record,
        false, // 不使用 dry-run 模式
    )
    .await
    .expect("等待 DNS 传播失败");
    
    println!("✅ DNS 传播结果: 已传播 = {}", propagation_result.propagated);
    println!("   检查的服务器: {:?}", propagation_result.checked_servers);
    println!("   成功的服务器: {:?}", propagation_result.successful_servers);
    
    // 清理 DNS 记录
    let cleanup_results = dns_challenge_manager.cleanup_challenge_records(
        domain,
        false, // 不使用 dry-run 模式
    )
    .await
    .expect("清理 DNS 挑战记录失败");
    
    println!("✅ 已清理 {} 条 DNS 记录", cleanup_results.len());
    for (i, result) in cleanup_results.iter().enumerate() {
        println!("  {}. 成功: {}", i + 1, result.success);
        if let Some(id) = &result.record_id {
            println!("     记录 ID: {}", id);
        }
        if let Some(error) = &result.error_message {
            println!("     错误: {}", error);
        }
    }
    
    // 验证记录是否已删除
    let record_after_cleanup = dns_manager.find_txt_record(
        domain,
        &format!("_acme-challenge.{}", domain),
    )
    .await
    .expect("查找 TXT 记录失败");
    
    assert!(record_after_cleanup.is_none(), "TXT 记录未被清理");
    println!("✅ 成功验证 TXT 记录已被清理");
}