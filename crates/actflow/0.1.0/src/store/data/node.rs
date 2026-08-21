use serde::{Deserialize, Serialize};

use crate::store::{DbCollectionIden, StoreIden};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Node {
    pub id: String,
    pub pid: String,
    pub nid: String,

    pub state: String,
    pub err: Option<String>,
    pub start_time: i64,
    pub end_time: i64,
    pub timestamp: i64,
}

impl DbCollectionIden for Node {
    fn iden() -> StoreIden {
        StoreIden::Nodes
    }
}
