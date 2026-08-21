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
    pub fn new_data(session_key: &SessionKey, session_id: &str, seq: u32, coords: &[(usize, usize, usize)]) -> Self {
        let mut coords_bytes = Vec::new();
        for (x, y, z) in coords {
            coords_bytes.push((*x & 0xFF) as u8);
            coords_bytes.push((*y & 0xFF) as u8);
            coords_bytes.push((*z & 0xFF) as u8);
        }
        let encrypted = session_key.encrypt(&coords_bytes);
        Packet {
            session_id: session_id.to_string(),
            seq,
            encrypted_coords: encrypted,
            auth_tag: [0u8; 32],
            nonce: [0u8; 24],
        }
    }

    pub fn decode_data(&self, session_key: &SessionKey) -> Result<Vec<(usize, usize, usize)>, ActonError> {
        let decrypted = session_key.decrypt(&self.encrypted_coords)
            .ok_or(ActonError::InvalidPacket)?;
        let mut coords = Vec::new();
        for chunk in decrypted.chunks(3) {
            if chunk.len() == 3 {
                coords.push((chunk[0] as usize, chunk[1] as usize, chunk[2] as usize));
            }
        }
        Ok(coords)
    }

    pub fn encode(&self) -> Result<Vec<u8>, ActonError> {
        bincode::serialize(self).map_err(|_| ActonError::InvalidPacket)
    }

    pub fn decode(data: &[u8]) -> Result<Self, ActonError> {
        bincode::deserialize(data).map_err(|_| ActonError::InvalidPacket)
    }
}