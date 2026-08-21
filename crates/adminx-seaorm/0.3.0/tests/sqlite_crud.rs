// Exercises the full SeaORM Storage impl against a real (temp-file) SQLite DB:
// create → list/count → get → find_one_by → update → delete.

use adminx_core::storage::{QueryOptions, Storage};
use adminx_seaorm::connect;
use serde_json::{json, Map, Value};

fn tmp_url() -> String {
    // Unique-ish file per run without Date/rand (pid-based).
    let path = std::env::temp_dir().join(format!("adminx_seaorm_test_{}.db", std::process::id()));
    let _ = std::fs::remove_file(&path);
    format!("sqlite://{}?mode=rwc", path.display())
}

fn opts() -> QueryOptions {
    QueryOptions {
        page: 1,
        per_page: 25,
        sort_by: None,
        sort_desc: false,
        filters: Vec::new(),
    }
}

fn row(title: &str) -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("title".into(), json!(title));
    m
}

#[tokio::test]
async fn sqlite_crud_roundtrip() {
    let store = connect(&tmp_url()).await.expect("connect");
    store
        .execute_sql("CREATE TABLE items (id INTEGER PRIMARY KEY AUTOINCREMENT, title TEXT)")
        .await
        .expect("create table");

    store.create("items", row("alpha")).await.expect("create a");
    store.create("items", row("beta")).await.expect("create b");
    store.create("items", row("gamma")).await.expect("create c");

    // list + total
    let page = store.list("items", &opts()).await.expect("list");
    assert_eq!(page.rows.len(), 3, "should return all rows");
    assert_eq!(page.total, 3, "COUNT(*) must report 3, not 0");

    // get by id
    let got = store.get("items", "id", "2").await.expect("get");
    assert_eq!(got.unwrap()["title"], json!("beta"));

    // find_one_by
    let found = store
        .find_one_by("items", "title", "gamma")
        .await
        .expect("find");
    assert_eq!(found.unwrap()["id"], json!(3));

    // update
    let n = store
        .update("items", "id", "1", row("ALPHA"))
        .await
        .expect("update");
    assert_eq!(n, 1);
    assert_eq!(
        store.get("items", "id", "1").await.unwrap().unwrap()["title"],
        json!("ALPHA")
    );

    // delete
    let d = store.delete("items", "id", "2", false).await.expect("delete");
    assert_eq!(d, 1);
    assert_eq!(store.list("items", &opts()).await.unwrap().total, 2);
}
