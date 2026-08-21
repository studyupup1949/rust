//! ACME 授权模块
//! 处理域名授权验证相关功能

use crate::error::{AcmeError, AcmeResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

// 从challenge模块导入类型
use super::challenge::{Challenge, ChallengeStatus};

/// 授权状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AuthorizationStatus {
    /// 待处理
    Pending,
    /// 有效
    Valid,
    /// 无效
    Invalid,
    /// 已撤销
    Revoked,
    /// 已过期
    Expired,
}

/// 授权对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Authorization {
    /// 授权标识符
    pub identifier: Identifier,
    /// 授权状态
    pub status: AuthorizationStatus,
    /// 过期时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<String>,
    /// 挑战列表
    pub challenges: Vec<Challenge>,
    /// 是否为通配符授权
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wildcard: Option<bool>,
}

/// 标识符类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum IdentifierType {
    /// DNS 域名
    Dns,
    /// IP 地址
    Ip,
}

/// 标识符
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identifier {
    /// 标识符类型
    #[serde(rename = "type")]
    pub identifier_type: IdentifierType,
    /// 标识符值（域名或IP）
    pub value: String,
}



impl Authorization {
    /// 创建新的授权对象
    pub fn new(domain: &str) -> Self {
        Self {
            identifier: Identifier {
                identifier_type: IdentifierType::Dns,
                value: domain.to_string(),
            },
            status: AuthorizationStatus::Pending,
            expires: None,
            challenges: Vec::new(),
            wildcard: None,
        }
    }
    
    /// 获取域名
    pub fn domain(&self) -> &str {
        &self.identifier.value
    }
    
    /// 检查授权是否有效
    pub fn is_valid(&self) -> bool {
        self.status == AuthorizationStatus::Valid
    }
    
    /// 检查授权是否待处理
    pub fn is_pending(&self) -> bool {
        self.status == AuthorizationStatus::Pending
    }
    
    /// 获取指定类型的挑战
    pub fn get_challenge(&self, challenge_type: super::challenge::ChallengeType) -> Option<&Challenge> {
        self.challenges.iter()
            .find(|c| c.challenge_type == challenge_type)
    }
    
    /// 获取 DNS-01 挑战
    pub fn get_dns_challenge(&self) -> Option<&Challenge> {
        self.get_challenge(super::challenge::ChallengeType::Dns01)
    }
    
    /// 获取 HTTP-01 挑战
    pub fn get_http_challenge(&self) -> Option<&Challenge> {
        self.get_challenge(super::challenge::ChallengeType::Http01)
    }
    
    /// 更新授权状态
    pub fn update_status(&mut self, status: AuthorizationStatus) {
        self.status = status;
    }
    
    /// 添加挑战
    pub fn add_challenge(&mut self, challenge: Challenge) {
        self.challenges.push(challenge);
    }
}

/// 授权管理器
#[derive(Debug)]
pub struct AuthorizationManager {
    /// 授权缓存
    authorizations: HashMap<String, Authorization>,
}

impl AuthorizationManager {
    /// 创建新的授权管理器
    pub fn new() -> Self {
        Self {
            authorizations: HashMap::new(),
        }
    }
    
    /// 添加授权
    pub fn add_authorization(&mut self, domain: String, authorization: Authorization) {
        self.authorizations.insert(domain, authorization);
    }
    
    /// 获取授权
    pub fn get_authorization(&self, domain: &str) -> Option<&Authorization> {
        self.authorizations.get(domain)
    }
    
    /// 获取可变授权
    pub fn get_authorization_mut(&mut self, domain: &str) -> Option<&mut Authorization> {
        self.authorizations.get_mut(domain)
    }
    
    /// 移除授权
    pub fn remove_authorization(&mut self, domain: &str) -> Option<Authorization> {
        self.authorizations.remove(domain)
    }
    
    /// 检查域名是否已授权
    pub fn is_authorized(&self, domain: &str) -> bool {
        self.authorizations.get(domain)
            .map(|auth| auth.is_valid())
            .unwrap_or(false)
    }
    
    /// 获取所有授权
    pub fn get_all_authorizations(&self) -> &HashMap<String, Authorization> {
        &self.authorizations
    }
    
    /// 清理过期授权
    pub fn cleanup_expired(&mut self) {
        let now = chrono::Utc::now();
        self.authorizations.retain(|_, auth| {
            if let Some(expires) = &auth.expires {
                if let Ok(expire_time) = chrono::DateTime::parse_from_rfc3339(expires) {
                    return expire_time.with_timezone(&chrono::Utc) > now;
                }
            }
            true // 如果没有过期时间或解析失败，保留授权
        });
    }
}

impl Default for AuthorizationManager {
    fn default() -> Self {
        Self::new()
    }
}
