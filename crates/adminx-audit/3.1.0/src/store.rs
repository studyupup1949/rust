// adminx-audit/src/store.rs
//
// The `Auditor` implementation: turn an `AuditEntry` into a row and write it
// through adminx-core's `Storage`, so the same code serves SeaORM (SQL) and
// MongoDB without naming either.

use adminx_core::audit::{AuditEntry, Auditor};
use adminx_core::storage::{storage, FilterClause, FilterOp, QueryOptions, StorageError};
use async_trait::async_trait;
use serde_json::{Map, Value};

/// Where audit rows are written. Exposed so a deployment can point the log at a
/// separate table (or a separate database, via a storage backend of its own).
pub const TABLE: &str = "adminx_audit_versions";

/// Writes audit entries through the globally registered `Storage`.
pub struct StorageAuditor {
    strict: bool,
}

impl StorageAuditor {
    pub fn new(strict: bool) -> Self {
        Self { strict }
    }

    /// Flatten an entry into the column map the storage layer inserts.
    ///
    /// `changes` is serialized to a JSON *string* rather than nested as a JSON
    /// value: it is the one representation both a SQL `TEXT`/`JSONB` column and
    /// a Mongo document accept unchanged, and it keeps the row shape flat for
    /// the generic list/filter machinery the panel reuses.
    fn row(entry: &AuditEntry) -> Map<String, Value> {
        let mut row = Map::new();
        row.insert("item_type".into(), Value::String(entry.item_type.clone()));
        row.insert("item_id".into(), Value::String(entry.item_id.clone()));
        row.insert(
            "event".into(),
            Value::String(entry.event.as_str().to_string()),
        );
        row.insert(
            "whodunnit".into(),
            entry
                .whodunnit
                .clone()
                .map(Value::String)
                .unwrap_or(Value::Null),
        );
        row.insert(
            "whodunnit_email".into(),
            entry
                .whodunnit_email
                .clone()
                .map(Value::String)
                .unwrap_or(Value::Null),
        );
        row.insert(
            "changes".into(),
            Value::String(
                serde_json::to_string(&Value::Object(entry.changes.clone()))
                    .unwrap_or_else(|_| "{}".into()),
            ),
        );
        // Recorded here rather than left to a column default, so the timestamp
        // is identical across backends and doesn't depend on DB clock config.
        row.insert(
            "created_at".into(),
            Value::String(chrono::Utc::now().to_rfc3339()),
        );
        row
    }
}

#[async_trait]
impl Auditor for StorageAuditor {
    async fn record(&self, entry: AuditEntry) -> Result<(), StorageError> {
        storage().create(TABLE, Self::row(&entry)).await?;
        Ok(())
    }

    /// Newest first, which for an append-only table is descending primary key —
    /// stable even when two entries share a `created_at` timestamp.
    async fn history(
        &self,
        item_type: &str,
        item_id: &str,
        limit: u64,
    ) -> Result<Vec<Value>, StorageError> {
        let opts = QueryOptions {
            page: 1,
            per_page: limit,
            sort_by: Some("id".to_string()),
            sort_desc: true,
            filters: vec![
                FilterClause {
                    field: "item_type".into(),
                    op: FilterOp::Eq,
                    value: item_type.to_string(),
                },
                FilterClause {
                    field: "item_id".into(),
                    op: FilterOp::Eq,
                    value: item_id.to_string(),
                },
            ],
        };
        Ok(storage().list(TABLE, &opts).await?.rows)
    }

    fn strict(&self) -> bool {
        self.strict
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adminx_core::audit::Event;
    use adminx_core::request::{Claims, ReqCtx};
    use serde_json::json;

    fn entry() -> AuditEntry {
        let ctx = ReqCtx::new().with_claims(Claims {
            sub: "7".into(),
            email: "admin@example.com".into(),
            ..Default::default()
        });
        let mut changes = Map::new();
        changes.insert("title".into(), json!(["Old", "New"]));
        AuditEntry::new(&ctx, "posts", "42", Event::Update, changes)
    }

    #[test]
    fn row_carries_the_actor_and_the_target() {
        let row = StorageAuditor::row(&entry());
        assert_eq!(row["item_type"], json!("posts"));
        assert_eq!(row["item_id"], json!("42"));
        assert_eq!(row["event"], json!("update"));
        assert_eq!(row["whodunnit"], json!("7"));
        assert_eq!(row["whodunnit_email"], json!("admin@example.com"));
    }

    #[test]
    fn changes_serialize_to_a_json_string() {
        let row = StorageAuditor::row(&entry());
        let text = row["changes"].as_str().expect("changes must be a string");
        let parsed: Value = serde_json::from_str(text).expect("must be valid JSON");
        assert_eq!(parsed["title"], json!(["Old", "New"]));
    }

    #[test]
    fn an_anonymous_actor_is_null_not_empty() {
        // Auth unconfigured: the log should say "nobody was identified", not
        // record an empty string that reads like a real user id.
        let ctx = ReqCtx::new();
        let e = AuditEntry::new(&ctx, "posts", "1", Event::Create, Map::new());
        let row = StorageAuditor::row(&e);
        assert_eq!(row["whodunnit"], Value::Null);
        assert_eq!(row["whodunnit_email"], Value::Null);
    }

    #[test]
    fn created_at_is_rfc3339() {
        let row = StorageAuditor::row(&entry());
        let at = row["created_at"].as_str().unwrap();
        assert!(
            chrono::DateTime::parse_from_rfc3339(at).is_ok(),
            "not RFC3339: {at}"
        );
    }
}
