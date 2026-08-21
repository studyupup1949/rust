use bincode::Result;
use serde::{de::DeserializeOwned, Serialize};
mod utils;

#[derive(Debug, PartialEq, Clone)]
pub struct Value {
    data: Vec<u8>,
}

pub trait Tags {
    fn is_tag(&self, name: &str) -> bool {
        self.value(name).is_some()
    }
    fn value(&self, name: &str) -> Option<Value>;
    fn keys() -> Vec<String>;
}

impl Value {
    pub fn new(data: &[u8]) -> Self {
        Self {
            data: data.to_vec(),
        }
    }

    /// convert real type to Value
    pub fn from<T: Serialize>(v: &T) -> Result<Self> {
        let data = utils::to_bytes(v)?;
        Ok(Self { data })
    }

    /// convert Value to real type
    pub fn real<T>(&self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        utils::from_bytes(&self.data)
    }

    /// get the value binary data
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}
