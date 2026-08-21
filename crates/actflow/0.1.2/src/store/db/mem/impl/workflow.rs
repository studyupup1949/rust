use std::collections::HashMap;

use serde_json::{Value as JsonValue, json};

use crate::{
    Result,
    store::{data::Workflow, db::mem::DbDocument},
};

impl DbDocument for Workflow {
    fn id(&self) -> &str {
        &self.id
    }

    fn doc(&self) -> Result<HashMap<String, JsonValue>> {
        let mut map = HashMap::new();
        map.insert("id".to_string(), json!(self.id.clone()));
        map.insert("name".to_string(), json!(self.name.clone()));
        map.insert("desc".to_string(), json!(self.desc.clone()));
        map.insert("data".to_string(), json!(self.data.clone()));
        map.insert("create_time".to_string(), json!(self.create_time));
        map.insert("update_time".to_string(), json!(self.update_time));
        Ok(map)
    }
}
