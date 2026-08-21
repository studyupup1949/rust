// adminx-storage/src/store.rs
//
// The `Attachments` implementation. It pairs a `BlobStore` (the bytes) with the
// `adminx_attachments` metadata table (written through adminx-core's `Storage`,
// so SeaORM and Mongo both work). Neither half names a concrete database or a
// concrete filesystem.

use crate::blobstore::BlobStore;
use adminx_core::attach::{Attachment, Attachments, UploadedFile};
use adminx_core::storage::{
    storage, FilterClause, FilterOp, QueryOptions, StorageError,
};
use async_trait::async_trait;
use serde_json::{Map, Value};

/// Metadata table name.
pub const TABLE: &str = "adminx_attachments";

/// Reasonable upper bound for a listing query — a record won't have hundreds of
/// attachments, and the metadata rows are tiny.
const LIST_CAP: u64 = 500;

/// Ties a blob store to the metadata table.
pub struct AttachmentStore {
    blobs: Box<dyn BlobStore>,
}

impl AttachmentStore {
    pub fn new(blobs: Box<dyn BlobStore>) -> Self {
        Self { blobs }
    }

    /// Rows for a record, optionally narrowed to one field.
    async fn rows(
        &self,
        owner_type: &str,
        owner_id: &str,
        field: Option<&str>,
    ) -> Result<Vec<Value>, StorageError> {
        let mut filters = vec![
            FilterClause {
                field: "owner_type".into(),
                op: FilterOp::Eq,
                value: owner_type.to_string(),
            },
            FilterClause {
                field: "owner_id".into(),
                op: FilterOp::Eq,
                value: owner_id.to_string(),
            },
        ];
        if let Some(f) = field {
            filters.push(FilterClause {
                field: "field".into(),
                op: FilterOp::Eq,
                value: f.to_string(),
            });
        }
        let opts = QueryOptions {
            page: 1,
            per_page: LIST_CAP,
            sort_by: Some("id".to_string()),
            sort_desc: false,
            filters,
        };
        Ok(storage().list(TABLE, &opts).await?.rows)
    }

    /// Delete a metadata row and its bytes. Bytes first: an orphaned metadata row
    /// is a visible, fixable inconsistency, whereas an orphaned blob is silent
    /// disk that nothing points at.
    async fn remove_row(&self, row: &Value) -> Result<(), StorageError> {
        if let Some(key) = row.get("storage_key").and_then(|v| v.as_str()) {
            self.blobs.delete(key).await?;
        }
        if let Some(id) = row_id(row) {
            storage().delete(TABLE, "id", &id, false).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl Attachments for AttachmentStore {
    async fn put(
        &self,
        owner_type: &str,
        owner_id: &str,
        field: &str,
        file: UploadedFile,
    ) -> Result<Attachment, StorageError> {
        // A field holds one file: clear whatever is there before writing the new
        // one, so re-uploading replaces rather than accumulates.
        for row in self.rows(owner_type, owner_id, Some(field)).await? {
            self.remove_row(&row).await?;
        }

        // Key layout mirrors the ownership path and ends in a random id, so two
        // uploads never collide and the on-disk tree is navigable.
        let storage_key = format!(
            "{owner_type}/{owner_id}/{field}/{}",
            nanoid::nanoid!()
        );
        self.blobs.put(&storage_key, &file.bytes).await?;

        let byte_size = file.bytes.len() as u64;
        let mut record = Map::new();
        record.insert("owner_type".into(), Value::String(owner_type.into()));
        record.insert("owner_id".into(), Value::String(owner_id.into()));
        record.insert("field".into(), Value::String(field.into()));
        record.insert("filename".into(), Value::String(file.filename.clone()));
        record.insert("content_type".into(), Value::String(file.content_type.clone()));
        record.insert("byte_size".into(), Value::Number(byte_size.into()));
        record.insert("storage_key".into(), Value::String(storage_key.clone()));
        record.insert(
            "created_at".into(),
            Value::String(chrono::Utc::now().to_rfc3339()),
        );

        // If the metadata write fails, don't leave the bytes stranded.
        if let Err(e) = storage().create(TABLE, record).await {
            let _ = self.blobs.delete(&storage_key).await;
            return Err(e);
        }

        Ok(Attachment {
            field: field.into(),
            filename: file.filename,
            content_type: file.content_type,
            byte_size,
            storage_key,
        })
    }

    async fn get(
        &self,
        owner_type: &str,
        owner_id: &str,
        field: &str,
    ) -> Result<Option<Attachment>, StorageError> {
        Ok(self
            .rows(owner_type, owner_id, Some(field))
            .await?
            .first()
            .map(to_attachment))
    }

    async fn list(
        &self,
        owner_type: &str,
        owner_id: &str,
    ) -> Result<Vec<Attachment>, StorageError> {
        Ok(self
            .rows(owner_type, owner_id, None)
            .await?
            .iter()
            .map(to_attachment)
            .collect())
    }

    async fn read(&self, storage_key: &str) -> Result<Vec<u8>, StorageError> {
        self.blobs.get(storage_key).await
    }

    async fn delete(
        &self,
        owner_type: &str,
        owner_id: &str,
        field: &str,
    ) -> Result<(), StorageError> {
        for row in self.rows(owner_type, owner_id, Some(field)).await? {
            self.remove_row(&row).await?;
        }
        Ok(())
    }

    async fn delete_all(&self, owner_type: &str, owner_id: &str) -> Result<(), StorageError> {
        for row in self.rows(owner_type, owner_id, None).await? {
            self.remove_row(&row).await?;
        }
        Ok(())
    }
}

/// A metadata row -> the neutral `Attachment` the panel renders.
fn to_attachment(row: &Value) -> Attachment {
    let s = |k: &str| row.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
    Attachment {
        field: s("field"),
        filename: s("filename"),
        content_type: s("content_type"),
        // Stored integer, but a JSON string over some backends — accept either.
        byte_size: row
            .get("byte_size")
            .and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
            .unwrap_or(0),
        storage_key: s("storage_key"),
    }
}

/// The primary key of a row, as a string, however the backend typed it.
fn row_id(row: &Value) -> Option<String> {
    match row.get("id") {
        Some(Value::Number(n)) => Some(n.to_string()),
        Some(Value::String(s)) => Some(s.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn row_maps_to_attachment_with_either_size_type() {
        let typed = json!({
            "field": "avatar", "filename": "a.png", "content_type": "image/png",
            "byte_size": 1234, "storage_key": "posts/1/avatar/x"
        });
        let a = to_attachment(&typed);
        assert_eq!(a.byte_size, 1234);
        assert_eq!(a.filename, "a.png");

        // Mongo/text backend may hand the number back as a string.
        let stringy = json!({ "byte_size": "1234", "storage_key": "k",
            "field": "f", "filename": "n", "content_type": "t" });
        assert_eq!(to_attachment(&stringy).byte_size, 1234);
    }

    #[test]
    fn row_id_survives_int_or_string_pk() {
        assert_eq!(row_id(&json!({"id": 5})).as_deref(), Some("5"));
        assert_eq!(row_id(&json!({"id": "abc"})).as_deref(), Some("abc"));
        assert_eq!(row_id(&json!({})), None);
    }
}
