use thiserror::Error;

#[derive(Error, Debug)]
pub enum UtilsError {
    #[error("鍵交換エラー: {0}")]
    KeyExchangeError(String),
    
    #[error("暗号化エラー: {0}")]
    CryptoError(String),
}

pub type UtilsResult<T> = Result<T, UtilsError>;