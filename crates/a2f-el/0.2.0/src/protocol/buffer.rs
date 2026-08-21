use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct PendingEntry {
    wrapped_key: Option<Vec<u8>>,
    encrypted_data: Option<Vec<u8>>,
    received_at: Instant,
}

pub struct TimestampBuffer {
    pending: HashMap<u64, PendingEntry>,
    timeout: Duration,
    max_size: usize,
}

impl TimestampBuffer {
    pub fn new(timeout_secs: u64, max_size: usize) -> Self {
        Self {
            pending: HashMap::new(),
            timeout: Duration::from_secs(timeout_secs),
            max_size,
        }
    }
    
    pub fn insert_key(&mut self, timestamp: u64, wrapped_key: Vec<u8>) -> Option<(Vec<u8>, Vec<u8>)> {
        self.cleanup();
        
        if self.pending.len() >= self.max_size {
            return None;
        }
        
        let entry = self.pending.entry(timestamp).or_insert(PendingEntry {
            wrapped_key: None,
            encrypted_data: None,
            received_at: Instant::now(),
        });
        entry.wrapped_key = Some(wrapped_key);
        
        self.try_complete(timestamp)
    }
    
    pub fn insert_data(&mut self, timestamp: u64, encrypted_data: Vec<u8>) -> Option<(Vec<u8>, Vec<u8>)> {
        self.cleanup();
        
        if self.pending.len() >= self.max_size {
            return None;
        }
        
        let entry = self.pending.entry(timestamp).or_insert(PendingEntry {
            wrapped_key: None,
            encrypted_data: None,
            received_at: Instant::now(),
        });
        entry.encrypted_data = Some(encrypted_data);
        
        self.try_complete(timestamp)
    }
    
    fn try_complete(&mut self, timestamp: u64) -> Option<(Vec<u8>, Vec<u8>)> {
        if let Some(entry) = self.pending.get(&timestamp) {
            if let (Some(key), Some(data)) = (&entry.wrapped_key, &entry.encrypted_data) {
                let result = Some((key.clone(), data.clone()));
                self.pending.remove(&timestamp);
                return result;
            }
        }
        None
    }
    
    fn cleanup(&mut self) {
        let now = Instant::now();
        self.pending.retain(|_, v| now.duration_since(v.received_at) < self.timeout);
    }
    
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
    
    pub fn clear_expired(&mut self) -> usize {
        let before = self.pending.len();
        self.cleanup();
        before - self.pending.len()
    }
}