// Proves the search seam fires from default CRUD with the right index name, id
// and document, and that only declared `search_fields` are indexed. A recording
// Indexer captures what it's handed; a mock Storage returns a known row so the
// document (read back after a write) is deterministic.

use adminx_core::prelude::*;
use adminx_core::search::{set_indexer, Indexer};
use adminx_core::storage::{CreateOutcome, ListPage, QueryOptions, StorageError};
use async_trait::async_trait;
use serde_json::{json, Map, Value};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
enum Op {
    Index { index: String, id: String, doc: Map<String, Value> },
    Remove { index: String, id: String },
}

#[derive(Clone)]
struct Recorder {
    ops: Arc<Mutex<Vec<Op>>>,
}

#[async_trait]
impl Indexer for Recorder {
    async fn index(&self, index: &str, id: &str, document: Map<String, Value>) -> Result<(), StorageError> {
        self.ops.lock().unwrap().push(Op::Index {
            index: index.into(),
            id: id.into(),
            doc: document,
        });
        Ok(())
    }
    async fn remove(&self, index: &str, id: &str) -> Result<(), StorageError> {
        self.ops.lock().unwrap().push(Op::Remove {
            index: index.into(),
            id: id.into(),
        });
        Ok(())
    }
    async fn search(&self, _index: &str, _query: &str, _limit: usize) -> Result<Vec<String>, StorageError> {
        Ok(vec![])
    }
}

/// The row `get` returns, carrying more columns than are searchable — so the
/// test can prove only `search_fields` get indexed.
fn stored_row() -> Value {
    json!({"id": 7, "title": "Hello", "body": "World", "secret": "nope"})
}

struct Mock;

#[async_trait]
impl Storage for Mock {
    async fn list(&self, _t: &str, _o: &QueryOptions) -> Result<ListPage, StorageError> {
        Ok(ListPage { rows: vec![], total: 0 })
    }
    async fn get(&self, _t: &str, _pk: &str, _id: &str) -> Result<Option<Value>, StorageError> {
        Ok(Some(stored_row()))
    }
    async fn create(&self, _t: &str, _d: Map<String, Value>) -> Result<CreateOutcome, StorageError> {
        Ok(CreateOutcome { last_insert_id: Some("7".into()) })
    }
    async fn update(&self, _t: &str, _pk: &str, _i: &str, _d: Map<String, Value>) -> Result<u64, StorageError> {
        Ok(1)
    }
    async fn delete(&self, _t: &str, _pk: &str, _i: &str, _s: bool) -> Result<u64, StorageError> {
        Ok(1)
    }
    async fn health(&self) -> bool {
        true
    }
}

#[derive(Clone)]
struct Post;

#[async_trait]
impl Resource for Post {
    fn resource_name(&self) -> &'static str { "Posts" }
    fn base_path(&self) -> &'static str { "posts" }
    fn table_name(&self) -> &'static str { "posts" }
    fn clone_box(&self) -> Box<dyn Resource> { Box::new(self.clone()) }
    fn permit_keys(&self) -> Vec<&'static str> { vec!["title", "body"] }
    fn search_fields(&self) -> Vec<&'static str> { vec!["title", "body"] }
}

#[tokio::test]
async fn crud_keeps_the_index_in_sync() {
    let ops = Arc::new(Mutex::new(Vec::new()));
    set_indexer(Box::new(Recorder { ops: ops.clone() }));
    set_storage(Box::new(Mock));

    // Auth unconfigured: the access check allows through, so the indexing path
    // (what's under test) runs.
    let ctx = ReqCtx::new().with_mount("/adminx");
    let p = Post;
    let drain = || -> Vec<Op> {
        let mut o = ops.lock().unwrap();
        let seen = o.clone();
        o.clear();
        seen
    };

    // create -> index, with only the searchable fields, keyed by the insert id
    p.create(&ctx, json!({"title": "Hello", "body": "World"})).await;
    let seen = drain();
    assert_eq!(seen.len(), 1);
    match &seen[0] {
        Op::Index { index, id, doc } => {
            assert_eq!(index, "posts");
            assert_eq!(id, "7");
            assert_eq!(doc.get("title").unwrap(), "Hello");
            assert_eq!(doc.get("body").unwrap(), "World");
            assert!(doc.get("secret").is_none(), "only search_fields are indexed: {doc:?}");
            assert!(doc.get("id").is_none(), "the id is carried separately, not in the doc");
        }
        other => panic!("expected an index op, got {other:?}"),
    }

    // update -> re-index
    p.update(&ctx, "7", json!({"title": "Hi"})).await;
    let seen = drain();
    assert!(matches!(&seen[0], Op::Index { id, .. } if id == "7"), "update re-indexes: {seen:?}");

    // delete -> remove
    p.delete(&ctx, "7").await;
    let seen = drain();
    assert_eq!(seen.len(), 1);
    assert!(matches!(&seen[0], Op::Remove { index, id } if index == "posts" && id == "7"));

    // reads never touch the index
    p.get(&ctx, "7").await;
    p.list(&ctx).await;
    assert!(drain().is_empty(), "reads must not index or de-index");
}
