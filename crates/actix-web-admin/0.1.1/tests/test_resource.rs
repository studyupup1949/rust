use actix_web_admin::{AdminResource, types::*};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

struct MockResource {
    data: Arc<Mutex<Vec<serde_json::Value>>>,
}

#[async_trait]
impl AdminResource for MockResource {
    fn name(&self) -> &str { "Mock" }
    fn plural_name(&self) -> &str { "Mocks" }
    fn slug(&self) -> &str { "mock" }
    fn list_columns(&self) -> Vec<Column> { vec![Column::text("name", "Name")] }
    fn form_fields(&self) -> Vec<FormField> { vec![FormField::text("name", "Name")] }

    async fn list(&self, query: ListQuery) -> Result<ListResult, AdminError> {
        let db = self.data.lock().unwrap();
        let mut rows: Vec<_> = db.iter().cloned().collect();
        
        if let Some(ref s) = query.search {
            let s = s.to_lowercase();
            rows.retain(|r| {
                r["name"].as_str().map(|n| n.to_lowercase().contains(&s)).unwrap_or(false)
            });
        }

        let total = rows.len() as u64;
        let per_page = query.per_page.unwrap_or(10);
        let page = query.page.unwrap_or(1);
        let start = ((page - 1) * per_page) as usize;
        
        let rows = if start < rows.len() {
            let end = (start + per_page as usize).min(rows.len());
            rows[start..end].to_vec()
        } else {
            vec![]
        };

        Ok(ListResult { rows, total, page, per_page })
    }

    async fn get(&self, id: &str) -> Result<serde_json::Value, AdminError> {
        let db = self.data.lock().unwrap();
        db.iter()
            .find(|r| r["id"].as_str().unwrap_or("") == id)
            .cloned()
            .ok_or(AdminError::NotFound)
    }

    async fn create(&self, data: HashMap<String, serde_json::Value>) -> Result<serde_json::Value, AdminError> {
        if let Err(e) = self.validate(&data).await {
            return Err(AdminError::ValidationError(e));
        }
        let mut db = self.data.lock().unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        let mut record = data;
        record.insert("id".to_string(), serde_json::Value::String(id.clone()));
        let val = serde_json::to_value(&record).unwrap();
        db.push(val.clone());
        Ok(val)
    }

    async fn update(&self, id: &str, data: HashMap<String, serde_json::Value>) -> Result<serde_json::Value, AdminError> {
        let mut db = self.data.lock().unwrap();
        if let Some(record) = db.iter_mut().find(|r| r["id"].as_str().unwrap_or("") == id) {
            for (k, v) in data {
                record[k] = v;
            }
            return Ok(record.clone());
        }
        Err(AdminError::NotFound)
    }

    async fn delete(&self, id: &str) -> Result<(), AdminError> {
        let mut db = self.data.lock().unwrap();
        let len_before = db.len();
        db.retain(|r| r["id"].as_str().unwrap_or("") != id);
        if db.len() < len_before { Ok(()) } else { Err(AdminError::NotFound) }
    }

    async fn validate(&self, data: &HashMap<String, serde_json::Value>) -> Result<(), HashMap<String, String>> {
        if data.get("name").and_then(|v| v.as_str()).map(|s| s.is_empty()).unwrap_or(true) {
            let mut errs = HashMap::new();
            errs.insert("name".to_string(), "Name is required".to_string());
            return Err(errs);
        }
        Ok(())
    }
}

#[tokio::test]
async fn test_resource_lifecycle() {
    let resource = MockResource { data: Arc::new(Mutex::new(vec![])) };
    
    // Create
    let mut data = HashMap::new();
    data.insert("name".to_string(), serde_json::Value::String("Test".to_string()));
    let created = resource.create(data.clone()).await.unwrap();
    let id = created["id"].as_str().unwrap().to_string();

    // Get
    let got = resource.get(&id).await.unwrap();
    assert_eq!(got["name"], "Test");

    // Update
    let mut update_data = HashMap::new();
    update_data.insert("name".to_string(), serde_json::Value::String("Updated".to_string()));
    let updated = resource.update(&id, update_data).await.unwrap();
    assert_eq!(updated["name"], "Updated");

    // List
    let list = resource.list(ListQuery::default()).await.unwrap();
    assert_eq!(list.total, 1);
    assert_eq!(list.rows[0]["name"], "Updated");

    // Search
    let search_query = ListQuery { search: Some("Updated".to_string()), ..Default::default() };
    let search_res = resource.list(search_query).await.unwrap();
    assert_eq!(search_res.total, 1);

    let search_query_none = ListQuery { search: Some("None".to_string()), ..Default::default() };
    let search_res_none = resource.list(search_query_none).await.unwrap();
    assert_eq!(search_res_none.total, 0);

    // Delete
    resource.delete(&id).await.unwrap();
    assert!(resource.get(&id).await.is_err());
}

#[tokio::test]
async fn test_resource_validation() {
    let resource = MockResource { data: Arc::new(Mutex::new(vec![])) };
    let mut data = HashMap::new();
    data.insert("name".to_string(), serde_json::Value::String("".to_string()));
    
    let res = resource.create(data).await;
    assert!(matches!(res, Err(AdminError::ValidationError(_))));
}
