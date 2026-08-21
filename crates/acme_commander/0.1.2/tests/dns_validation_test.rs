//! DNS 验证测试
//! 测试 acme_commander 库的 DNS 验证功能

mod test_common;
use test_common::*;
use acme_commander::dns::cloudflare::CloudflareDnsManager;
use acme_commander::dns::{DnsManager, DnsChallengeManager};
use acme_commander::acme::{AcmeClient, AcmeConfig};

#[tokio::test]
async fn test_dns_validation_with_cloudflare() {
    init_test_logger();

    let domain = TEST_DOMAIN;

    // 创建账户密钥
    let account_key = create_test_account_key();

    // 创建证书密钥
    let certificate_key = create_test_certificate_key();

    // 创建测试用的Cloudflare DNS管理器
    let dns_manager = match create_test_cloudflare_manager().await {
        Ok(manager) => manager,
        Err(e) => {
            println!("警告: 无法创建Cloudflare DNS管理器: {}", e);
            println!("跳过依赖DNS的测试部分");
            return;
        }
    };

    // 创建 DNS 挑战管理器
    let dns_challenge_manager = create_test_dns_challenge_manager(dns_manager);

    // 创建 ACME 客户端配置 (使用 Let's Encrypt 测试环境)
    let acme_config = create_test_acme_config(Some("test@example.com".to_string()));
    
    // 创建 ACME 客户端
    let mut acme_client = create_test_acme_client(acme_config, account_key.clone())
        .expect("无法创建 ACME 客户端");
    
    // 注册账户 (如果尚未注册)
    let _account = {
        let mut account_manager = acme_commander::acme::account::AccountManager::new(&mut acme_client);
        account_manager.register_account(Some("test@example.com"), true, None).await.expect("账户注册失败")
    };
    
    // 创建新订单
    let (order, _order_url) = {
        let mut order_manager = acme_commander::acme::order::OrderManager::new(&mut acme_client);
        order_manager.create_order(&[domain.to_string()], None, None).await.expect("创建订单失败")
    };
    
    println!("订单创建成功: {:?}", order.status);
    
    // 获取授权
    let authorizations = {
        let mut order_manager = acme_commander::acme::order::OrderManager::new(&mut acme_client);
        order_manager.get_order_authorizations(&order).await.expect("获取授权失败")
    };
    
    println!("获取到 {} 个授权", authorizations.len());
    
    // 处理每个授权
    for auth in &authorizations {
        println!("处理域名 {} 的授权", auth.identifier.value);
        
        // 查找 DNS-01 挑战
        let dns_challenge = auth.challenges.iter()
            .find(|c| c.challenge_type == acme_commander::acme::ChallengeType::Dns01)
            .expect("未找到 DNS-01 挑战");
        
        // 获取 DNS 挑战值
        let challenge_info = {
            let mut challenge_manager = acme_commander::acme::challenge::ChallengeManager::new(&mut acme_client);
            challenge_manager.prepare_challenge(dns_challenge).expect("无法准备挑战")
        };
        
        let dns_value = if let acme_commander::acme::challenge::ChallengeInfo::Dns01(dns01) = challenge_info {
            dns01.record_value
        } else {
            panic!("挑战类型不匹配，期望 DNS-01 挑战");
        };
        
        println!("DNS 挑战值: {}", dns_value);
        
        // 添加 DNS 记录
        let challenge_record = dns_challenge_manager.add_challenge_record(
            &auth.identifier.value,
            &dns_value,
            false, // 不使用 dry-run 模式
        )
        .await
        .expect("添加 DNS 挑战记录失败");
        
        println!("DNS 记录已添加: {}", challenge_record.record_name);
        
        // 等待 DNS 传播
        let propagation_result = dns_challenge_manager.wait_for_propagation(
            &challenge_record,
            false, // 不使用 dry-run 模式
        )
        .await
        .expect("等待 DNS 传播失败");
        
        println!("DNS 传播结果: {:?}", propagation_result);
        
        // 通知 ACME 服务器验证挑战
        let challenge_result = {
            let mut challenge_manager = acme_commander::acme::challenge::ChallengeManager::new(&mut acme_client);
            challenge_manager.respond_to_challenge(&dns_challenge).await.expect("验证挑战失败")
        };
        
        println!("挑战验证结果: {:?}", challenge_result.status);
    }
    
    // 完成订单
    let updated_order = {
        let mut order_manager = acme_commander::acme::order::OrderManager::new(&mut acme_client);
        order_manager.get_order(&_order_url).await.expect("轮询订单失败")
    };
    
    println!("更新后的订单状态: {:?}", updated_order.status);
    
    // 如果订单已就绪，完成证书签发
    if updated_order.status == acme_commander::acme::OrderStatus::Ready {
        // 创建 CSR
        let cert_request = acme_commander::acme::certificate::create_domain_certificate_request(
            domain.to_string(),
            vec![],
        );
        let csr = {
            let cert_manager = acme_commander::acme::certificate::CertificateManager::new(certificate_key.clone());
            cert_manager.generate_csr(&cert_request).expect("生成 CSR 失败")
        };
        
        // 完成订单
        let finalized_order = {
            let mut order_manager = acme_commander::acme::order::OrderManager::new(&mut acme_client);
            order_manager.finalize_order(&updated_order, &csr).await.expect("完成订单失败")
        };
        
        println!("最终订单状态: {:?}", finalized_order.status);
        
        // 下载证书
        if finalized_order.status == acme_commander::acme::OrderStatus::Valid {
            let certificate = if let Some(cert_url) = &finalized_order.certificate {
                acme_client.download_certificate(cert_url).await.expect("下载证书失败")
            } else {
                panic!("证书URL不可用")
            };
            
            println!("证书已下载，长度: {}", certificate.len());
            
            // 可以将证书保存到文件
            // std::fs::write("certificate.pem", certificate).expect("保存证书失败");
        }
    }
    
    // 清理 DNS 记录
    for auth in &authorizations {
        let cleanup_results = dns_challenge_manager.cleanup_challenge_records(
            &auth.identifier.value,
            false, // 不使用 dry-run 模式
        )
        .await
        .expect("清理 DNS 挑战记录失败");
        
        println!("已清理 {} 条 DNS 记录", cleanup_results.len());
    }
}