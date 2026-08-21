//! DNS 记录清理工具
//! 用于清理残留的 ACME 挑战 DNS 记录

use acme_commander::dns::cloudflare::CloudflareDnsManager;
use acme_commander::dns::DnsManager;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("用法: {} <cloudflare_token> <domain>", args[0]);
        std::process::exit(1);
    }
    
    let token = &args[1];
    let domain = &args[2];
    
    println!("🧹 开始清理域名 {} 的 ACME 挑战记录...", domain);
    
    // 创建 Cloudflare DNS 管理器
    let dns_manager = CloudflareDnsManager::new(token.to_string())?;
    
    // 查找并删除所有 _acme-challenge 记录
    let challenge_name = format!("_acme-challenge.{}", domain);
    
    println!("🔍 查找记录: {}", challenge_name);
    
    // 获取 zone ID
    let zone_id = dns_manager.get_zone_id(domain).await?;
    println!("📍 Zone ID: {}", zone_id);
    
    // 列出所有 TXT 记录
    let records = dns_manager.list_dns_records(&zone_id, "TXT", Some(&challenge_name)).await?;
    
    if records.is_empty() {
        println!("✅ 没有找到需要清理的记录");
        return Ok(());
    }
    
    println!("📋 找到 {} 条记录需要清理:", records.len());
    for record in &records {
        println!("  - ID: {}, 名称: {}, 值: {}", record.id, record.name, record.content);
    }
    
    // 删除所有找到的记录
    for record in records {
        println!("🗑️  删除记录: {} ({})", record.name, record.id);
        dns_manager.delete_dns_record(&zone_id, &record.id).await?;
        println!("✅ 记录已删除");
    }
    
    println!("🎉 清理完成！");
    Ok(())
}