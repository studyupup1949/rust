use serde::{Deserialize, Serialize};

use crate::store::{DbCollectionIden, StoreIden};

#[derive(Default, Deserialize, Serialize, Debug, Clone)]
pub struct Log {
    pub id: String,
    pub pid: String,
    pub nid: String,

    pub content: String,
    pub timestamp: i64,
}

impl DbCollectionIden for Log {
    fn iden() -> StoreIden {
        StoreIden::Logs
    }
}
