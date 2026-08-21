#[cfg(feature = "serde_derive")]
use serde::{Serialize, Deserialize};

use crate::value::Value;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde_derive", derive(Serialize, Deserialize))]
pub struct Tile {
    pub possible: Vec<Value>
}