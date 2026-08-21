#[cfg(feature = "serde_derive")]
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde_derive", derive(Serialize, Deserialize))]
pub struct Value {
    pub value: String,
    pub color: String,
    pub connectors: [Vec<String>; 4],
    pub disallow: Option<[Vec<String>; 4]>
}