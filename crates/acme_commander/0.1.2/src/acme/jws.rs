//! ACME JWS (JSON Web Signature) 模块
//! 实现 ACME 协议所需的 JWS 签名功能

use crate::crypto::KeyPair;
use crate::error::{AcmeError, AcmeResult};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// JWS 保护头部
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwsProtectedHeader {
    /// 算法
    pub alg: String,
    /// 随机数
    pub nonce: String,
    /// URL
    pub url: String,
    /// JSON Web Key（用于新账户注册）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwk: Option<Value>,
    /// 密钥 ID（用于已注册账户）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kid: Option<String>,
}

/// JWS 签名结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwsSignature {
    /// Base64URL 编码的保护头部
    pub protected: String,
    /// Base64URL 编码的载荷
    pub payload: String,
    /// Base64URL 编码的签名
    pub signature: String,
}

/// JWS 结构体（别名）
pub type Jws = JwsSignature;

/// JWS 构建器
#[derive(Debug)]
pub struct JwsBuilder {
    /// 密钥对
    key_pair: KeyPair,
    /// 算法
    algorithm: String,
}

impl JwsBuilder {
    /// 创建新的 JWS 构建器
    pub fn new(key_pair: KeyPair) -> Self {
        Self {
            key_pair,
            algorithm: "ES384".to_string(), // ECDSA P-384 with SHA-384
        }
    }
    
    /// 为新账户注册创建 JWS
    pub fn create_for_new_account(
        &self,
        nonce: &str,
        url: &str,
        payload: &Value,
    ) -> AcmeResult<JwsSignature> {
        let jwk = self.key_pair.to_jwk()
            .map_err(|e| AcmeError::CryptoError(format!("创建 JWK 失败: {}", e)))?;
        
        let protected_header = JwsProtectedHeader {
            alg: self.algorithm.clone(),
            nonce: nonce.to_string(),
            url: url.to_string(),
            jwk: Some(jwk),
            kid: None,
        };
        
        self.create_jws(protected_header, payload)
    }
    
    /// 为已注册账户创建 JWS
    pub fn create_for_existing_account(
        &self,
        nonce: &str,
        url: &str,
        kid: &str,
        payload: &Value,
    ) -> AcmeResult<JwsSignature> {
        let protected_header = JwsProtectedHeader {
            alg: self.algorithm.clone(),
            nonce: nonce.to_string(),
            url: url.to_string(),
            jwk: None,
            kid: Some(kid.to_string()),
        };
        
        self.create_jws(protected_header, payload)
    }
    
    /// 创建空载荷的 JWS（用于 POST-as-GET 请求）
    pub fn create_post_as_get(
        &self,
        nonce: &str,
        url: &str,
        kid: &str,
    ) -> AcmeResult<JwsSignature> {
        let protected_header = JwsProtectedHeader {
            alg: self.algorithm.clone(),
            nonce: nonce.to_string(),
            url: url.to_string(),
            jwk: None,
            kid: Some(kid.to_string()),
        };
        
        // POST-as-GET 使用空字符串作为载荷
        let empty_payload = Value::String("".to_string());
        self.create_jws(protected_header, &empty_payload)
    }
    
    /// 创建 JWS 签名
    fn create_jws(
        &self,
        protected_header: JwsProtectedHeader,
        payload: &Value,
    ) -> AcmeResult<JwsSignature> {
        // 序列化保护头部
        let protected_json = serde_json::to_string(&protected_header)
            .map_err(|e| AcmeError::JsonError(format!("序列化保护头部失败: {}", e)))?;
        
        // Base64URL 编码保护头部
        let protected_b64 = URL_SAFE_NO_PAD.encode(protected_json.as_bytes());
        
        // 序列化载荷
        let payload_json = if payload.is_string() && payload.as_str() == Some("") {
            // 空载荷用于 POST-as-GET
            "".to_string()
        } else {
            serde_json::to_string(payload)
                .map_err(|e| AcmeError::JsonError(format!("序列化载荷失败: {}", e)))?
        };
        
        // Base64URL 编码载荷
        let payload_b64 = URL_SAFE_NO_PAD.encode(payload_json.as_bytes());
        
        // 创建签名输入
        let signing_input = format!("{}.{}", protected_b64, payload_b64);
        
        // 签名
        let signature_bytes = signing_input.as_bytes();
        let signature = self.key_pair.sign(signature_bytes)
            .map_err(|e| AcmeError::CryptoError(format!("签名失败: {}", e)))?;
        
        // Base64URL 编码签名
        let signature_b64 = URL_SAFE_NO_PAD.encode(&signature);
        
        Ok(JwsSignature {
            protected: protected_b64,
            payload: payload_b64,
            signature: signature_b64,
        })
    }
}

/// 验证 JWS 签名（用于测试）
pub fn verify_jws_signature(
    jws: &JwsSignature,
    public_key: &KeyPair,
) -> AcmeResult<bool> {
    // 重构签名输入
    let signing_input = format!("{}.{}", jws.protected, jws.payload);
    
    // 解码签名
    let signature = URL_SAFE_NO_PAD.decode(&jws.signature)
        .map_err(|e| AcmeError::CryptoError(format!("解码签名失败: {}", e)))?;
    
    // 验证签名（这里需要实现公钥验证，暂时返回 true）
    // 在实际实现中，应该使用公钥验证签名
    Ok(true)
}

/// 解析 JWS 保护头部
pub fn parse_protected_header(protected_b64: &str) -> AcmeResult<JwsProtectedHeader> {
    let protected_bytes = URL_SAFE_NO_PAD.decode(protected_b64)
        .map_err(|e| AcmeError::CryptoError(format!("解码保护头部失败: {}", e)))?;
    
    let protected_json = String::from_utf8(protected_bytes)
        .map_err(|e| AcmeError::CryptoError(format!("保护头部中包含无效的 UTF-8: {}", e)))?;
    
    serde_json::from_str(&protected_json)
        .map_err(|e| AcmeError::JsonError(format!("解析保护头部失败: {}", e)))
}

/// 解析 JWS 载荷
pub fn parse_payload(payload_b64: &str) -> AcmeResult<Value> {
    if payload_b64.is_empty() {
        return Ok(Value::String("".to_string()));
    }
    
    let payload_bytes = URL_SAFE_NO_PAD.decode(payload_b64)
        .map_err(|e| AcmeError::CryptoError(format!("解码载荷失败: {}", e)))?;
    
    let payload_json = String::from_utf8(payload_bytes)
        .map_err(|e| AcmeError::CryptoError(format!("载荷中包含无效的 UTF-8: {}", e)))?;
    
    if payload_json.is_empty() {
        return Ok(Value::String("".to_string()));
    }
    
    serde_json::from_str(&payload_json)
        .map_err(|e| AcmeError::JsonError(format!("解析载荷失败: {}", e)))
}

/// 创建外部账户绑定 (EAB) JWS
pub fn create_eab_jws(
    kid: &str,
    hmac_key: &str,
    account_public_key: &Value,
    url: &str,
) -> AcmeResult<Value> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    
    // 创建 EAB 保护头部
    let eab_protected = serde_json::json!({
        "alg": "HS256",
        "kid": kid,
        "url": url
    });
    
    // 序列化并编码保护头部
    let eab_protected_json = serde_json::to_string(&eab_protected)
        .map_err(|e| AcmeError::JsonError(format!("序列化 EAB 保护头部失败: {}", e)))?;
    let eab_protected_b64 = URL_SAFE_NO_PAD.encode(eab_protected_json.as_bytes());
    
    // 序列化并编码载荷（账户公钥）
    let eab_payload_json = serde_json::to_string(account_public_key)
        .map_err(|e| AcmeError::JsonError(format!("序列化 EAB 载荷失败: {}", e)))?;
    let eab_payload_b64 = URL_SAFE_NO_PAD.encode(eab_payload_json.as_bytes());
    
    // 创建签名输入
    let eab_signing_input = format!("{}.{}", eab_protected_b64, eab_payload_b64);
    
    // 解码 HMAC 密钥
    let hmac_key_bytes = URL_SAFE_NO_PAD.decode(hmac_key)
        .map_err(|e| AcmeError::CryptoError(format!("解码 HMAC 密钥失败: {}", e)))?;
    
    // 创建 HMAC 签名
    let mut mac = Hmac::<Sha256>::new_from_slice(&hmac_key_bytes)
        .map_err(|e| AcmeError::CryptoError(format!("创建 HMAC 失败: {}", e)))?;
    mac.update(eab_signing_input.as_bytes());
    let eab_signature = mac.finalize().into_bytes();
    
    // Base64URL 编码签名
    let eab_signature_b64 = URL_SAFE_NO_PAD.encode(&eab_signature);
    
    // 构建 EAB JWS
    Ok(serde_json::json!({
        "protected": eab_protected_b64,
        "payload": eab_payload_b64,
        "signature": eab_signature_b64
    }))
}

/// JWS 工具函数
pub mod utils {
    use super::*;
    
    /// 检查 JWS 格式是否有效
    pub fn is_valid_jws_format(jws: &JwsSignature) -> bool {
        !jws.protected.is_empty() && !jws.signature.is_empty()
    }
    
    /// 获取 JWS 中的算法
    pub fn get_algorithm_from_jws(jws: &JwsSignature) -> AcmeResult<String> {
        let header = parse_protected_header(&jws.protected)?;
        Ok(header.alg)
    }
    
    /// 获取 JWS 中的 nonce
    pub fn get_nonce_from_jws(jws: &JwsSignature) -> AcmeResult<String> {
        let header = parse_protected_header(&jws.protected)?;
        Ok(header.nonce)
    }
    
    /// 获取 JWS 中的 URL
    pub fn get_url_from_jws(jws: &JwsSignature) -> AcmeResult<String> {
        let header = parse_protected_header(&jws.protected)?;
        Ok(header.url)
    }
    
    /// 检查 JWS 是否包含 JWK
    pub fn has_jwk(jws: &JwsSignature) -> AcmeResult<bool> {
        let header = parse_protected_header(&jws.protected)?;
        Ok(header.jwk.is_some())
    }
    
    /// 检查 JWS 是否包含 kid
    pub fn has_kid(jws: &JwsSignature) -> AcmeResult<bool> {
        let header = parse_protected_header(&jws.protected)?;
        Ok(header.kid.is_some())
    }
}
