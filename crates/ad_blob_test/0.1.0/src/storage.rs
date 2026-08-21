pub mod wasabi;

// Base trait that all Blob storage services must implement
pub trait BlobStorageBase {
    fn get(&self, bucket: &str, key: &str) -> Result<Vec<u8>, anyhow::Error>;
    fn put(&self, key: &str, bytes: Vec<u8>) -> Result<(), anyhow::Error>;
}
