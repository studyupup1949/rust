//! ECDSA 密钥生成和处理模块
//! 专门处理 ECDSA P-384 (secp384r1) 密钥对的生成、加载和签名操作

use crate::error::{CryptoError, CryptoResult};
use crate::crypto::KeyPair;
use ring::signature::{EcdsaKeyPair, ECDSA_P384_SHA384_FIXED_SIGNING, KeyPair as RingKeyPair};
use ring::rand::SystemRandom;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde_json::json;

/// 生成 ECDSA P-384 密钥对
pub fn generate_secp384r1_key_pair() -> CryptoResult<KeyPair> {
    let rng = SystemRandom::new();
    
    // 生成 ECDSA P-384 密钥对
    let pkcs8_bytes = EcdsaKeyPair::generate_pkcs8(&ECDSA_P384_SHA384_FIXED_SIGNING, &rng)
        .map_err(|_| CryptoError::KeyGenerationFailed("Failed to generate ECDSA P-384 key pair".to_string()))?;
    
    // 从 PKCS#8 格式创建密钥对
    let key_pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P384_SHA384_FIXED_SIGNING, pkcs8_bytes.as_ref(), &rng)
        .map_err(|_| CryptoError::KeyGenerationFailed("Failed to create key pair from PKCS#8".to_string()))?;
    
    // 转换为 PEM 格式
    let private_key_pem = pkcs8_to_pem(pkcs8_bytes.as_ref(), "PRIVATE KEY")?;
    let public_key_pem = public_key_to_pem(&key_pair)?;
    
    Ok(KeyPair {
        private_key_pem,
        public_key_pem,
        ring_key_pair: pkcs8_bytes.as_ref().to_vec(),
    })
}

/// 从 PEM 格式的私钥加载密钥对
pub fn load_key_pair_from_pem(private_key_pem: &str) -> CryptoResult<KeyPair> {
    // 解析 PEM 格式
    let pem = pem::parse(private_key_pem)
        .map_err(|e| CryptoError::PemError(format!("Failed to parse PEM: {}", e)))?;
    
    // 检查 PEM 标签
    if pem.tag() != "PRIVATE KEY" {
        return Err(CryptoError::InvalidKeyFormat);
    }
    
    // 从 PKCS#8 格式创建密钥对
    let rng = SystemRandom::new();
    let key_pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P384_SHA384_FIXED_SIGNING, pem.contents(), &rng)
        .map_err(|_| CryptoError::KeyParsingFailed("Invalid ECDSA P-384 private key".to_string()))?;
    
    // 生成公钥 PEM
    let public_key_pem = public_key_to_pem(&key_pair)?;
    
    Ok(KeyPair {
        private_key_pem: private_key_pem.to_string(),
        public_key_pem,
        ring_key_pair: pem.contents().to_vec(),
    })
}

/// 使用私钥对数据进行签名
pub fn sign_data(pkcs8_key: &[u8], data: &[u8]) -> CryptoResult<Vec<u8>> {
    let rng = SystemRandom::new();
    let key_pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P384_SHA384_FIXED_SIGNING, pkcs8_key, &rng)
        .map_err(|_| CryptoError::SignatureFailed("Invalid private key for signing".to_string()))?;
    
    let rng = SystemRandom::new();
    let signature = key_pair.sign(&rng, data)
        .map_err(|_| CryptoError::SignatureFailed("Failed to sign data".to_string()))?;
    
    Ok(signature.as_ref().to_vec())
}

/// 将公钥转换为 JWK 格式
pub fn public_key_to_jwk(pkcs8_key: &[u8]) -> CryptoResult<serde_json::Value> {
    let rng = SystemRandom::new();
    let key_pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P384_SHA384_FIXED_SIGNING, pkcs8_key, &rng)
        .map_err(|_| CryptoError::KeyParsingFailed("Invalid private key".to_string()))?;
    
    // 获取公钥字节
    let public_key_bytes = RingKeyPair::public_key(&key_pair).as_ref();
    
    // ECDSA P-384 公钥格式：0x04 + x坐标(48字节) + y坐标(48字节)
    if public_key_bytes.len() != 97 || public_key_bytes[0] != 0x04 {
        return Err(CryptoError::InvalidKeyFormat);
    }
    
    // 提取 x 和 y 坐标
    let x = &public_key_bytes[1..49];
    let y = &public_key_bytes[49..97];
    
    // 转换为 base64url 编码
    let x_b64 = URL_SAFE_NO_PAD.encode(x);
    let y_b64 = URL_SAFE_NO_PAD.encode(y);
    
    Ok(json!({
        "kty": "EC",
        "crv": "P-384",
        "x": x_b64,
        "y": y_b64,
        "use": "sig",
        "alg": "ES384"
    }))
}

/// 将公钥转换为 PEM 格式
fn public_key_to_pem(key_pair: &EcdsaKeyPair) -> CryptoResult<String> {
    // 获取公钥字节
    let public_key_bytes = RingKeyPair::public_key(key_pair).as_ref();
    
    // 构建 SubjectPublicKeyInfo 结构
    // 这是一个简化的实现，实际应该使用 ASN.1 编码
    let spki = build_p384_spki(public_key_bytes)?;
    
    // 转换为 PEM 格式
    pkcs8_to_pem(&spki, "PUBLIC KEY")
}

/// 构建 P-384 的 SubjectPublicKeyInfo 结构
fn build_p384_spki(public_key: &[u8]) -> CryptoResult<Vec<u8>> {
    // P-384 的 OID: 1.2.840.10045.3.1.34
    // 这是一个硬编码的 ASN.1 DER 编码的 SubjectPublicKeyInfo 结构
    let mut spki = Vec::new();
    
    // SEQUENCE
    spki.push(0x30);
    spki.push(0x76); // 长度
    
    // AlgorithmIdentifier SEQUENCE
    spki.push(0x30);
    spki.push(0x10);
    
    // algorithm OBJECT IDENTIFIER (ecPublicKey)
    spki.extend_from_slice(&[0x06, 0x07, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02, 0x01]);
    
    // parameters OBJECT IDENTIFIER (secp384r1)
    spki.extend_from_slice(&[0x06, 0x05, 0x2b, 0x81, 0x04, 0x00, 0x22]);
    
    // subjectPublicKey BIT STRING
    spki.push(0x03);
    spki.push(0x62); // 长度
    spki.push(0x00); // 未使用的位数
    
    // 公钥数据
    spki.extend_from_slice(public_key);
    
    Ok(spki)
}

/// 将字节数组转换为 PEM 格式
fn pkcs8_to_pem(der_bytes: &[u8], label: &str) -> CryptoResult<String> {
    let pem = pem::Pem::new(label, der_bytes);
    Ok(pem::encode(&pem))
}

/// 验证密钥是否为有效的 ECDSA P-384 密钥
pub fn validate_key_pair(private_key_pem: &str) -> CryptoResult<bool> {
    match load_key_pair_from_pem(private_key_pem) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// 从公钥 PEM 提取公钥字节
pub fn extract_public_key_bytes(public_key_pem: &str) -> CryptoResult<Vec<u8>> {
    let pem = pem::parse(public_key_pem)
        .map_err(|e| CryptoError::PemError(format!("Failed to parse public key PEM: {}", e)))?;
    
    if pem.tag() != "PUBLIC KEY" {
        return Err(CryptoError::InvalidKeyFormat);
    }
    
    // 解析 SubjectPublicKeyInfo 结构，提取公钥字节
    // 这是一个简化的实现
    let der = pem.contents();
    if der.len() < 120 {
        return Err(CryptoError::InvalidKeyFormat);
    }
    
    // 跳过 ASN.1 结构，直接提取公钥部分
    // 实际的公钥数据在 DER 编码的末尾 97 字节
    let public_key_start = der.len() - 97;
    Ok(der[public_key_start..].to_vec())
}
