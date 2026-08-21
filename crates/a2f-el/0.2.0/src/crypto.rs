use chacha20poly1305::{
    ChaCha20Poly1305, Key, Nonce,
    aead::{Aead, KeyInit},
};
use x25519_dalek::{EphemeralSecret, PublicKey};
use rand::RngCore;
use rand::rngs::OsRng;
use crate::error::{A2FResult, A2FError};

pub struct SimpleCrypto {
    key: Option<[u8; 32]>,
}

impl SimpleCrypto {
    pub fn new() -> Self {
        Self { key: None }
    }

    pub fn set_key(&mut self, key: [u8; 32]) {
        self.key = Some(key);
    }

    pub fn encrypt(&self, data: &[u8]) -> A2FResult<Vec<u8>> {
        let key = self.key.as_ref().ok_or_else(|| 
            A2FError::ConfigError("暗号鍵が設定されていません".into()))?;
        
        let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce);
        
        let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce), data)
            .map_err(|e| A2FError::CryptoError(e.to_string()))?;
        
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    pub fn decrypt(&self, encrypted: &[u8]) -> A2FResult<Vec<u8>> {
        if encrypted.len() < 12 {
            return Err(A2FError::DecryptionError("データが短すぎます".into()));
        }
        
        let key = self.key.as_ref().ok_or_else(|| 
            A2FError::ConfigError("暗号鍵が設定されていません".into()))?;
        
        let nonce = &encrypted[0..12];
        let ciphertext = &encrypted[12..];
        let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
        
        cipher.decrypt(Nonce::from_slice(nonce), ciphertext)
            .map_err(|e| A2FError::DecryptionError(e.to_string()))
    }
}

pub struct KeyExchange {
    secret: Option<EphemeralSecret>,
    public: Option<[u8; 32]>,
    shared: Option<[u8; 32]>,
}

impl KeyExchange {
    pub fn new() -> Self {
        Self {
            secret: None,
            public: None,
            shared: None,
        }
    }

    pub fn generate_keypair(&mut self) -> [u8; 32] {
        let secret = EphemeralSecret::random_from_rng(OsRng);
        let public = PublicKey::from(&secret);
        let pub_bytes = public.to_bytes();
        self.secret = Some(secret);
        self.public = Some(pub_bytes);
        pub_bytes
    }

    pub fn compute_shared_secret(&mut self, peer_public: &[u8; 32]) -> A2FResult<[u8; 32]> {
        if self.secret.is_none() {
            self.generate_keypair();
        }
        
        let secret = self.secret.take().ok_or_else(|| 
            A2FError::ConfigError("鍵ペアの生成に失敗しました".into()))?;
        let peer_public = PublicKey::from(*peer_public);
        let shared = secret.diffie_hellman(&peer_public);
        let mut result = [0u8; 32];
        result.copy_from_slice(shared.as_bytes());
        self.shared = Some(result);
        Ok(result)
    }

    pub fn get_shared_secret(&self) -> Option<[u8; 32]> {
        self.shared
    }
}