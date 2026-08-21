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
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};

static GLOBAL_SEQ: AtomicU64 = AtomicU64::new(1);

/// 現在のタイムスタンプをミリ秒単位で取得
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

pub fn next_sequence() -> u64 {
    GLOBAL_SEQ.fetch_add(1, Ordering::SeqCst)
}

/// 送信側
pub struct A2FSender {
    crypto: MultiLayerCrypto,
    shuffler: ShuffleScheduler,
    current_session_key: Option<[u8; 32]>,
    next_seq: u64,
}

impl A2FSender {
    pub fn new(master_key: [u8; 32], config: &A2FConfig) -> Self {
        Self {
            crypto: MultiLayerCrypto::new(master_key),
            shuffler: config.into_scheduler(),
            current_session_key: None,
            next_seq: next_sequence(),
        }
    }
    
    pub fn set_next_sequence(&mut self, seq: u64) {
        self.next_seq = seq;
    }
    
    /// 新しいセッション鍵を生成し、ラップする
    pub fn generate_key_packet(&mut self, timestamp: u64) -> A2FResult<Packet> {
        let mut session_key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut session_key);
        self.current_session_key = Some(session_key);
        
        let wrapped = self.crypto.wrap_session_key(&session_key)?;
        let seq = self.next_seq;
        self.next_seq += 1;
        
        Ok(Packet::new(seq, timestamp, PayloadType::WrappedKey, wrapped))
    }
    
    /// データを暗号化する
    pub fn encrypt_data(&mut self, data: &[u8], timestamp: u64) -> A2FResult<Packet> {
        let session_key = self.current_session_key
            .ok_or_else(|| A2FError::ConfigError("セッション鍵が生成されていません".into()))?;
        
        let encrypted = self.crypto.encrypt_data(&session_key, data)?;
        let seq = self.next_seq;
        self.next_seq += 1;
        
        Ok(Packet::new(seq, timestamp, PayloadType::EncryptedData, encrypted))
    }
    
    /// ダミーパケットを生成
    pub fn generate_dummy_packet(&mut self, timestamp: u64) -> Packet {
        let seq = self.next_seq;
        self.next_seq += 1;
        Packet::dummy(seq, timestamp)
    }
    
    /// ハートビートパケットを生成
    pub fn generate_heartbeat_packet(&mut self, timestamp: u64) -> Packet {
        let seq = self.next_seq;
        self.next_seq += 1;
        Packet::heartbeat(seq, timestamp)
    }
    
    /// パケットをシャッフルする
    pub fn shuffle_packets<T>(&mut self, packets: Vec<T>) -> Vec<T> {
        self.shuffler.shuffle_packets(packets)
    }
}

/// スライディングウィンドウ（リプレイ対策＋順序自由）
struct SlidingWindow {
    window_size: u64,
    min_seq: u64,
    received: VecDeque<bool>,
}

impl SlidingWindow {
    fn new(window_size: u64) -> Self {
        Self {
            window_size,
            min_seq: 0,
            received: VecDeque::from(vec![false; window_size as usize]),
        }
    }
    
    /// シーケンス番号をチェックし、受信済みとして記録する
    fn check_and_record(&mut self, seq: u64) -> bool {
        // ウィンドウ下限より古いものは拒否
        if seq < self.min_seq {
            return false;
        }
        
        // ウィンドウ上限を超えている場合はウィンドウをスライド
        if seq >= self.min_seq + self.window_size {
            let shift = seq - (self.min_seq + self.window_size) + 1;
            for _ in 0..shift {
                self.received.pop_front();
                self.received.push_back(false);
            }
            self.min_seq += shift;
        }
        
        let index = (seq - self.min_seq) as usize;
        if self.received[index] {
            // すでに受信済み → リプレイ攻撃
            return false;
        }
        
        // 初めてのパケット → 受信済みとして記録
        self.received[index] = true;
        true
    }
    
    #[allow(dead_code)]
    fn get_min_seq(&self) -> u64 {
        self.min_seq
    }
}

/// 受信側
pub struct A2FReceiver {
    crypto: MultiLayerCrypto,
    buffer: TimestampBuffer,
    pending_keys: HashMap<u64, [u8; 32]>,
    sliding_window: SlidingWindow,
}

impl A2FReceiver {
    pub fn new(master_key: [u8; 32], config: &A2FConfig) -> Self {
        Self {
            crypto: MultiLayerCrypto::new(master_key),
            buffer: TimestampBuffer::new(config.buffer_timeout_secs, config.buffer_max_size),
            pending_keys: HashMap::new(),
            sliding_window: SlidingWindow::new(128),  // ウィンドウサイズ128
        }
    }
    
    /// パケットを受信し、復号できたらデータを返す
    pub fn receive_packet(&mut self, packet: Packet) -> A2FResult<Option<Vec<u8>>> {
        // リプレイ攻撃対策：スライディングウィンドウでチェック
        if !self.sliding_window.check_and_record(packet.seq) {
            return Err(A2FError::ExpiredSequence(packet.seq));
        }
        
        match packet.payload_type {
            PayloadType::WrappedKey => {
                let session_key = self.crypto.unwrap_session_key(&packet.payload)?;
                self.pending_keys.insert(packet.timestamp, session_key);
                
                if let Some((_, data)) = self.buffer.insert_key(packet.timestamp, packet.payload) {
                    let key = self.pending_keys.get(&packet.timestamp).unwrap();
                    return Ok(Some(self.crypto.decrypt_data(key, &data)?));
                }
            }
            PayloadType::EncryptedData => {
                if let Some(key) = self.pending_keys.get(&packet.timestamp) {
                    return Ok(Some(self.crypto.decrypt_data(key, &packet.payload)?));
                }
                
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
    
    #[allow(dead_code)]
    pub fn current_min_seq(&self) -> u64 {
        self.sliding_window.get_min_seq()
    }
}
