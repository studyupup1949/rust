use bincode::Result;
use serde::{de::DeserializeOwned, Serialize};
mod utils;

#[derive(Debug, PartialEq, Clone)]
pub struct Value {
    data: Vec<u8>,
}

impl Value {
    pub fn from<T: Serialize>(v: &T) -> Self {
        Self {
            data: utils::to_bytes(v).unwrap(),
        }
    }

    pub fn to<T>(&self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        utils::from_bytes(&self.data)
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}
