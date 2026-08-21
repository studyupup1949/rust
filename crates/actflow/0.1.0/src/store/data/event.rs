use serde::{Deserialize, Serialize};

use crate::store::{DbCollectionIden, StoreIden};

#[derive(Default, Deserialize, Serialize, Debug, Clone)]
pub struct Event {
    pub id: String,
    pub pid: String,
    pub nid: String,
    pub name: String,
    pub message: String,

    pub timestamp: i64,
}

impl DbCollectionIden for Event {
    fn iden() -> StoreIden {
        StoreIden::Events
    }
}
