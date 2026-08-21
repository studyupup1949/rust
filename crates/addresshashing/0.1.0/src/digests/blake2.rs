use blake2::{Blake2sVar,Blake2sVarCore};
use digest::{Digest, Update, FixedOutput, VariableOutput};

pub struct Blake2sHasher;

impl Blake2sHasher {
    /// # Standard Address Scheme
    /// 
    /// This is the standard address scheme used in most cases.
    pub fn keser_standard(data: &[u8]) -> [u8; 28] {
        Self::new_224(data)
    }
    /// # Security Address Scheme
    pub fn keser_256(data: &[u8]) -> [u8; 32] {
        Self::new_256(data)
    }
    /// # Mid-Address Scheme
    pub fn keser_mid_240(data: &[u8]) -> [u8; 30] {
        Self::new_240(data)
    }
    pub fn new_160(data: &[u8]) -> [u8; 20] {
        let mut hasher = Blake2sVar::new(20).expect("Invalid output size");
        hasher.update(data);
        let result = hasher.finalize_boxed();
        let mut output = [0u8; 20];
        output.copy_from_slice(&result[..20]);
        output
    }
    pub fn new_176(data: &[u8]) -> [u8; 22] {
        let mut hasher = Blake2sVar::new(22).expect("Invalid output size");
        hasher.update(data);
        let result = hasher.finalize_boxed();
        let mut output = [0u8; 22];
        output.copy_from_slice(&result[..22]);
        output
    }
    pub fn new_190(data: &[u8]) -> [u8; 24] {
        let mut hasher = Blake2sVar::new(24).expect("Invalid output size");
        hasher.update(data);
        let result = hasher.finalize_boxed();
        let mut output = [0u8; 24];
        output.copy_from_slice(&result[..24]);
        output
    }
    pub fn new_208(data: &[u8]) -> [u8; 26] {
        let mut hasher = Blake2sVar::new(26).expect("Invalid output size");
        hasher.update(data);
        let result = hasher.finalize_boxed();
        let mut output = [0u8; 26];
        output.copy_from_slice(&result[..26]);
        output
    }
    /// # Default Address Scheme
    /// 
    /// This is the default address scheme
    pub fn new_224(data: &[u8]) -> [u8; 28] {
        let mut hasher = Blake2sVar::new(28).expect("Invalid output size");
        hasher.update(data);
        let result = hasher.finalize_boxed();
        let mut output = [0u8; 28];
        output.copy_from_slice(&result[..28]);
        output
    }
    /// # Secondary Address Scheme
    /// 
    /// This address scheme is used in special cases
    pub fn new_240(data: &[u8]) -> [u8; 30] {
        let mut hasher = Blake2sVar::new(30).expect("Invalid output size");
        hasher.update(data);
        let result = hasher.finalize_boxed();
        let mut output = [0u8; 30];
        output.copy_from_slice(&result[..30]);
        output
    }
    /// # Security Address Scheme
    /// 
    /// This address scheme offers the most security but has longer addresses.
    pub fn new_256(data: &[u8]) -> [u8; 32] {
        let mut hasher = Blake2sVar::new(32).expect("Invalid output size");
        hasher.update(data);
        let result = hasher.finalize_boxed();
        let mut output = [0u8; 32];
        output.copy_from_slice(&result[..32]);
        output
    }
    pub fn assert_length(data: &[u8], length: usize) -> bool {
        data.len() == length && length <= 32
    }
}

#[test]
fn run() {
    let data = b"hello world";
    let hash_224 = Blake2sHasher::new_224(data);
    println!("224-bit Hash: {:?}", hash_224);
}

