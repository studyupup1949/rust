//! 沙盒模式 DNS 验证测试
//! 使用 Let's Encrypt 测试环境进行 DNS 验证测试

mod test_common;
use test_common::*;
use acme_commander::dns::cloudflare::CloudflareDnsManager;
use acme_commander::dns::{DnsManager, DnsChallengeManager};
use acme_commander::acme::{AcmeClient, AcmeConfig};


#[tokio::test]
async fn test_sandbox_dns_validation() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logger();

    println!("开始沙盒模式 DNS 验证测试，域名: {}", TEST_DOMAIN);

    // 创建 Cloudflare DNS 管理器
    let dns_manager = create_test_cloudflare_manager().await?;
    println!("✅ 成功创建 Cloudflare DNS 管理器");
    println!("✅ Cloudflare API Token 验证成功");
    
    // 创建 DNS 挑战管理器
    let dns_challenge_manager = create_test_dns_challenge_manager(dns_manager);

    // 创建账户密钥
    let account_key = create_test_account_key();
    println!("✅ 成功生成账户密钥");

    // 创建 ACME 客户端配置 (使用 Let's Encrypt 测试环境)
    let acme_config = create_test_acme_config(Some("test@example.com".to_string()));

    // 创建 ACME 客户端
    let mut acme_client = create_test_acme_client(acme_config, account_key.clone())?;
    
    // 注册账户
    let _account = {
        let mut account_manager = acme_commander::acme::account::AccountManager::new(&mut acme_client);
        match account_manager.register_account(Some("test@example.com"), true, None).await {
            Ok(account) => {
                println!("✅ 成功注册 ACME 账户");
                account
            },
            Err(e) => {
                panic!("注册 ACME 账户失败: {}", e);
            }
        }
    };
    
    // 创建新订单
    let (order, _order_url) = {
        let mut order_manager = acme_commander::acme::order::OrderManager::new(&mut acme_client);
        match order_manager.create_order(&[TEST_DOMAIN.to_string()], None, None).await {
            Ok((order, order_url)) => {
                println!("✅ 成功创建订单，状态: {:?}", order.status);
                (order, order_url)
            },
            Err(e) => {
                panic!("创建订单失败: {}", e);
            }
        }
    };
    
    // 获取授权
    let authorizations = {
        let mut order_manager = acme_commander::acme::order::OrderManager::new(&mut acme_client);
        match order_manager.get_order_authorizations(&order).await {
            Ok(auths) => {
                println!("✅ 成功获取授权，数量: {}", auths.len());
                auths
            },
            Err(e) => {
                panic!("获取授权失败: {}", e);
            }
        }
    };
    
    // 处理每个授权
    for auth in &authorizations {
        println!("处理域名 {} 的授权", auth.identifier.value);
        
        // 查找 DNS-01 挑战
        let dns_challenge = match auth.challenges.iter()
            .find(|c| c.challenge_type == acme_commander::acme::ChallengeType::Dns01) {
            Some(challenge) => {
                println!("✅ 找到 DNS-01 挑战");
                challenge
            },
            None => {
                panic!("未找到 DNS-01 挑战");
            }
        };
        
        // 获取 DNS 挑战值
        let challenge_info = {
            let mut challenge_manager = acme_commander::acme::challenge::ChallengeManager::new(&mut acme_client);
            match challenge_manager.prepare_challenge(dns_challenge) {
                Ok(info) => {
                    println!("✅ 成功准备挑战");
                    info
                },
                Err(e) => {
                    panic!("无法准备挑战: {}", e);
                }
            }
        };
        
        let dns_value = if let acme_commander::acme::challenge::ChallengeInfo::Dns01(dns01) = challenge_info {
            dns01.record_value
        } else {
            panic!("挑战类型不匹配，期望 DNS-01 挑战");
        };
        println!("✅ 成功获取 DNS-01 值: {}", dns_value);
        
        // 添加 DNS 记录
        let challenge_record = match dns_challenge_manager.add_challenge_record(
            &auth.identifier.value,
            &dns_value,
            false, // 不使用 dry-run 模式
        ).await {
            Ok(record) => {
                println!("✅ 成功添加 DNS 记录: {}", record.record_name);
                record
            },
            Err(e) => {
                panic!("添加 DNS 挑战记录失败: {}", e);
            }
        };
        
        // 等待 DNS 传播
        match dns_challenge_manager.wait_for_propagation(
            &challenge_record,
            false, // 不使用 dry-run 模式
        ).await {
            Ok(result) => {
                println!("✅ DNS 传播成功: 已传播 = {}", result.propagated);
                println!("   成功的服务器: {:?}", result.successful_servers);
            },
            Err(e) => {
                println!("⚠️ 等待 DNS 传播失败: {}", e);
                println!("   继续测试，但可能会导致验证失败");
            }
        };
        
        // 通知 ACME 服务器验证挑战
        {
            let mut challenge_manager = acme_commander::acme::challenge::ChallengeManager::new(&mut acme_client);
            match challenge_manager.respond_to_challenge(&dns_challenge).await {
                Ok(result) => {
                    println!("✅ 挑战验证结果: {:?}", result.status);
                },
                Err(e) => {
                    println!("⚠️ 验证挑战失败: {}", e);
                }
            }
        };
    }
    
    // 轮询订单状态
    let updated_order = {
        let mut order_manager = acme_commander::acme::order::OrderManager::new(&mut acme_client);
        match order_manager.get_order(&_order_url).await {
            Ok(order) => {
                println!("✅ 成功轮询订单，状态: {:?}", order.status);
                order
            },
            Err(e) => {
                println!("⚠️ 轮询订单失败: {}", e);
                println!("   继续测试，但可能无法完成证书签发");
                order
            }
        }
    };
    
    // 清理 DNS 记录
    for auth in &authorizations {
        match dns_challenge_manager.cleanup_challenge_records(
            &auth.identifier.value,
            false, // 不使用 dry-run 模式
        ).await {
            Ok(results) => {
                println!("✅ 成功清理 {} 条 DNS 记录", results.len());
                for (i, result) in results.iter().enumerate() {
                    println!("  {}. 成功: {}", i + 1, result.success);
                    if let Some(id) = &result.record_id {
                        println!("     记录 ID: {}", id);
                    }
                }
            },
            Err(e) => {
                println!("⚠️ 清理 DNS 挑战记录失败: {}", e);
            }
        };
    }
    
    println!("✅ 沙盒模式 DNS 验证测试完成");
    Ok(())
}