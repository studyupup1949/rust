mod crc32;
mod murmur3;
mod sha256;

pub use crc32::hash as crc32;
pub use murmur3::hash as murmur3;
pub use sha256::hash as sha256;
