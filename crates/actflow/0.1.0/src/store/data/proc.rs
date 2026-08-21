use serde::{Deserialize, Serialize};

use crate::store::{DbCollectionIden, StoreIden};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Proc {
    pub id: String,
    pub wid: String,

    pub state: String,
    pub start_time: i64,
    pub end_time: i64,
    pub err: Option<String>,
    pub timestamp: i64,
}

impl DbCollectionIden for Proc {
    fn iden() -> StoreIden {
        StoreIden::Procs
    }
}
