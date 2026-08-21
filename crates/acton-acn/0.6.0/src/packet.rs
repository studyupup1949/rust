use serde::{Serialize, Deserialize};
use crate::crypto::SessionKey;
use crate::ActonError;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Packet {
    pub session_id: String,
    pub seq: u32,
    pub encrypted_coords: Vec<u8>,
    pub auth_tag: [u8; 32],
    pub nonce: [u8; 24],
}

pub enum PacketType {
    Data(Vec<Packet>),
    Control(Vec<u8>),
}

impl Packet {
    pub fn new_data(session_key: &SessionKey, session_id: &str, seq: u32, encrypted_data: &[u8]) -> Self {
        Packet {
            session_id: session_id.to_string(),
            seq,
            encrypted_coords: encrypted_data.to_vec(),
            auth_tag: [0u8; 32],
            nonce: [0u8; 24],
        }
    }
    
    pub fn decode_data(&self, session_key: &SessionKey) -> Result<Vec<u8>, ActonError> {
        session_key.decrypt(&self.encrypted_coords)
            .ok_or(ActonError::InvalidPacket)
    }
    
    pub fn encode(&self) -> Result<Vec<u8>, ActonError> {
        bincode::serialize(self).map_err(|_| ActonError::InvalidPacket)
    }
    
    pub fn decode(data: &[u8]) -> Result<Self, ActonError> {
        bincode::deserialize(data).map_err(|_| ActonError::InvalidPacket)
    }
}