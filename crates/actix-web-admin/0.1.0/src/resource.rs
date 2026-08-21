use async_trait::async_trait;
use std::collections::HashMap;
use crate::types::*;
use serde::Serialize;

/// Newtype for the admin site title to avoid Data<String> collisions.
#[derive(Clone)]
pub struct AdminTitle(pub String);

/// Newtype for the admin site prefix to avoid Data<String> collisions.
#[derive(Clone)]
pub struct AdminPrefix(pub String);

/// Information about a resource for serialization (e.g. for Tera templates).
#[derive(Serialize, Debug)]
pub struct ResourceInfo {
    pub name: String,
    pub plural_name: String,
    pub slug: String,
    pub icon: String,
    pub path_list: String,
    pub path_new: String,
    pub path_edit: String,
    pub path_delete: String,
}

/// Trait to be implemented by any resource that should be managed by the admin backoffice.
#[async_trait]
pub trait AdminResource: Send + Sync + 'static {
    /// Display name of the resource.
    fn name(&self) -> &str;
    /// Plural display name of the resource.
    fn plural_name(&self) -> &str;
    /// URL slug for the resource.
    fn slug(&self) -> &str;
    /// Icon for the resource in the dashboard (default: "box").
    fn icon(&self) -> &str { "box" }
    /// Columns to show in the list view.
    fn list_columns(&self) -> Vec<Column>;
    /// Fields to show in the create/edit form.
    fn form_fields(&self) -> Vec<FormField>;
    /// Fields that should be used for the global search.
    fn searchable_fields(&self) -> Vec<&str> { vec![] }
    /// Whether the resource can be created.
    fn can_create(&self) -> bool { true }
    /// Whether the resource can be deleted.
    fn can_delete(&self) -> bool { true }

    /// List records based on the provided query.
    async fn list(&self, query: ListQuery) -> Result<ListResult, AdminError>;
    /// Retrieve a single record by its ID.
    async fn get(&self, id: &str) -> Result<serde_json::Value, AdminError>;
    /// Create a new record.
    async fn create(&self, data: HashMap<String, serde_json::Value>) -> Result<serde_json::Value, AdminError>;
    /// Update an existing record.
    async fn update(&self, id: &str, data: HashMap<String, serde_json::Value>) -> Result<serde_json::Value, AdminError>;
    /// Delete a record.
    async fn delete(&self, id: &str) -> Result<(), AdminError>;
    
    /// Optional hook to validate data before saving.
    async fn validate(
        &self, 
        _data: &HashMap<String, serde_json::Value>
    ) -> Result<(), HashMap<String, String>> { Ok(()) }

    /// Return a serializable summary of the resource.
    fn info(&self, prefix: &str) -> ResourceInfo {
        ResourceInfo {
            name: self.name().to_string(),
            plural_name: self.plural_name().to_string(),
            slug: self.slug().to_string(),
            icon: self.icon().to_string(),
            path_list: format!("{}/{}/", prefix, self.slug()),
            path_new: format!("{}/{}/new", prefix, self.slug()),
            path_edit: format!("{}/{}/:id/edit", prefix, self.slug()),
            path_delete: format!("{}/{}/:id/delete", prefix, self.slug()),
        }
    }
}
