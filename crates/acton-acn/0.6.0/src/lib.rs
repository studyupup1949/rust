mod crystal;
mod crypto;
mod packet;
mod session;
mod transport;
mod utils;

pub use crystal::{AdaptiveCrystal, MessageCrystal, CrystalConfig};
pub use crypto::{Identity, SessionKey};
pub use packet::{Packet, PacketType};
pub use session::{Session, SessionState};
pub use transport::{Transport, TransportType, Message, MemoryTransport};

use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ActonError {
    #[error("Crypto error: {0}")]
    Crypto(String),
    #[error("Session error: {0}")]
    Session(String),
    #[error("Transport error: {0}")]
    Transport(String),
    #[error("Invalid packet")]
    InvalidPacket,
    #[error("Decode failed")]
    DecodeFailed,
    #[error("IO error: {0}")]
    Io(String),
}

pub type Result<T> = std::result::Result<T, ActonError>;

pub struct ActonClient {
    identity: Identity,
    sessions: HashMap<String, Session>,
    transport: Box<dyn Transport>,
}

impl ActonClient {
    pub fn new(seed_phrase: &str, transport: Box<dyn Transport>) -> Self {
        let identity = Identity::from_seed(seed_phrase);
        ActonClient {
            identity,
            sessions: HashMap::new(),
            transport,
        }
    }
    
    pub fn identity(&self) -> &Identity {
        &self.identity
    }
    
    pub fn initiate_session(&mut self, peer_public_id: &str) -> Result<&mut Session> {
        let session = Session::initiate(&self.identity, peer_public_id)?;
        let id = session.session_id();
        self.sessions.insert(id.clone(), session);
        Ok(self.sessions.get_mut(&id).unwrap())
    }
    
    pub fn accept_session(&mut self, peer_public_id: &str, shared_secret: &[u8]) -> Result<&mut Session> {
        let session = Session::accept(&self.identity, peer_public_id, shared_secret)?;
        let id = session.session_id();
        self.sessions.insert(id.clone(), session);
        Ok(self.sessions.get_mut(&id).unwrap())
    }
    
    pub fn send(&mut self, session_id: &str, message: &[u8]) -> Result<()> {
        let session = self.sessions.get_mut(session_id).ok_or(ActonError::Session("Session not found".into()))?;
        let packets = session.encode_message(message)?;
        for packet in packets {
            let data = packet.encode()?;
            let peer_id = session.peer_id();
            self.transport.send(&peer_id, &data)?;
        }
        Ok(())
    }
    
    pub fn receive(&mut self) -> Result<Vec<(String, Vec<u8>)>> {
        let mut results = Vec::new();
        while let Some(msg) = self.transport.receive()? {
            let packet = Packet::decode(&msg.data)?;
            for (session_id, session) in self.sessions.iter_mut() {
                if session.can_receive(&packet) {
                    if let Some(message) = session.decode_packet(&packet)? {
                        results.push((session_id.clone(), message));
                    }
                    break;
                }
            }
        }
        Ok(results)
    }
    
    pub fn destroy_session(&mut self, session_id: &str) -> Result<()> {
        if let Some(mut session) = self.sessions.remove(session_id) {
            session.destroy()?;
        }
        Ok(())
    }
    
    pub fn sessions(&self) -> Vec<String> {
        self.sessions.keys().cloned().collect()
    }
}