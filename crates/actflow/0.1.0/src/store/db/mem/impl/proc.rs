use std::collections::HashMap;

use serde_json::{Value as JsonValue, json};

use crate::{
    Result,
    store::{data::Proc, db::mem::DbDocument},
};

impl DbDocument for Proc {
    fn id(&self) -> &str {
        &self.id
    }

    fn doc(&self) -> Result<HashMap<String, JsonValue>> {
        let mut map = HashMap::new();
        map.insert("id".to_string(), json!(self.id.clone()));
        map.insert("wid".to_string(), json!(self.wid.clone()));
        map.insert("state".to_string(), json!(self.state.clone()));
        map.insert("start_time".to_string(), json!(self.start_time));
        map.insert("end_time".to_string(), json!(self.end_time));
        map.insert("timestamp".to_string(), json!(self.timestamp));
        Ok(map)
    }
}
