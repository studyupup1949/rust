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
    
    #[error("シーケンス番号が古すぎます: {0}")]
    ExpiredSequence(u64),
    
    #[error("バッファオーバーフロー")]
    BufferOverflow,
    
    #[error("設定エラー: {0}")]
    ConfigError(String),
    
    #[error("セッションエラー: {0}")]
    SessionError(String),
    
    #[error("セッションが期限切れです")]
    SessionExpired,
    
    #[error("セッションIDが一致しません")]
    SessionIdMismatch,
    
    #[error("セッション状態が不正です: {0}")]
    InvalidSessionState(String),
}

pub type A2FResult<T> = Result<T, A2FError>;