use std::collections::VecDeque;
use crate::{Crystal, CrystalConfig, SessionKey, Identity, Packet, ActonError, Result};

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
    crystal: Crystal,
    session_key: SessionKey,
    state: SessionState,
    incoming_packets: VecDeque<Vec<(usize, usize, usize)>>,
    seq_send: u32,
    seq_recv: u32,
}

impl Session {
    pub fn initiate(identity: &Identity, peer_public_id: &str) -> Result<Self> {
        let crystal_seed = format!("{}_{}", identity.public_id, peer_public_id);
        let crystal = Crystal::new(crystal_seed.as_bytes(), CrystalConfig::default());
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
        let crystal = Crystal::new(crystal_seed.as_bytes(), CrystalConfig::default());
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
        let coords = self.crystal.encode(message);
        let mut packets = Vec::new();
        for chunk in coords.chunks(100) {
            let packet = Packet::new_data(&self.session_key, &self.session_id, self.seq_send, chunk);
            self.seq_send += 1;
            packets.push(packet);
        }
        Ok(packets)
    }

    pub fn decode_packet(&mut self, packet: &Packet) -> Result<Option<Vec<u8>>> {
        if packet.session_id != self.session_id {
            return Ok(None);
        }
        let coords = packet.decode_data(&self.session_key)?;
        self.incoming_packets.push_back(coords);
        self.try_assemble_message()
    }

    fn try_assemble_message(&mut self) -> Result<Option<Vec<u8>>> {
        let mut all_coords = Vec::new();
        for chunk in &self.incoming_packets {
            all_coords.extend(chunk);
        }
        if let Some(message) = self.crystal.decode(&all_coords, all_coords.len() / 8) {
            self.incoming_packets.clear();
            return Ok(Some(message));
        }
        Ok(None)
    }

    pub fn destroy(&mut self) -> Result<()> {
        self.crystal.wipe();
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