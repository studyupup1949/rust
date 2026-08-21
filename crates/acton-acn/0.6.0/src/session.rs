use std::collections::VecDeque;
use crate::{AdaptiveCrystal, Identity, Packet, ActonError, Result};
use crate::crypto::SessionKey;

#[derive(PartialEq)]
pub enum SessionState {
    Init,
    Established,
    Destroyed,
}

pub struct Session {
    session_id: String,
    peer_id: String,
    peer_public_id: String,
    crystal: AdaptiveCrystal,
    session_key: SessionKey,
    state: SessionState,
    incoming_packets: VecDeque<Vec<u8>>,
    seq_send: u32,
    seq_recv: u32,
}

impl Session {
    pub fn initiate(identity: &Identity, peer_public_id: &str) -> Result<Self> {
        let crystal_seed = format!("{}_{}", identity.public_id, peer_public_id);
        let crystal = AdaptiveCrystal::new(crystal_seed.as_bytes());
        let session_id = format!("{}_{}", identity.public_id, peer_public_id);
        Ok(Session {
            session_id: session_id.clone(),
            peer_id: peer_public_id.to_string(),
            peer_public_id: peer_public_id.to_string(),
            crystal,
            session_key: SessionKey::new(&[0u8; 32]),
            state: SessionState::Init,
            incoming_packets: VecDeque::new(),
            seq_send: 0,
            seq_recv: 0,
        })
    }
    
    pub fn accept(identity: &Identity, peer_public_id: &str, shared_secret: &[u8]) -> Result<Self> {
        let crystal_seed = format!("{}_{}", identity.public_id, peer_public_id);
        let crystal = AdaptiveCrystal::new(crystal_seed.as_bytes());
        let session_id = format!("{}_{}", identity.public_id, peer_public_id);
        Ok(Session {
            session_id: session_id.clone(),
            peer_id: peer_public_id.to_string(),
            peer_public_id: peer_public_id.to_string(),
            crystal,
            session_key: SessionKey::new(shared_secret),
            state: SessionState::Established,
            incoming_packets: VecDeque::new(),
            seq_send: 0,
            seq_recv: 0,
        })
    }
    
    pub fn encode_message(&mut self, message: &[u8]) -> Result<Vec<Packet>> {
        if self.state == SessionState::Destroyed {
            return Err(ActonError::Session("Crystal destroyed".into()));
        }
        
        let (msg_id, coords, crystal_size) = self.crystal.encode(message);
        let data = (msg_id, coords, crystal_size);
        let serialized = bincode::serialize(&data).map_err(|_| ActonError::InvalidPacket)?;
        
        let encrypted = self.session_key.encrypt(&serialized);
        let packet = Packet::new_data(&self.session_key, &self.session_id, self.seq_send, &encrypted);
        self.seq_send += 1;
        
        Ok(vec![packet])
    }
    
    pub fn decode_packet(&mut self, packet: &Packet) -> Result<Option<Vec<u8>>> {
        if packet.session_id != self.session_id {
            return Ok(None);
        }
        
        let decrypted = self.session_key.decrypt(&packet.encrypted_coords)
            .ok_or(ActonError::InvalidPacket)?;
        
        let (msg_id, coords, crystal_size): (u64, Vec<usize>, usize) = 
            bincode::deserialize(&decrypted).map_err(|_| ActonError::InvalidPacket)?;
        
        if let Some(message) = self.crystal.decode(msg_id, &coords, crystal_size) {
            self.crystal.confirm(msg_id);
            return Ok(Some(message));
        }
        
        Ok(None)
    }
    
    pub fn destroy(&mut self) -> Result<()> {
        self.crystal.cleanup();
        self.state = SessionState::Destroyed;
        Ok(())
    }
    
    pub fn session_id(&self) -> String {
        self.session_id.clone()
    }
    
    pub fn peer_id(&self) -> String {
        self.peer_id.clone()
    }
    
    pub fn can_receive(&self, packet: &Packet) -> bool {
        packet.session_id == self.session_id
    }
}