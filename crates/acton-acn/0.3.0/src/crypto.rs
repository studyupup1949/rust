use blake2::{Blake2b, Digest};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use x25519_dalek::{StaticSecret, PublicKey};
use ed25519_dalek::{Signer, Verifier, SigningKey, VerifyingKey, Signature};
use rand::Rng;
use sha2::Sha256;
use hkdf::Hkdf;

#[derive(Clone)]
pub struct Identity {
    pub public_id: String,
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
    static_secret: StaticSecret,
    static_public: PublicKey,
}

impl Identity {
    pub fn from_seed(seed_phrase: &str) -> Self {
        let mut hasher = Blake2b::new();
        hasher.input(seed_phrase.as_bytes());
        let result = hasher.result();
        let bytes = result.as_slice();
        
        let signing_key = SigningKey::from_bytes(&bytes[..32]).unwrap();
        let verifying_key = signing_key.verifying_key();
        
        let mut static_secret_bytes = [0u8; 32];
        static_secret_bytes.copy_from_slice(&bytes[32..64]);
        let static_secret = StaticSecret::from(static_secret_bytes);
        let static_public = PublicKey::from(&static_secret);
        
        let mut id_hasher = Blake2b::new();
        id_hasher.input(static_public.as_bytes());
        let id_result = id_hasher.result();
        let public_id = hex::encode(&id_result.as_slice()[..16]);
        
        Identity {
            public_id,
            signing_key,
            verifying_key,
            static_secret,
            static_public,
        }
    }

    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        self.signing_key.sign(data).to_bytes().to_vec()
    }

    pub fn verify(&self, data: &[u8], signature: &[u8]) -> bool {
        if let Ok(sig) = Signature::from_bytes(signature) {
            self.verifying_key.verify(data, &sig).is_ok()
        } else {
            false
        }
    }

    pub fn public_key(&self) -> Vec<u8> {
        self.static_public.as_bytes().to_vec()
    }
}

pub struct SessionKey {
    key: [u8; 32],
}

impl SessionKey {
    pub fn new(shared_secret: &[u8]) -> Self {
        let hkdf = Hkdf::<Sha256>::new(None, shared_secret);
        let mut key = [0u8; 32];
        hkdf.expand(b"acton_session", &mut key).unwrap();
        SessionKey { key }
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Vec<u8> {
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill(&mut nonce_bytes);
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&self.key));
        cipher.encrypt(Nonce::from_slice(&nonce_bytes), plaintext).unwrap()
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Option<Vec<u8>> {
        if ciphertext.len() < 12 {
            return None;
        }
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&self.key));
        cipher.decrypt(Nonce::from_slice(&ciphertext[..12]), &ciphertext[12..]).ok()
    }
}