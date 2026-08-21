use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use chacha20poly1305::ChaCha20Poly1305;
use rand::RngCore;
use rand::thread_rng;
use crate::error::{A2FError, A2FResult};

pub struct MultiLayerCrypto {
    master_key: [u8; 32],
}

impl MultiLayerCrypto {
    pub fn new(master_key: [u8; 32]) -> Self {
        Self { master_key }
    }
    
    /// セッション鍵をAES-256 + ChaCha20で多重ラップ
    pub fn wrap_session_key(&self, session_key: &[u8; 32]) -> A2FResult<Vec<u8>> {
        // レイヤー1: AES-256-GCM
        let aes_key = Key::<Aes256Gcm>::from_slice(&self.master_key);
        let aes_cipher = Aes256Gcm::new(aes_key);
        let mut aes_nonce = [0u8; 12];
        thread_rng().fill_bytes(&mut aes_nonce);
        
        let aes_encrypted = aes_cipher.encrypt(Nonce::from_slice(&aes_nonce), session_key.as_ref())
            .map_err(|e| A2FError::CryptoError(e.to_string()))?;
        
        // レイヤー2: ChaCha20-Poly1305
        let chacha_key = Key::<ChaCha20Poly1305>::from_slice(&self.master_key);
        let chacha_cipher = ChaCha20Poly1305::new(chacha_key);
        let mut chacha_nonce = [0u8; 12];
        thread_rng().fill_bytes(&mut chacha_nonce);
        
        // &Vec<u8> ではなく &[u8] として渡す
        let final_ciphertext = chacha_cipher.encrypt(Nonce::from_slice(&chacha_nonce), &aes_encrypted[..])
            .map_err(|e| A2FError::CryptoError(e.to_string()))?;
        
        // nonce + 暗号文
        let mut result = Vec::with_capacity(24 + final_ciphertext.len());
        result.extend_from_slice(&aes_nonce);
        result.extend_from_slice(&chacha_nonce);
        result.extend_from_slice(&final_ciphertext);
        
        Ok(result)
    }
    
    /// 多重ラップされたセッション鍵を展開
    pub fn unwrap_session_key(&self, wrapped: &[u8]) -> A2FResult<[u8; 32]> {
        if wrapped.len() < 24 {
            return Err(A2FError::DecryptionError("データが短すぎます".into()));
        }
        
        let aes_nonce = &wrapped[0..12];
        let chacha_nonce = &wrapped[12..24];
        let ciphertext = &wrapped[24..];
        
        // レイヤー2復号
        let chacha_key = Key::<ChaCha20Poly1305>::from_slice(&self.master_key);
        let chacha_cipher = ChaCha20Poly1305::new(chacha_key);
        let aes_encrypted = chacha_cipher.decrypt(Nonce::from_slice(chacha_nonce), ciphertext)
            .map_err(|e| A2FError::DecryptionError(e.to_string()))?;
        
        // レイヤー1復号
        let aes_key = Key::<Aes256Gcm>::from_slice(&self.master_key);
        let aes_cipher = Aes256Gcm::new(aes_key);
        let session_key_bytes = aes_cipher.decrypt(Nonce::from_slice(aes_nonce), &aes_encrypted[..])
            .map_err(|e| A2FError::DecryptionError(e.to_string()))?;
        
        if session_key_bytes.len() != 32 {
            return Err(A2FError::DecryptionError("鍵の長さが不正です".into()));
        }
        
        let mut result = [0u8; 32];
        result.copy_from_slice(&session_key_bytes);
        Ok(result)
    }
    
    /// データ暗号化（AES-256-GCM、セッション鍵使用）
    pub fn encrypt_data(&self, key: &[u8; 32], data: &[u8]) -> A2FResult<Vec<u8>> {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
        let mut nonce = [0u8; 12];
        thread_rng().fill_bytes(&mut nonce);
        
        let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce), data)
            .map_err(|e| A2FError::CryptoError(e.to_string()))?;
        
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }
    
    /// データ復号
    pub fn decrypt_data(&self, key: &[u8; 32], encrypted: &[u8]) -> A2FResult<Vec<u8>> {
        if encrypted.len() < 12 {
            return Err(A2FError::DecryptionError("データが短すぎます".into()));
        }
        
        let nonce = &encrypted[0..12];
        let ciphertext = &encrypted[12..];
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
        
        cipher.decrypt(Nonce::from_slice(nonce), ciphertext)
            .map_err(|e| A2FError::DecryptionError(e.to_string()))
    }
}