//! Live check for the Mongo `execute_raw` / seed path: run a JSON `insert`
//! command, confirm the rows land, then clean up. Soft-skips when Mongo is
//! unreachable so it never breaks a Mongo-free run.
//!
//! Run: `cargo test -p adminx-mongo --test execute_raw` (needs local Mongo).

use adminx_core::storage::{QueryOptions, Storage};
use adminx_mongo::connect;

fn opts() -> QueryOptions {
    QueryOptions { page: 1, per_page: 50, sort_by: None, sort_desc: false, filters: Vec::new() }
}

#[tokio::test]
async fn seed_via_execute_raw_inserts_documents() {
    let uri = std::env::var("MONGO_URL").unwrap_or_else(|_| "mongodb://127.0.0.1:27017".into());
    let store = match connect(&uri, "adminx_seed_test").await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("skipping: Mongo unavailable ({e})");
            return;
        }
    };

    let clear = r#"{"delete":"seedcoll","deletes":[{"q":{},"limit":0}]}"#;
    if store.execute_raw(clear).await.is_err() {
        eprintln!("skipping: Mongo not writable");
        return;
    }

    // Seed two documents through the neutral raw API (a Mongo command document).
    let n = store
        .execute_raw(r#"{"insert":"seedcoll","documents":[{"name":"A","active":true},{"name":"B","active":false}]}"#)
        .await
        .expect("insert command");
    assert_eq!(n, 2, "insert should report 2 documents");

    // The records are now listable through the normal Storage API.
    let page = store.list("seedcoll", &opts()).await.expect("list");
    assert_eq!(page.total, 2);

    let _ = store.execute_raw(clear).await;
}
