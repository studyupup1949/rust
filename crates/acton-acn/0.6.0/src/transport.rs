use crate::Result;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TransportType {
    Memory,
    Tcp,
    Bluetooth,
    Radio,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub peer_id: String,
    pub data: Vec<u8>,
    pub transport_type: TransportType,
}

pub trait Transport: Send + Sync {
    fn send(&mut self, peer_id: &str, data: &[u8]) -> Result<()>;
    fn receive(&mut self) -> Result<Option<Message>>;
    fn receive_blocking(&mut self) -> Result<Message>;
    fn peers(&self) -> Vec<String>;
    fn transport_type(&self) -> TransportType;
    fn local_id(&self) -> String;
}

pub struct MemoryTransport {
    local_id: String,
    queues: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, std::collections::VecDeque<Vec<u8>>>>>,
}

impl MemoryTransport {
    pub fn new(local_id: &str) -> Self {
        use std::collections::{HashMap, VecDeque};
        use std::sync::{Arc, Mutex};
        
        MemoryTransport {
            local_id: local_id.to_string(),
            queues: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Transport for MemoryTransport {
    fn send(&mut self, peer_id: &str, data: &[u8]) -> Result<()> {
        use std::collections::VecDeque;
        
        let mut queues = self.queues.lock().unwrap();
        let queue = queues.entry(peer_id.to_string()).or_insert(VecDeque::new());
        queue.push_back(data.to_vec());
        Ok(())
    }
    
    fn receive(&mut self) -> Result<Option<Message>> {
        let mut queues = self.queues.lock().unwrap();
        if let Some(queue) = queues.get_mut(&self.local_id) {
            if let Some(data) = queue.pop_front() {
                return Ok(Some(Message {
                    peer_id: "unknown".to_string(),
                    data,
                    transport_type: TransportType::Memory,
                }));
            }
        }
        Ok(None)
    }
    
    fn receive_blocking(&mut self) -> Result<Message> {
        loop {
            if let Some(msg) = self.receive()? {
                return Ok(msg);
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
    
    fn peers(&self) -> Vec<String> {
        let queues = self.queues.lock().unwrap();
        queues.keys().cloned().collect()
    }
    
    fn transport_type(&self) -> TransportType {
        TransportType::Memory
    }
    
    fn local_id(&self) -> String {
        self.local_id.clone()
    }
}