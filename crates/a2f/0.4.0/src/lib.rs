// ========== src/lib.rs ==========
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

struct SessionState {
    crypto: MultiLayerCrypto,
    buffer: TimestampBuffer,
    pending_keys: HashMap<u64, [u8; 32]>,
    sliding_window: SlidingWindow,
}

/// 送信側
pub struct A2FSender {
    session_id: u64,           
    crypto: MultiLayerCrypto,
    shuffler: ShuffleScheduler,
    current_session_key: Option<[u8; 32]>,
    next_seq: u64,
}

impl A2FSender {
    pub fn new(master_key: [u8; 32], config: &A2FConfig) -> Self {
        let session_id = rand::thread_rng().next_u64();
        Self {
            session_id,
            crypto: MultiLayerCrypto::new(master_key),
            shuffler: config.into_scheduler(),
            current_session_key: None,
            next_seq: next_sequence(),
        }
    }
    
    pub fn session_id(&self) -> u64 {
        self.session_id
    }
    
    pub fn set_next_sequence(&mut self, seq: u64) {
        self.next_seq = seq;
    }
    

    pub fn generate_key_packet(&mut self, timestamp: u64) -> A2FResult<Packet> {
        let mut session_key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut session_key);
        self.current_session_key = Some(session_key);
        
        let wrapped = self.crypto.wrap_session_key(&session_key)?;
        let seq = self.next_seq;
        self.next_seq += 1;
        
        Ok(Packet::new(self.session_id, seq, timestamp, PayloadType::WrappedKey, wrapped))
    }
    

    pub fn encrypt_data(&mut self, data: &[u8], timestamp: u64) -> A2FResult<Packet> {
        let session_key = self.current_session_key
            .ok_or_else(|| A2FError::ConfigError("セッション鍵が生成されていません".into()))?;
        
        let encrypted = self.crypto.encrypt_data(&session_key, data)?;
        let seq = self.next_seq;
        self.next_seq += 1;
        
        Ok(Packet::new(self.session_id, seq, timestamp, PayloadType::EncryptedData, encrypted))
    }

    pub fn send_data(&mut self, data: &[u8]) -> A2FResult<Vec<Packet>> {
        let ts = current_timestamp();
        let key_packet = self.generate_key_packet(ts)?;
        let data_packet = self.encrypt_data(data, ts)?;
        
        let mut packets = vec![key_packet, data_packet];
        packets = self.shuffle_packets(packets);
        
        Ok(packets)
    }

    pub fn send_multiple(&mut self, chunks: &[&[u8]]) -> A2FResult<Vec<Packet>> {
        if chunks.is_empty() {
            return Ok(vec![]);
        }
        
        let ts = current_timestamp();
        let mut all_packets = Vec::new();
        
        let key_packet = self.generate_key_packet(ts)?;
        all_packets.push(key_packet);
        
        for (i, chunk) in chunks.iter().enumerate() {
            let chunk_ts = ts + i as u64;
            let data_packet = self.encrypt_data(chunk, chunk_ts)?;
            all_packets.push(data_packet);
        }
        
        all_packets = self.shuffle_packets(all_packets);
        Ok(all_packets)
    }
    
    pub fn generate_dummy_packet(&mut self, timestamp: u64) -> Packet {
        let seq = self.next_seq;
        self.next_seq += 1;
        Packet::dummy(self.session_id, seq, timestamp)
    }
    
    pub fn generate_heartbeat_packet(&mut self, timestamp: u64) -> Packet {
        let seq = self.next_seq;
        self.next_seq += 1;
        Packet::heartbeat(self.session_id, seq, timestamp)
    }
    
    pub fn shuffle_packets<T>(&mut self, packets: Vec<T>) -> Vec<T> {
        self.shuffler.shuffle_packets(packets)
    }
}

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
    
    fn check_and_record(&mut self, seq: u64) -> bool {
        if seq < self.min_seq {
            return false;
        }
        
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
            return false;
        }
        
        self.received[index] = true;
        true
    }
    
    #[allow(dead_code)]
    fn get_min_seq(&self) -> u64 {
        self.min_seq
    }
}

pub struct A2FReceiver {
    master_key: [u8; 32],
    config: A2FConfig,
    sessions: HashMap<u64, SessionState>,  
}

impl A2FReceiver {
    pub fn new(master_key: [u8; 32], config: &A2FConfig) -> Self {
        Self {
            master_key,
            config: config.clone(),
            sessions: HashMap::new(),
        }
    }
    
    fn get_or_create_session(&mut self, session_id: u64) -> &mut SessionState {
        self.sessions.entry(session_id).or_insert_with(|| {
            let crypto = MultiLayerCrypto::new(self.master_key);
            let buffer = TimestampBuffer::new(self.config.buffer_timeout_secs, self.config.buffer_max_size);
            let sliding_window = SlidingWindow::new(self.config.replay_window_size);
            
            SessionState {
                crypto,
                buffer,
                pending_keys: HashMap::new(),
                sliding_window,
            }
        })
    }
    
    pub fn receive_packet(&mut self, packet: Packet) -> A2FResult<Option<Vec<u8>>> {
        let session_id = packet.session_id;
        let session = self.get_or_create_session(session_id);
        
        if !session.sliding_window.check_and_record(packet.seq) {
            return Err(A2FError::ExpiredSequence(packet.seq));
        }
        
        match packet.payload_type {
            PayloadType::WrappedKey => {
                let session_key = session.crypto.unwrap_session_key(&packet.payload)?;
                session.pending_keys.insert(packet.timestamp, session_key);
                
                if let Some((_, data)) = session.buffer.insert_key(packet.timestamp, packet.payload) {
                    let key = session.pending_keys.get(&packet.timestamp).unwrap();
                    return Ok(Some(session.crypto.decrypt_data(key, &data)?));
                }
            }
            PayloadType::EncryptedData => {
                if let Some(key) = session.pending_keys.get(&packet.timestamp) {
                    return Ok(Some(session.crypto.decrypt_data(key, &packet.payload)?));
                }
                
                if let Some((wrapped_key, data)) = session.buffer.insert_data(packet.timestamp, packet.payload) {
                    let key = session.crypto.unwrap_session_key(&wrapped_key)?;
                    session.pending_keys.insert(packet.timestamp, key);
                    return Ok(Some(session.crypto.decrypt_data(&key, &data)?));
                }
            }
            PayloadType::Dummy | PayloadType::Heartbeat => {}
        }
        
        Ok(None)
    }
    
    pub fn pending_count(&self) -> usize {
        self.sessions.values().map(|s| s.buffer.pending_count()).sum()
    }
    
    pub fn clear_expired(&mut self) -> usize {
        let mut total = 0;
        for session in self.sessions.values_mut() {
            total += session.buffer.clear_expired();
        }
        total
    }
    pub fn remove_session(&mut self, session_id: u64) -> Option<SessionState> {
        self.sessions.remove(&session_id)
    }
}