use bincode::Result;
use serde::{de::DeserializeOwned, Serialize};

pub fn to_bytes<T: Serialize>(t: &T) -> Result<Vec<u8>> {
    bincode::serialize(t)
}

pub fn from_bytes<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    bincode::deserialize(bytes)
}
