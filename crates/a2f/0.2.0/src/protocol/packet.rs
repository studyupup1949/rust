// ========== packet.rs ==========
use serde::{Serialize, Deserialize};
use crate::error::A2FResult;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum PayloadType {
    WrappedKey,
    EncryptedData,
    Dummy,
    Heartbeat,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Packet {
    pub seq: u64,              // 平文シーケンス番号（リプレイ対策）
    pub timestamp: u64,        // 内部タイムスタンプ（暗号化対象）
    pub payload_type: PayloadType,
    pub payload: Vec<u8>,
}

impl Packet {
    pub fn new(seq: u64, timestamp: u64, payload_type: PayloadType, payload: Vec<u8>) -> Self {
        Self {
            seq,
            timestamp,
            payload_type,
            payload,
        }
    }
    
    pub fn dummy(seq: u64, timestamp: u64) -> Self {
        Self {
            seq,
            timestamp,
            payload_type: PayloadType::Dummy,
            payload: vec![],
        }
    }
    
    pub fn heartbeat(seq: u64, timestamp: u64) -> Self {
        Self {
            seq,
            timestamp,
            payload_type: PayloadType::Heartbeat,
            payload: vec![],
        }
    }
    
    pub fn serialize(&self) -> A2FResult<Vec<u8>> {
        bincode::serialize(self).map_err(|e| crate::error::A2FError::PacketError(e.to_string()))
    }
    
    pub fn deserialize(data: &[u8]) -> A2FResult<Self> {
        bincode::deserialize(data).map_err(|e| crate::error::A2FError::PacketError(e.to_string()))
    }
}