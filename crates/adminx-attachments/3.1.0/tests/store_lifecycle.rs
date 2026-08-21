// Pins the attachment store's lifecycle guarantees against an in-memory metadata
// table and a real on-disk blob store: re-uploading replaces (no accumulation,
// old bytes gone), and deleting a record purges every blob.
//
// This is the CI backstop for what was verified live against SeaORM + SQLite.

use adminx_attachments::{AttachmentStore, LocalFsStore};
use adminx_core::attach::{Attachments, UploadedFile};
use adminx_core::storage::{
    set_storage, CreateOutcome, ListPage, QueryOptions, Storage, StorageError,
};
use async_trait::async_trait;
use serde_json::{Map, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

/// A minimal in-memory `Storage` that actually round-trips rows — enough for the
/// attachment store's create / filtered-list / delete-by-id calls. (The audit
/// tests use a canned-response mock; this one needs real state.)
#[derive(Default)]
struct MemStore {
    rows: Mutex<Vec<Value>>,
    next_id: AtomicU64,
}

#[async_trait]
impl Storage for MemStore {
    async fn list(&self, _table: &str, opts: &QueryOptions) -> Result<ListPage, StorageError> {
        let rows = self.rows.lock().unwrap();
        let matched: Vec<Value> = rows
            .iter()
            .filter(|row| {
                // Only the equality filters the attachment store uses.
                opts.filters.iter().all(|f| {
                    row.get(&f.field).and_then(|v| v.as_str()) == Some(f.value.as_str())
                })
            })
            .cloned()
            .collect();
        let total = matched.len() as u64;
        Ok(ListPage { rows: matched, total })
    }

    async fn get(&self, _t: &str, _pk: &str, _id: &str) -> Result<Option<Value>, StorageError> {
        Ok(None)
    }

    async fn create(&self, _t: &str, data: Map<String, Value>) -> Result<CreateOutcome, StorageError> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst) + 1;
        let mut row = data;
        row.insert("id".into(), Value::Number(id.into()));
        self.rows.lock().unwrap().push(Value::Object(row));
        Ok(CreateOutcome { last_insert_id: Some(id.to_string()) })
    }

    async fn update(&self, _t: &str, _pk: &str, _i: &str, _d: Map<String, Value>) -> Result<u64, StorageError> {
        Ok(0)
    }

    async fn delete(&self, _t: &str, pk: &str, id: &str, _soft: bool) -> Result<u64, StorageError> {
        let mut rows = self.rows.lock().unwrap();
        let before = rows.len();
        rows.retain(|r| r.get(pk).map(|v| v.to_string().trim_matches('"').to_string()) != Some(id.to_string()));
        Ok((before - rows.len()) as u64)
    }

    async fn health(&self) -> bool {
        true
    }
}

fn upload(name: &str, body: &[u8]) -> UploadedFile {
    UploadedFile {
        filename: name.into(),
        content_type: "application/octet-stream".into(),
        bytes: body.to_vec(),
    }
}

fn count_files(dir: &std::path::Path) -> usize {
    fn walk(dir: &std::path::Path, acc: &mut usize) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for e in entries.flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, acc);
                } else {
                    *acc += 1;
                }
            }
        }
    }
    let mut n = 0;
    walk(dir, &mut n);
    n
}

#[tokio::test]
async fn reupload_replaces_and_delete_purges() {
    // A blob root unique to this test binary, cleaned up-front.
    let root = std::env::temp_dir().join("adminx_attachments_store_lifecycle");
    let _ = std::fs::remove_dir_all(&root);

    set_storage(Box::new(MemStore::default()));
    let store = AttachmentStore::new(Box::new(LocalFsStore::new(&root)));

    // First upload.
    let a1 = store.put("posts", "1", "cover", upload("first.bin", b"AAAA")).await.unwrap();
    assert_eq!(a1.byte_size, 4);
    assert_eq!(count_files(&root), 1, "one blob after first upload");
    assert_eq!(store.read(&a1.storage_key).await.unwrap(), b"AAAA");

    // Re-upload to the same field: replaces, does not accumulate.
    let a2 = store.put("posts", "1", "cover", upload("second.bin", b"BBBBBB")).await.unwrap();
    assert_ne!(a1.storage_key, a2.storage_key, "replacement gets a fresh key");
    let listed = store.list("posts", "1").await.unwrap();
    assert_eq!(listed.len(), 1, "still exactly one attachment for the field");
    assert_eq!(listed[0].filename, "second.bin");
    assert_eq!(count_files(&root), 1, "the old blob's bytes were removed");
    assert!(store.read(&a1.storage_key).await.is_err(), "old blob is gone");

    // A second field coexists.
    store.put("posts", "1", "banner", upload("b.bin", b"CC")).await.unwrap();
    assert_eq!(store.list("posts", "1").await.unwrap().len(), 2);
    assert_eq!(count_files(&root), 2);

    // delete_all (the purge path) removes every blob and row for the record.
    store.delete_all("posts", "1").await.unwrap();
    assert_eq!(store.list("posts", "1").await.unwrap().len(), 0, "no rows remain");
    assert_eq!(count_files(&root), 0, "no blobs remain on disk");

    let _ = std::fs::remove_dir_all(&root);
}
