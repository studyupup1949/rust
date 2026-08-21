// Re-export the Math Layer (KEM)
pub use abhedya_kem::*;

// Re-export the Encoding Layer (Chhandas) as 'sanskrit' module to maintain API compatibility
pub mod sanskrit {
    pub use abhedya_chhandas::*;
}
