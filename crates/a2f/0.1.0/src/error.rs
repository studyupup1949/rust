use thiserror::Error;

#[derive(Error, Debug)]
pub enum A2FError {
    #[error("暗号化エラー: {0}")]
    CryptoError(String),
    
    #[error("復号エラー: {0}")]
    DecryptionError(String),
    
    #[error("パケット解析エラー: {0}")]
    PacketError(String),
    
    #[error("タイムスタンプが古すぎます: {0}")]
    ExpiredTimestamp(u64),
    
    #[error("バッファオーバーフロー")]
    BufferOverflow,
    
    #[error("設定エラー: {0}")]
    ConfigError(String),
}

pub type A2FResult<T> = Result<T, A2FError>;