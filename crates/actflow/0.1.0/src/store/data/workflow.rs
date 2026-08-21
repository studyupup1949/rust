use serde::{Deserialize, Serialize};

use crate::store::{DbCollectionIden, StoreIden};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub data: String,
    pub create_time: i64,
    pub update_time: i64,
}

impl DbCollectionIden for Workflow {
    fn iden() -> StoreIden {
        StoreIden::Workflows
    }
}
