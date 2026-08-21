//! ACME 目录模块
//! 处理 ACME 服务器目录信息和端点发现

use crate::error::{AcmeError, AcmeResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

/// ACME 目录结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Directory {
    /// 新账户端点
    #[serde(rename = "newAccount")]
    pub new_account: String,
    /// 新订单端点
    #[serde(rename = "newOrder")]
    pub new_order: String,
    /// 新随机数端点
    #[serde(rename = "newNonce")]
    pub new_nonce: String,
    /// 撤销证书端点
    #[serde(rename = "revokeCert")]
    pub revoke_cert: String,
    /// 密钥更改端点
    #[serde(rename = "keyChange", skip_serializing_if = "Option::is_none")]
    pub key_change: Option<String>,
    /// 元数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<DirectoryMeta>,
}

/// 目录元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryMeta {
    /// 服务条款 URL
    #[serde(rename = "termsOfService", skip_serializing_if = "Option::is_none")]
    pub terms_of_service: Option<String>,
    /// 网站 URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    /// CAA 身份
    #[serde(rename = "caaIdentities", skip_serializing_if = "Option::is_none")]
    pub caa_identities: Option<Vec<String>>,
    /// 是否需要外部账户绑定
    #[serde(rename = "externalAccountRequired", skip_serializing_if = "Option::is_none")]
    pub external_account_required: Option<bool>,
}

/// 知名的 ACME 服务器
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcmeServer {
    /// Let's Encrypt 生产环境
    LetsEncryptProd,
    /// Let's Encrypt 测试环境
    LetsEncryptStaging,
    /// ZeroSSL
    ZeroSSL,
    /// 自定义服务器
    Custom,
}

impl AcmeServer {
    /// 获取服务器的目录 URL
    pub fn directory_url(&self) -> &'static str {
        match self {
            AcmeServer::LetsEncryptProd => "https://acme-v02.api.letsencrypt.org/directory",
            AcmeServer::LetsEncryptStaging => "https://acme-staging-v02.api.letsencrypt.org/directory",
            AcmeServer::ZeroSSL => "https://acme.zerossl.com/v2/DV90/directory",
            AcmeServer::Custom => panic!("Custom server requires explicit URL"),
        }
    }
    
    /// 获取服务器名称
    pub fn name(&self) -> &'static str {
        match self {
            AcmeServer::LetsEncryptProd => "Let's Encrypt (Production)",
            AcmeServer::LetsEncryptStaging => "Let's Encrypt (Staging)",
            AcmeServer::ZeroSSL => "ZeroSSL",
            AcmeServer::Custom => "Custom",
        }
    }
    
    /// 从字符串解析服务器类型
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "letsencrypt" | "le" | "prod" | "production" => Some(AcmeServer::LetsEncryptProd),
            "staging" | "test" | "letsencrypt-staging" => Some(AcmeServer::LetsEncryptStaging),
            "zerossl" => Some(AcmeServer::ZeroSSL),
            _ => None,
        }
    }
}

/// 目录管理器
#[derive(Debug)]
pub struct DirectoryManager {
    /// 当前目录
    directory: Option<Directory>,
    /// 目录 URL
    directory_url: Option<String>,
    /// HTTP 客户端
    client: reqwest::Client,
}

impl DirectoryManager {
    /// 创建新的目录管理器
    pub fn new() -> Self {
        Self {
            directory: None,
            directory_url: None,
            client: reqwest::Client::new(),
        }
    }
    
    /// 使用指定的 HTTP 客户端创建目录管理器
    pub fn with_client(client: reqwest::Client) -> Self {
        Self {
            directory: None,
            directory_url: None,
            client,
        }
    }
    
    /// 从 ACME 服务器获取目录
    pub async fn fetch_directory(&mut self, server: AcmeServer) -> AcmeResult<&Directory> {
        let url = server.directory_url();
        self.fetch_directory_from_url(url).await
    }
    
    /// 从自定义 URL 获取目录
    pub async fn fetch_directory_from_url(&mut self, url: &str) -> AcmeResult<&Directory> {
        // 验证 URL 格式
        let parsed_url = Url::parse(url)
            .map_err(|e| AcmeError::InvalidDomain(format!("Invalid directory URL: {}", e)))?;
        
        // 发送 GET 请求获取目录
        let response = self.client
            .get(url)
            .header("User-Agent", "acme-commander/1.0")
            .send()
            .await
            .map_err(|e| AcmeError::HttpError(format!("Failed to fetch directory: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(AcmeError::HttpError(
                format!("Directory request failed with status: {}", response.status())
            ));
        }
        
        // 解析 JSON 响应
        let directory: Directory = response
            .json()
            .await
            .map_err(|e| AcmeError::JsonError(format!("Failed to parse directory JSON: {}", e)))?;
        
        // 验证目录结构
        self.validate_directory(&directory)?;
        
        self.directory = Some(directory);
        self.directory_url = Some(url.to_string());
        
        Ok(self.directory.as_ref().unwrap())
    }
    
    /// 获取当前目录
    pub fn get_directory(&self) -> Option<&Directory> {
        self.directory.as_ref()
    }
    
    /// 获取目录 URL
    pub fn get_directory_url(&self) -> Option<&str> {
        self.directory_url.as_deref()
    }
    
    /// 获取新账户端点
    pub fn new_account_url(&self) -> AcmeResult<&str> {
        self.directory.as_ref()
            .map(|d| d.new_account.as_str())
            .ok_or_else(|| AcmeError::ProtocolError("Directory not loaded".to_string()))
    }
    
    /// 获取新订单端点
    pub fn new_order_url(&self) -> AcmeResult<&str> {
        self.directory.as_ref()
            .map(|d| d.new_order.as_str())
            .ok_or_else(|| AcmeError::ProtocolError("Directory not loaded".to_string()))
    }
    
    /// 获取新随机数端点
    pub fn new_nonce_url(&self) -> AcmeResult<&str> {
        self.directory.as_ref()
            .map(|d| d.new_nonce.as_str())
            .ok_or_else(|| AcmeError::ProtocolError("Directory not loaded".to_string()))
    }
    
    /// 获取撤销证书端点
    pub fn revoke_cert_url(&self) -> AcmeResult<&str> {
        self.directory.as_ref()
            .map(|d| d.revoke_cert.as_str())
            .ok_or_else(|| AcmeError::ProtocolError("Directory not loaded".to_string()))
    }
    
    /// 获取密钥更改端点
    pub fn key_change_url(&self) -> AcmeResult<Option<&str>> {
        Ok(self.directory.as_ref()
            .and_then(|d| d.key_change.as_deref()))
    }
    
    /// 获取服务条款 URL
    pub fn terms_of_service_url(&self) -> Option<&str> {
        self.directory.as_ref()
            .and_then(|d| d.meta.as_ref())
            .and_then(|m| m.terms_of_service.as_deref())
    }
    
    /// 检查是否需要外部账户绑定
    pub fn requires_external_account(&self) -> bool {
        self.directory.as_ref()
            .and_then(|d| d.meta.as_ref())
            .and_then(|m| m.external_account_required)
            .unwrap_or(false)
    }
    
    /// 验证目录结构
    fn validate_directory(&self, directory: &Directory) -> AcmeResult<()> {
        // 验证必需的端点
        let required_endpoints = [
            ("newAccount", &directory.new_account),
            ("newOrder", &directory.new_order),
            ("newNonce", &directory.new_nonce),
            ("revokeCert", &directory.revoke_cert),
        ];
        
        for (name, url) in &required_endpoints {
            if url.is_empty() {
                return Err(AcmeError::ProtocolError(
                    format!("Directory missing required endpoint: {}", name)
                ));
            }
            
            // 验证 URL 格式
            Url::parse(url)
                .map_err(|e| AcmeError::ProtocolError(
                    format!("Invalid URL for {}: {}", name, e)
                ))?;
        }
        
        Ok(())
    }
    
    /// 刷新目录（重新获取）
    pub async fn refresh(&mut self) -> AcmeResult<&Directory> {
        if let Some(url) = self.directory_url.clone() {
            self.fetch_directory_from_url(&url).await
        } else {
            Err(AcmeError::ProtocolError("No directory URL set".to_string()))
        }
    }
}

impl Default for DirectoryManager {
    fn default() -> Self {
        Self::new()
    }
}
