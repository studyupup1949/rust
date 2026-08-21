//! PEM 格式编码和解码模块
//! 提供 PEM 格式的读写、验证和转换功能

use crate::error::{CryptoError, CryptoResult};
use std::fs;
use std::path::Path;

/// PEM 文件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PemType {
    /// 私钥
    PrivateKey,
    /// 公钥
    PublicKey,
    /// 证书
    Certificate,
    /// 证书请求
    CertificateRequest,
}

impl PemType {
    /// 获取 PEM 标签
    pub fn label(&self) -> &'static str {
        match self {
            PemType::PrivateKey => "PRIVATE KEY",
            PemType::PublicKey => "PUBLIC KEY",
            PemType::Certificate => "CERTIFICATE",
            PemType::CertificateRequest => "CERTIFICATE REQUEST",
        }
    }
    
    /// 从标签识别 PEM 类型
    pub fn from_label(label: &str) -> Option<Self> {
        match label {
            "PRIVATE KEY" => Some(PemType::PrivateKey),
            "PUBLIC KEY" => Some(PemType::PublicKey),
            "CERTIFICATE" => Some(PemType::Certificate),
            "CERTIFICATE REQUEST" => Some(PemType::CertificateRequest),
            _ => None,
        }
    }
}

/// PEM 数据结构
#[derive(Debug, Clone)]
pub struct PemData {
    /// PEM 类型
    pub pem_type: PemType,
    /// 原始数据
    pub data: Vec<u8>,
    /// PEM 格式字符串
    pub pem_string: String,
}

impl PemData {
    /// 创建新的 PEM 数据
    pub fn new(pem_type: PemType, data: Vec<u8>) -> CryptoResult<Self> {
        let pem = pem::Pem::new(pem_type.label(), data.clone());
        let pem_string = pem::encode(&pem);
        
        Ok(PemData {
            pem_type,
            data,
            pem_string,
        })
    }
    
    /// 从 PEM 字符串解析
    pub fn from_pem_string(pem_string: &str) -> CryptoResult<Self> {
        let pem = pem::parse(pem_string)
            .map_err(|e| CryptoError::PemError(format!("Failed to parse PEM: {}", e)))?;
        
        let pem_type = PemType::from_label(pem.tag())
            .ok_or_else(|| CryptoError::PemError(format!("Unknown PEM type: {}", pem.tag())))?;
        
        Ok(PemData {
            pem_type,
            data: pem.contents().to_vec(),
            pem_string: pem_string.to_string(),
        })
    }
    
    /// 从文件加载 PEM 数据
    pub fn from_file<P: AsRef<Path>>(path: P) -> CryptoResult<Self> {
        let content = fs::read_to_string(path)
            .map_err(|e| CryptoError::PemError(format!("Failed to read PEM file: {}", e)))?;
        
        Self::from_pem_string(&content)
    }
    
    /// 保存到文件
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> CryptoResult<()> {
        fs::write(path, &self.pem_string)
            .map_err(|e| CryptoError::PemError(format!("Failed to write PEM file: {}", e)))?;
        
        Ok(())
    }
    
    /// 获取 PEM 字符串
    pub fn as_pem_string(&self) -> &str {
        &self.pem_string
    }
    
    /// 获取原始数据
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
    
    /// 验证 PEM 格式是否正确
    pub fn validate(&self) -> CryptoResult<()> {
        // 重新解析以验证格式
        let parsed = pem::parse(&self.pem_string)
            .map_err(|e| CryptoError::PemError(format!("PEM validation failed: {}", e)))?;
        
        // 验证标签
        if parsed.tag() != self.pem_type.label() {
            return Err(CryptoError::PemError(
                format!("PEM label mismatch: expected {}, got {}", 
                    self.pem_type.label(), parsed.tag())
            ));
        }
        
        // 验证数据
        if parsed.contents() != self.data {
            return Err(CryptoError::PemError("PEM data mismatch".to_string()));
        }
        
        Ok(())
    }
}

/// 从字节数据创建 PEM 格式字符串
pub fn encode_pem(data: &[u8], label: &str) -> String {
    let pem = pem::Pem::new(label, data);
    pem::encode(&pem)
}

/// 从 PEM 格式字符串解码数据
pub fn decode_pem(pem_string: &str) -> CryptoResult<(String, Vec<u8>)> {
    let pem = pem::parse(pem_string)
        .map_err(|e| CryptoError::PemError(format!("Failed to decode PEM: {}", e)))?;
    
    Ok((pem.tag().to_string(), pem.contents().to_vec()))
}

/// 验证 PEM 格式字符串
pub fn validate_pem_format(pem_string: &str) -> CryptoResult<PemType> {
    let pem = pem::parse(pem_string)
        .map_err(|e| CryptoError::PemError(format!("Invalid PEM format: {}", e)))?;
    
    PemType::from_label(pem.tag())
        .ok_or_else(|| CryptoError::PemError(format!("Unknown PEM type: {}", pem.tag())))
}

/// 从文件读取 PEM 数据
pub fn read_pem_file<P: AsRef<Path>>(path: P) -> CryptoResult<PemData> {
    PemData::from_file(path)
}

/// 将 PEM 数据写入文件
pub fn write_pem_file<P: AsRef<Path>>(path: P, pem_type: PemType, data: &[u8]) -> CryptoResult<()> {
    let pem_data = PemData::new(pem_type, data.to_vec())?;
    pem_data.save_to_file(path)
}

/// 检查文件是否为有效的 PEM 文件
pub fn is_pem_file<P: AsRef<Path>>(path: P) -> bool {
    match fs::read_to_string(path) {
        Ok(content) => validate_pem_format(&content).is_ok(),
        Err(_) => false,
    }
}

/// 从 PEM 字符串中提取多个 PEM 块
pub fn parse_multiple_pem(pem_string: &str) -> CryptoResult<Vec<PemData>> {
    let mut results = Vec::new();
    let mut remaining = pem_string;
    
    while !remaining.trim().is_empty() {
        // 查找下一个 PEM 块的开始
        if let Some(start) = remaining.find("-----BEGIN ") {
            let pem_start = &remaining[start..];
            
            // 查找对应的结束标记
            if let Some(begin_line_end) = pem_start.find('\n') {
                let begin_line = &pem_start[..begin_line_end];
                let label_start = begin_line.find("-----BEGIN ").unwrap() + 11;
                let label_end = begin_line[label_start..].find("-----").unwrap() + label_start;
                let label = &begin_line[label_start..label_end];
                
                let end_marker = format!("-----END {}-----", label);
                if let Some(end_pos) = pem_start.find(&end_marker) {
                    let pem_block = &pem_start[..end_pos + end_marker.len()];
                    
                    match PemData::from_pem_string(pem_block) {
                        Ok(pem_data) => results.push(pem_data),
                        Err(e) => return Err(e),
                    }
                    
                    remaining = &remaining[start + end_pos + end_marker.len()..];
                } else {
                    return Err(CryptoError::PemError(
                        format!("Missing end marker for PEM block with label: {}", label)
                    ));
                }
            } else {
                return Err(CryptoError::PemError("Invalid PEM begin line".to_string()));
            }
        } else {
            break;
        }
    }
    
    if results.is_empty() {
        return Err(CryptoError::PemError("No valid PEM blocks found".to_string()));
    }
    
    Ok(results)
}

/// 将多个 PEM 数据合并为单个字符串
pub fn combine_pem_data(pem_data_list: &[PemData]) -> String {
    pem_data_list
        .iter()
        .map(|pem| pem.as_pem_string())
        .collect::<Vec<_>>()
        .join("\n")
}