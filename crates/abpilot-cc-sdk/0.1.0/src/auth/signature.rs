use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

pub struct SignatureGenerator {
    secret: String,
}

impl SignatureGenerator {
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
        }
    }

    #[cfg(feature = "app")]
    pub fn generate_app_signature(&self, app_id: &str) -> (String, i64) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        let message = format!("{}{}", app_id, timestamp);
        let signature = self.sign(&message);
        
        (signature, timestamp)
    }

    #[cfg(feature = "app")]
    pub fn generate_world_signature(&self, world_id: &str) -> (String, i64) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        let message = format!("{}{}", world_id, timestamp);
        let signature = self.sign(&message);
        
        (signature, timestamp)
    }

    fn sign(&self, message: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(self.secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(message.as_bytes());
        let result = mac.finalize();
        hex::encode(result.into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "app")]
    fn test_signature_generation() {
        let generator = SignatureGenerator::new("test_secret");
        let (sig, ts) = generator.generate_app_signature("test_app_id");
        
        assert!(!sig.is_empty());
        assert!(ts > 0);
        assert_eq!(sig.len(), 64); // SHA256 hex is 64 chars
    }
}
