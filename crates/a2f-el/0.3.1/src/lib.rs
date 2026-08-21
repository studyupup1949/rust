//! # A2F-EL (Essential Lite) - Analysis to Fake Protocol Essential Lite
//!
//! 必要不可欠な要素に厳選した、軽量で実用的な暗号プロトコル
//! - ChaCha20-Poly1305 単層暗号化
//! - X25519鍵交換（内蔵）
//! - スライディングウィンドウによるリプレイ対策
//! - 非同期・順不同・高遅延耐性

mod error;
mod crypto;
mod protocol;
mod shuffle;    
mod config;
mod replay;

pub use error::{A2FError, A2FResult};
pub use crypto::{SimpleCrypto, KeyExchange};
pub use protocol::{Packet, PayloadType, TimestampBuffer};
pub use shuffle::ShuffleScheduler;  
pub use config::A2FConfig;
pub use replay::SlidingWindow;

use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

static GLOBAL_SEQ: AtomicU64 = AtomicU64::new(1);

pub fn next_sequence() -> u64 {
    GLOBAL_SEQ.fetch_add(1, Ordering::SeqCst)
}

// セッション状態を定義
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SessionState {
    Init,
    KeyExchange,
    Established,
    Expired,
    Terminated,
}

// セッション情報
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub id: u64,
    pub state: SessionState,
    pub created_at: u64,
    pub last_activity: u64,
    pub timeout_ms: u64,
}

impl SessionInfo {
    pub fn new(id: u64, timeout_ms: u64) -> Self {
        let now = current_timestamp();
        Self {
            id,
            state: SessionState::Init,
            created_at: now,
            last_activity: now,
            timeout_ms,
        }
    }
    
    pub fn is_expired(&self) -> bool {
        let elapsed = current_timestamp() - self.last_activity;
        elapsed > self.timeout_ms
    }
    
    pub fn update_activity(&mut self) {
        self.last_activity = current_timestamp();
    }
}

pub struct A2FELSender {
    crypto: SimpleCrypto,
    key_exchange: KeyExchange,
    shuffler: ShuffleScheduler,
    session_key: Option<[u8; 32]>,
    session_info: Option<SessionInfo>,
    peer_public_key: Option<[u8; 32]>,
    next_seq: u64,
    session_id: u64,
    session_timeout_ms: u64,
}

impl A2FELSender {
    pub fn new() -> Self {
        Self {
            crypto: SimpleCrypto::new(),
            key_exchange: KeyExchange::new(),
            shuffler: ShuffleScheduler::new(),
            session_key: None,
            session_info: None,
            peer_public_key: None,
            next_seq: next_sequence(),
            session_id: rand::random::<u64>(),
            session_timeout_ms: 3600 * 1000, // デフォルト1時間
        }
    }

    pub fn start_key_exchange(&mut self) -> [u8; 32] {
        let pub_key = self.key_exchange.generate_keypair();
        self.session_info = Some(SessionInfo::new(self.session_id, self.session_timeout_ms));
        pub_key
    }

    pub fn complete_key_exchange(&mut self, peer_public: &[u8; 32]) -> A2FResult<()> {
        let shared = self.key_exchange.compute_shared_secret(peer_public)?;
        let session_key = self.derive_session_key(&shared);
        self.session_key = Some(session_key);
        self.crypto.set_key(session_key);
        self.peer_public_key = Some(*peer_public);
        
        if let Some(info) = &mut self.session_info {
            info.state = SessionState::Established;
            info.update_activity();
        }
        Ok(())
    }

    fn derive_session_key(&self, shared: &[u8; 32]) -> [u8; 32] {
        use sha2::Sha256;
        use hkdf::Hkdf;
        
        let hkdf = Hkdf::<Sha256>::new(None, shared);
        let mut okm = [0u8; 32];
        hkdf.expand(b"a2f-el-sender-key", &mut okm).unwrap();
        okm
    }

    pub fn is_session_expired(&self) -> bool {
        if let Some(info) = &self.session_info {
            info.is_expired()
        } else {
            true
        }
    }

    pub fn get_session_state(&self) -> Option<SessionState> {
        self.session_info.as_ref().map(|info| info.state)
    }

    pub fn encrypt_data(&mut self, data: &[u8], timestamp: u64) -> A2FResult<Packet> {
        if self.is_session_expired() {
            return Err(A2FError::SessionExpired);
        }
        
        if self.session_key.is_none() {
            return Err(A2FError::ConfigError("鍵交換が完了していません".into()));
        }
        
        let encrypted = self.crypto.encrypt(data)?;
        let seq = self.next_seq;
        self.next_seq += 1;
        
        if let Some(info) = &mut self.session_info {
            info.update_activity();
        }
        
        Ok(Packet::new(self.session_id, seq, timestamp, PayloadType::EncryptedData, encrypted))
    }

    pub fn make_key_packet(&mut self, timestamp: u64) -> A2FResult<Packet> {
        let public_key = self.start_key_exchange();
        let seq = self.next_seq;
        self.next_seq += 1;
        
        if let Some(info) = &mut self.session_info {
            info.state = SessionState::KeyExchange;
            info.update_activity();
        }
        
        Ok(Packet::new(self.session_id, seq, timestamp, PayloadType::WrappedKey, public_key.to_vec()))
    }

    pub fn send_multiple(&mut self, chunks: &[&[u8]]) -> A2FResult<Vec<Packet>> {
        if chunks.is_empty() {
            return Ok(vec![]);
        }
        
        let ts = current_timestamp();
        let mut packets = Vec::new();
        
        for (i, chunk) in chunks.iter().enumerate() {
            let chunk_ts = ts + i as u64;
            let data_packet = self.encrypt_data(chunk, chunk_ts)?;
            packets.push(data_packet);
        }
        
        packets = self.shuffler.shuffle_packets(packets);
        Ok(packets)
    }

    pub fn shuffle_packets<T>(&mut self, packets: Vec<T>) -> Vec<T> {
        self.shuffler.shuffle_packets(packets)
    }

    pub fn get_session_key(&self) -> Option<[u8; 32]> {
        self.session_key
    }

    pub fn terminate_session(&mut self) {
        self.session_key = None;
        self.crypto.clear_key();
        self.peer_public_key = None;
        if let Some(info) = &mut self.session_info {
            info.state = SessionState::Terminated;
        }
    }

    pub fn get_session_info(&self) -> Option<&SessionInfo> {
        self.session_info.as_ref()
    }

    pub fn get_session_id(&self) -> u64 {
        self.session_id
    }
}

impl Default for A2FELSender {
    fn default() -> Self {
        Self::new()
    }
}

pub struct A2FELReceiver {
    crypto: SimpleCrypto,
    key_exchange: KeyExchange,
    buffer: TimestampBuffer,
    pending_keys: HashMap<u64, [u8; 32]>,
    sliding_window: SlidingWindow,
    session_key: Option<[u8; 32]>,
    session_info: Option<SessionInfo>,
    peer_public_key: Option<[u8; 32]>,
    session_id: u64,  // Senderから受け取る（初期値0）
    session_timeout_ms: u64,
}

impl A2FELReceiver {
    pub fn new(config: &A2FConfig) -> Self {
        Self {
            crypto: SimpleCrypto::new(),
            key_exchange: KeyExchange::new(),
            buffer: TimestampBuffer::new(config.buffer_timeout_secs, config.buffer_max_size),
            pending_keys: HashMap::new(),
            sliding_window: SlidingWindow::new(config.replay_window_size),
            session_key: None,
            session_info: None,
            peer_public_key: None,
            session_id: 0,  // 初期値0、初回パケットでSenderのIDを採用
            session_timeout_ms: 3600 * 1000, // デフォルト1時間
        }
    }

    pub fn get_public_key(&mut self) -> [u8; 32] {
        self.key_exchange.generate_keypair()
    }

    pub fn complete_key_exchange(&mut self, peer_public: &[u8; 32]) -> A2FResult<()> {
        let shared = self.key_exchange.compute_shared_secret(peer_public)?;
        let session_key = self.derive_session_key(&shared);
        self.session_key = Some(session_key);
        self.crypto.set_key(session_key);
        self.peer_public_key = Some(*peer_public);
        
        if let Some(info) = &mut self.session_info {
            info.state = SessionState::Established;
            info.update_activity();
        }
        Ok(())
    }

    fn derive_session_key(&self, shared: &[u8; 32]) -> [u8; 32] {
        use sha2::Sha256;
        use hkdf::Hkdf;
        
        let hkdf = Hkdf::<Sha256>::new(None, shared);
        let mut okm = [0u8; 32];
        hkdf.expand(b"a2f-el-receiver-key", &mut okm).unwrap();
        okm
    }

    pub fn is_session_expired(&self) -> bool {
        if let Some(info) = &self.session_info {
            info.is_expired()
        } else {
            true
        }
    }

    pub fn get_session_state(&self) -> Option<SessionState> {
        self.session_info.as_ref().map(|info| info.state)
    }

    pub fn receive_packet(&mut self, packet: Packet) -> A2FResult<Option<Vec<u8>>> {
        // 初回パケットの場合、Senderのsession_idを採用
        if self.session_id == 0 {
            self.session_id = packet.session_id;
            self.session_info = Some(SessionInfo::new(self.session_id, self.session_timeout_ms));
        }
        
        // セッションIDが一致するかチェック
        if packet.session_id != self.session_id {
            return Err(A2FError::SessionIdMismatch);
        }

        if self.is_session_expired() {
            return Err(A2FError::SessionExpired);
        }

        if !self.sliding_window.check_and_record(packet.seq) {
            return Err(A2FError::ExpiredSequence(packet.seq));
        }

        match packet.payload_type {
            PayloadType::WrappedKey => {
                if packet.payload.len() != 32 {
                    return Err(A2FError::DecryptionError("公開鍵の長さが不正".into()));
                }
                let mut peer_public = [0u8; 32];
                peer_public.copy_from_slice(&packet.payload);
                self.complete_key_exchange(&peer_public)?;
                
                if let Some(info) = &mut self.session_info {
                    info.update_activity();
                }
                Ok(None)
            }
            PayloadType::EncryptedData => {
                if self.session_key.is_none() {
                    return Err(A2FError::ConfigError("鍵交換が完了していません".into()));
                }
                let decrypted = self.crypto.decrypt(&packet.payload)?;
                
                if let Some(info) = &mut self.session_info {
                    info.update_activity();
                }
                Ok(Some(decrypted))
            }
        }
    }

    pub fn pending_count(&self) -> usize {
        self.buffer.pending_count()
    }

    pub fn clear_expired(&mut self) -> usize {
        self.buffer.clear_expired()
    }

    pub fn get_session_key(&self) -> Option<[u8; 32]> {
        self.session_key
    }

    pub fn terminate_session(&mut self) {
        self.session_key = None;
        self.crypto.clear_key();
        self.peer_public_key = None;
        self.session_id = 0;  // リセット
        if let Some(info) = &mut self.session_info {
            info.state = SessionState::Terminated;
        }
    }

    pub fn get_session_info(&self) -> Option<&SessionInfo> {
        self.session_info.as_ref()
    }

    pub fn get_session_id(&self) -> u64 {
        self.session_id
    }
}