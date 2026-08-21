// EL版 src/protocol/packet.rs
use serde::{Serialize, Deserialize};
use crate::error::{A2FError, A2FResult};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum PayloadType {
    WrappedKey,
    EncryptedData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Packet {
    pub session_id: u64, 
    pub seq: u64,
    pub timestamp: u64,
    pub payload_type: PayloadType,
    pub payload: Vec<u8>,
}

impl Packet {
    pub fn new(session_id: u64, seq: u64, timestamp: u64, payload_type: PayloadType, payload: Vec<u8>) -> Self {
        Self {
            session_id,
            seq,
            timestamp,
            payload_type,
            payload,
        }
    }
    
    pub fn serialize(&self) -> A2FResult<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| A2FError::PacketError(format!("シリアライズ失敗: {}", e)))
    }
    
    pub fn deserialize(data: &[u8]) -> A2FResult<Self> {
        bincode::deserialize(data)
            .map_err(|e| A2FError::PacketError(format!("デシリアライズ失敗: {} (データ長: {})", e, data.len())))
    }
}