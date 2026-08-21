//! # A2F - Analysis to Fake Protocol
//!
//! 非同期・順不同・高遅延耐性を持つ暗号プロトコル

mod error;
mod crypto;
mod protocol;
mod shuffle;
mod config;

pub use error::{A2FError, A2FResult};
pub use crypto::MultiLayerCrypto;
pub use protocol::{Packet, PayloadType, TimestampBuffer};
pub use shuffle::ShuffleScheduler;
pub use config::A2FConfig;

use rand::RngCore;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;

/// 現在のタイムスタンプをミリ秒単位で取得
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

/// 送信側
pub struct A2FSender {
    crypto: MultiLayerCrypto,
    shuffler: ShuffleScheduler,
    current_session_key: Option<[u8; 32]>,
}

impl A2FSender {
    pub fn new(master_key: [u8; 32], config: &A2FConfig) -> Self {
        Self {
            crypto: MultiLayerCrypto::new(master_key),
            shuffler: config.into_scheduler(),
            current_session_key: None,
        }
    }
    
    /// 新しいセッション鍵を生成し、ラップする
    pub fn generate_key_packet(&mut self, timestamp: u64) -> A2FResult<Packet> {
        let mut session_key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut session_key);
        self.current_session_key = Some(session_key);
        
        let wrapped = self.crypto.wrap_session_key(&session_key)?;
        
        Ok(Packet::new(timestamp, PayloadType::WrappedKey, wrapped))
    }
    
    /// データを暗号化する
    pub fn encrypt_data(&mut self, data: &[u8], timestamp: u64) -> A2FResult<Packet> {
        let session_key = self.current_session_key
            .ok_or_else(|| A2FError::ConfigError("セッション鍵が生成されていません".into()))?;
        
        let encrypted = self.crypto.encrypt_data(&session_key, data)?;
        
        Ok(Packet::new(timestamp, PayloadType::EncryptedData, encrypted))
    }
    
    /// ダミーパケットを生成
    pub fn generate_dummy_packet(&mut self, timestamp: u64) -> Packet {
        Packet::dummy(timestamp)
    }
    
    /// パケットをシャッフルする
    pub fn shuffle_packets<T>(&mut self, packets: Vec<T>) -> Vec<T> {
        self.shuffler.shuffle_packets(packets)
    }
}

/// 受信側
pub struct A2FReceiver {
    crypto: MultiLayerCrypto,
    buffer: TimestampBuffer,
    pending_keys: HashMap<u64, [u8; 32]>,
}

impl A2FReceiver {
    pub fn new(master_key: [u8; 32], config: &A2FConfig) -> Self {
        Self {
            crypto: MultiLayerCrypto::new(master_key),
            buffer: TimestampBuffer::new(config.buffer_timeout_secs, config.buffer_max_size),
            pending_keys: HashMap::new(),
        }
    }
    
    /// パケットを受信し、復号できたらデータを返す
    pub fn receive_packet(&mut self, packet: Packet) -> A2FResult<Option<Vec<u8>>> {
        match packet.payload_type {
            PayloadType::WrappedKey => {
                let session_key = self.crypto.unwrap_session_key(&packet.payload)?;
                self.pending_keys.insert(packet.timestamp, session_key);
                
                // バッファにすでに同じタイムスタンプのデータがあれば結合
                if let Some((_, data)) = self.buffer.insert_key(packet.timestamp, packet.payload) {
                    let key = self.pending_keys.get(&packet.timestamp).unwrap();
                    return Ok(Some(self.crypto.decrypt_data(key, &data)?));
                }
            }
            PayloadType::EncryptedData => {
                // すでに鍵があれば即復号
                if let Some(key) = self.pending_keys.get(&packet.timestamp) {
                    return Ok(Some(self.crypto.decrypt_data(key, &packet.payload)?));
                }
                
                // なければバッファ
                if let Some((wrapped_key, data)) = self.buffer.insert_data(packet.timestamp, packet.payload) {
                    let key = self.crypto.unwrap_session_key(&wrapped_key)?;
                    self.pending_keys.insert(packet.timestamp, key);
                    return Ok(Some(self.crypto.decrypt_data(&key, &data)?));
                }
            }
            PayloadType::Dummy | PayloadType::Heartbeat => {
                // 何もしない
            }
        }
        
        Ok(None)
    }
    
    pub fn pending_count(&self) -> usize {
        self.buffer.pending_count()
    }
    
    pub fn clear_expired(&mut self) -> usize {
        self.buffer.clear_expired()
    }
}