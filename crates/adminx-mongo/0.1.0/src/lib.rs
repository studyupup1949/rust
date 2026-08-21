// adminx-mongo/src/lib.rs
//
// MongoDB storage backend for adminx-core. Implements the same `Storage` trait
// the SeaORM backend does, so a resource written once runs unchanged on SQL or
// Mongo — the only difference is which `init` the app calls at startup.
//
// Note: Mongo's primary key is `_id`. A resource backed by Mongo should set
// `fn primary_key(&self) -> &'static str { "_id" }` so the UI builds correct
// links; this backend also maps the default `"id"` onto `_id` transparently.

mod convert;

use adminx_core::storage::{
    set_storage, CreateOutcome, FilterClause, FilterOp, ListPage, QueryOptions, Storage,
    StorageError,
};
use async_trait::async_trait;
use futures_util::stream::TryStreamExt;
use mongodb::bson::{doc, to_document, Bson, Document};
use mongodb::options::FindOptions;
use mongodb::{Client, Collection, Database};
use serde_json::{Map, Value};

use convert::{bson_to_json, doc_to_json, id_filter, json_map_to_doc};

/// MongoDB-backed storage. Cheap to clone (the client is `Arc` internally).
pub struct MongoStorage {
    db: Database,
}

impl MongoStorage {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    fn collection(&self, name: &str) -> Collection<Document> {
        self.db.collection::<Document>(name)
    }
}

/// Connect to MongoDB and return a ready storage handle.
pub async fn connect(uri: &str, db_name: &str) -> Result<MongoStorage, mongodb::error::Error> {
    let client = Client::with_uri_str(uri).await?;
    let db = client.database(db_name);
    tracing::info!("✅ adminx-mongo connected to database '{}'", db_name);
    Ok(MongoStorage::new(db))
}

/// Convenience: connect and register as the global adminx storage backend.
pub async fn init(uri: &str, db_name: &str) -> Result<(), mongodb::error::Error> {
    let storage = connect(uri, db_name).await?;
    set_storage(Box::new(storage));
    Ok(())
}

/// Connect and run a batch of **Mongo command documents** (JSON strings, e.g.
/// `{"insert":"products","documents":[{...}]}`), in order, returning the total
/// affected document count. Self-contained — no `set_storage` needed. Used by
/// `adminx seed` and by app startup code.
pub async fn seed(uri: &str, db_name: &str, statements: &[&str]) -> Result<u64, StorageError> {
    let store = connect(uri, db_name).await.map_err(map_err)?;
    let mut total = 0u64;
    for stmt in statements {
        total += store.execute_raw(stmt).await?;
    }
    Ok(total)
}

fn map_err(e: mongodb::error::Error) -> StorageError {
    StorageError::Backend(e.to_string())
}

#[async_trait]
impl Storage for MongoStorage {
    async fn list(&self, table: &str, opts: &QueryOptions) -> Result<ListPage, StorageError> {
        let collection = self.collection(table);
        let query = build_filter_doc(&opts.filters);

        let total = collection
            .count_documents(query.clone(), None)
            .await
            .map_err(map_err)?;

        let sort = opts.sort_by.as_ref().map(|col| {
            let dir = if opts.sort_desc { -1i32 } else { 1i32 };
            doc! { col: dir }
        });
        let find_options = FindOptions::builder()
            .skip(Some(opts.offset()))
            .limit(Some(opts.per_page as i64))
            .sort(sort)
            .build();

        let mut cursor = collection
            .find(query, find_options)
            .await
            .map_err(map_err)?;

        let mut rows = Vec::new();
        while let Some(doc) = cursor.try_next().await.map_err(map_err)? {
            rows.push(doc_to_json(doc));
        }

        Ok(ListPage { rows, total })
    }

    async fn get(&self, table: &str, pk: &str, id: &str) -> Result<Option<Value>, StorageError> {
        let doc = self
            .collection(table)
            .find_one(id_filter(pk, id), None)
            .await
            .map_err(map_err)?;
        Ok(doc.map(doc_to_json))
    }

    async fn find_one_by(
        &self,
        table: &str,
        column: &str,
        value: &str,
    ) -> Result<Option<Value>, StorageError> {
        let doc = self
            .collection(table)
            .find_one(doc! { column: value }, None)
            .await
            .map_err(map_err)?;
        Ok(doc.map(doc_to_json))
    }

    async fn create(
        &self,
        table: &str,
        data: Map<String, Value>,
    ) -> Result<CreateOutcome, StorageError> {
        let document = json_map_to_doc(data);
        let res = self
            .collection(table)
            .insert_one(document, None)
            .await
            .map_err(map_err)?;

        let last_insert_id = match bson_to_json(res.inserted_id) {
            Value::String(s) => Some(s),
            other => Some(other.to_string()),
        };
        Ok(CreateOutcome { last_insert_id })
    }

    async fn update(
        &self,
        table: &str,
        pk: &str,
        id: &str,
        data: Map<String, Value>,
    ) -> Result<u64, StorageError> {
        let set = json_map_to_doc(data);
        let res = self
            .collection(table)
            .update_one(id_filter(pk, id), doc! { "$set": set }, None)
            .await
            .map_err(map_err)?;
        Ok(res.modified_count)
    }

    async fn delete(
        &self,
        table: &str,
        pk: &str,
        id: &str,
        soft: bool,
    ) -> Result<u64, StorageError> {
        let collection = self.collection(table);
        let filter = id_filter(pk, id);

        if soft {
            let res = collection
                .update_one(filter, doc! { "$set": { "deleted": true } }, None)
                .await
                .map_err(map_err)?;
            Ok(res.modified_count)
        } else {
            let res = collection.delete_one(filter, None).await.map_err(map_err)?;
            Ok(res.deleted_count)
        }
    }

    async fn execute_raw(&self, statement: &str) -> Result<u64, StorageError> {
        // The statement is a JSON Mongo command document, e.g.
        // {"insert":"products","documents":[{...}]}.
        let value: Value = serde_json::from_str(statement)
            .map_err(|e| StorageError::Backend(format!("invalid JSON command: {e}")))?;
        let command = to_document(&value)
            .map_err(|e| StorageError::Backend(format!("command is not a document: {e}")))?;
        let res = self.db.run_command(command, None).await.map_err(map_err)?;
        // Write commands report the affected count in `n`.
        let n = res
            .get_i64("n")
            .ok()
            .or_else(|| res.get_i32("n").ok().map(|v| v as i64))
            .unwrap_or(0);
        Ok(n.max(0) as u64)
    }

    async fn health(&self) -> bool {
        self.db.run_command(doc! { "ping": 1 }, None).await.is_ok()
    }
}

/// Coerce a filter's text value into a BSON scalar: `true`/`false` → bool,
/// integer text → i64, everything else stays a string.
fn filter_bson(v: &str) -> Bson {
    match v {
        "true" => Bson::Boolean(true),
        "false" => Bson::Boolean(false),
        _ => match v.parse::<i64>() {
            Ok(i) => Bson::Int64(i),
            Err(_) => Bson::String(v.to_owned()),
        },
    }
}

/// Build a Mongo query document from column filters. `Eq` matches exactly;
/// `Contains` becomes a case-insensitive regex; `Gte`/`Lte` merge into a single
/// range sub-document per field (so a date range yields `{$gte, $lte}`).
fn build_filter_doc(filters: &[FilterClause]) -> Document {
    let mut doc = Document::new();
    for f in filters {
        match f.op {
            FilterOp::Eq => {
                doc.insert(f.field.clone(), filter_bson(&f.value));
            }
            FilterOp::Contains => {
                doc.insert(
                    f.field.clone(),
                    doc! { "$regex": f.value.clone(), "$options": "i" },
                );
            }
            FilterOp::Gte | FilterOp::Lte => {
                let key = if f.op == FilterOp::Gte { "$gte" } else { "$lte" };
                // Merge into any existing range sub-document on this field.
                let mut sub = doc
                    .get_document(&f.field)
                    .cloned()
                    .unwrap_or_default();
                sub.insert(key, filter_bson(&f.value));
                doc.insert(f.field.clone(), sub);
            }
        }
    }
    doc
}
