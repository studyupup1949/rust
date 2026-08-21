//! 加密模块
//! 提供密钥生成、签名和PEM格式处理功能

pub mod ecdsa;
pub mod pem;

// 重新导出常用类型
pub use pem::{PemData, PemType};

use crate::error::{CryptoError, CryptoResult};
use ring::signature::{EcdsaKeyPair, ECDSA_P384_SHA384_FIXED_SIGNING};
use ring::rand::SystemRandom;

/// 密钥对结构
#[derive(Debug, Clone)]
pub struct KeyPair {
    /// 私钥 PEM 格式
    pub private_key_pem: String,
    /// 公钥 PEM 格式
    pub public_key_pem: String,
    /// Ring 密钥对（用于签名）
    pub(crate) ring_key_pair: Vec<u8>, // 存储 PKCS#8 格式的私钥
}

impl KeyPair {
    /// 创建新的密钥对（别名）
    pub fn new() -> CryptoResult<Self> {
        Self::generate()
    }
    
    /// 生成新的 ECDSA P-384 密钥对
    pub fn generate() -> CryptoResult<Self> {
        ecdsa::generate_secp384r1_key_pair()
    }
    
    /// 从 PEM 格式的私钥加载密钥对
    pub fn from_private_key_pem(private_key_pem: &str) -> CryptoResult<Self> {
        ecdsa::load_key_pair_from_pem(private_key_pem)
    }
    
    /// 获取私钥的 PEM 格式
    pub fn private_key_pem(&self) -> &str {
        &self.private_key_pem
    }
    
    /// 获取公钥的 PEM 格式
    pub fn public_key_pem(&self) -> &str {
        &self.public_key_pem
    }
    
    /// 获取私钥的 DER 格式
    pub fn private_key_der(&self) -> CryptoResult<Vec<u8>> {
        // 从 PEM 中提取 DER 数据
        use pem::PemData;
        let pem_data = PemData::from_pem_string(&self.private_key_pem)?;
        Ok(pem_data.data)
    }
    
    /// 转换为 PEM 格式（返回私钥 PEM）
    pub fn to_pem(&self) -> &str {
        &self.private_key_pem
    }
    
    /// 使用私钥对数据进行签名
    pub fn sign(&self, data: &[u8]) -> CryptoResult<Vec<u8>> {
        ecdsa::sign_data(&self.ring_key_pair, data)
    }
    
    /// 获取 JWK (JSON Web Key) 格式的公钥
    pub fn to_jwk(&self) -> CryptoResult<serde_json::Value> {
        ecdsa::public_key_to_jwk(&self.ring_key_pair)
    }
}

/// 算法类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algorithm {
    /// ECDSA P-384 with SHA-384
    EcdsaP384Sha384,
}

impl Algorithm {
    /// 获取算法的字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            Algorithm::EcdsaP384Sha384 => "ES384",
        }
    }
    
    /// 获取算法的 JWS 标识符
    pub fn jws_alg(&self) -> &'static str {
        match self {
            Algorithm::EcdsaP384Sha384 => "ES384",
        }
    }
}

/// 默认算法（固定为 ECDSA P-384）
pub const DEFAULT_ALGORITHM: Algorithm = Algorithm::EcdsaP384Sha384;

/// 便捷函数：生成默认的密钥对
pub fn generate_key_pair() -> CryptoResult<KeyPair> {
    KeyPair::generate()
}

/// 便捷函数：从 PEM 加载密钥对
pub fn load_key_pair_from_pem(private_key_pem: &str) -> CryptoResult<KeyPair> {
    KeyPair::from_private_key_pem(private_key_pem)
}
