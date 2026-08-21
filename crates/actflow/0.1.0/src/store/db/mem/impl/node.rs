use std::collections::HashMap;

use serde_json::{Value as JsonValue, json};

use crate::{
    Result,
    store::{data::Node, db::mem::DbDocument},
};

impl DbDocument for Node {
    fn id(&self) -> &str {
        &self.id
    }
    fn doc(&self) -> Result<HashMap<String, JsonValue>> {
        let mut map = HashMap::new();
        map.insert("id".to_string(), json!(self.id.clone()));
        map.insert("pid".to_string(), json!(self.pid.clone()));
        map.insert("nid".to_string(), json!(self.nid.clone()));
        map.insert("state".to_string(), json!(self.state.clone()));
        map.insert("start_time".to_string(), json!(self.start_time));
        map.insert("end_time".to_string(), json!(self.end_time));
        map.insert("timestamp".to_string(), json!(self.timestamp));
        Ok(map)
    }
}
