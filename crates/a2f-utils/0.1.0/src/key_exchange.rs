use x25519_dalek::{EphemeralSecret, PublicKey};
use rand::rngs::OsRng;
use hkdf::Hkdf;
use sha2::Sha256;
use crate::{UtilsError, UtilsResult};

/// X25519鍵交換
pub struct KeyExchange {
    secret: Option<EphemeralSecret>,
    public_key: Option<[u8; 32]>,
    shared_secret: Option<[u8; 32]>,
}

impl KeyExchange {
    pub fn new() -> Self {
        Self {
            secret: None,
            public_key: None,
            shared_secret: None,
        }
    }

    /// 鍵ペアを生成し、公開鍵を返す
    pub fn generate_keypair(&mut self) -> [u8; 32] {
        let secret = EphemeralSecret::random_from_rng(OsRng);
        let public = PublicKey::from(&secret);
        let pub_bytes = public.to_bytes();
        self.secret = Some(secret);
        self.public_key = Some(pub_bytes);
        pub_bytes
    }

    /// 相手の公開鍵から共有秘密を計算
    pub fn compute_shared_secret(&mut self, peer_public: &[u8; 32]) -> UtilsResult<[u8; 32]> {
        let secret = self.secret.take()
            .ok_or_else(|| UtilsError::KeyExchangeError("鍵ペアが生成されていません".into()))?;
        
        let peer_public = PublicKey::from(*peer_public);
        let shared = secret.diffie_hellman(&peer_public);
        let mut result = [0u8; 32];
        result.copy_from_slice(shared.as_bytes());
        self.shared_secret = Some(result);
        Ok(result)
    }

    /// 共有秘密からセッション鍵を導出
    pub fn derive_session_key(&self, shared: &[u8; 32]) -> [u8; 32] {
        let hkdf = Hkdf::<Sha256>::new(None, shared);
        let mut okm = [0u8; 32];
        hkdf.expand(b"a2f-session-key", &mut okm).unwrap();
        okm
    }

    pub fn get_public_key(&self) -> Option<[u8; 32]> {
        self.public_key
    }

    pub fn get_shared_secret(&self) -> Option<[u8; 32]> {
        self.shared_secret
    }
}

impl Default for KeyExchange {
    fn default() -> Self {
        Self::new()
    }
}