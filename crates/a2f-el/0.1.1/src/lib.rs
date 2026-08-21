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

/// 現在のタイムスタンプをミリ秒単位で取得
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

pub struct A2FELSender {
    crypto: SimpleCrypto,
    key_exchange: KeyExchange,
    shuffler: ShuffleScheduler,
    session_key: Option<[u8; 32]>,
    peer_public_key: Option<[u8; 32]>,
    next_seq: u64,
}

impl A2FELSender {
    pub fn new() -> Self {
        Self {
            crypto: SimpleCrypto::new(),
            key_exchange: KeyExchange::new(),
            shuffler: ShuffleScheduler::new(),
            session_key: None,
            peer_public_key: None,
            next_seq: next_sequence(),
        }
    }

    pub fn start_key_exchange(&mut self) -> [u8; 32] {
        self.key_exchange.generate_keypair()
    }

    pub fn complete_key_exchange(&mut self, peer_public: &[u8; 32]) -> A2FResult<()> {
        let shared = self.key_exchange.compute_shared_secret(peer_public)?;
        let session_key = self.derive_session_key(&shared);
        self.session_key = Some(session_key);
        self.crypto.set_key(session_key);
        self.peer_public_key = Some(*peer_public);
        Ok(())
    }

    fn derive_session_key(&self, shared: &[u8; 32]) -> [u8; 32] {
        use sha2::Sha256;
        use hkdf::Hkdf;
        
        let hkdf = Hkdf::<Sha256>::new(None, shared);
        let mut okm = [0u8; 32];
        hkdf.expand(b"a2f-el-session", &mut okm).unwrap();
        okm
    }

    pub fn encrypt_data(&mut self, data: &[u8], timestamp: u64) -> A2FResult<Packet> {
        if self.session_key.is_none() {
            return Err(A2FError::ConfigError("鍵交換が完了していません".into()));
        }
        
        let encrypted = self.crypto.encrypt(data)?;
        let seq = self.next_seq;
        self.next_seq += 1;
        
        Ok(Packet::new(seq, timestamp, PayloadType::EncryptedData, encrypted))
    }

    pub fn make_key_packet(&mut self, timestamp: u64) -> A2FResult<Packet> {
        let public_key = self.start_key_exchange();
        let seq = self.next_seq;
        self.next_seq += 1;
        
        Ok(Packet::new(seq, timestamp, PayloadType::WrappedKey, public_key.to_vec()))
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
    peer_public_key: Option<[u8; 32]>,
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
            peer_public_key: None,
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
        Ok(())
    }

    fn derive_session_key(&self, shared: &[u8; 32]) -> [u8; 32] {
        use sha2::Sha256;
        use hkdf::Hkdf;
        
        let hkdf = Hkdf::<Sha256>::new(None, shared);
        let mut okm = [0u8; 32];
        hkdf.expand(b"a2f-el-session", &mut okm).unwrap();
        okm
    }

    pub fn receive_packet(&mut self, packet: Packet) -> A2FResult<Option<Vec<u8>>> {
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
                Ok(None)
            }
            PayloadType::EncryptedData => {
                if self.session_key.is_none() {
                    return Err(A2FError::ConfigError("鍵交換が完了していません".into()));
                }
                let decrypted = self.crypto.decrypt(&packet.payload)?;
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
}