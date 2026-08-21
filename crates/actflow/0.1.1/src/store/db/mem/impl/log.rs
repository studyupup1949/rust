use std::collections::HashMap;

use serde_json::{Value as JsonValue, json};

use crate::{
    Result,
    store::{data::Log, db::mem::DbDocument},
};

impl DbDocument for Log {
    fn id(&self) -> &str {
        &self.id
    }

    fn doc(&self) -> Result<HashMap<String, JsonValue>> {
        let mut map = HashMap::new();
        map.insert("id".to_string(), json!(self.id.clone()));
        map.insert("pid".to_string(), json!(self.pid.clone()));
        map.insert("nid".to_string(), json!(self.nid.clone()));
        map.insert("content".to_string(), json!(self.content.clone()));
        map.insert("timestamp".to_string(), json!(self.timestamp));
        Ok(map)
    }
}
