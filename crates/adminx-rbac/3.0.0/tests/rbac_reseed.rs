// Separate binary (process-global singletons): with grants already in the DB, a
// fresh `init` must NOT re-seed over them — the database is authoritative once
// populated.

mod common;

use adminx_core::authz::Action;
use adminx_core::prelude::*;
use adminx_rbac::Ability;
use common::{ctx_for, configure_test_auth, Mock, Posts};
use serde_json::json;

#[tokio::test]
async fn init_does_not_reseed_a_populated_table() {
    let preexisting = vec![json!({"id": 1, "role": "editor", "resource": "posts", "action": "read"})];
    set_storage(Box::new(Mock::with(preexisting)));
    configure_test_auth();

    // This admin block WOULD grant manage-all — but the table is non-empty, so
    // seeding is skipped and only the pre-existing editor:read survives.
    adminx_rbac::init(vec![Ability::role("admin").can_manage_all()])
        .await
        .expect("rbac init");

    let posts = Posts;
    assert!(
        posts.authorize(&ctx_for("editor"), Action::Read),
        "the pre-existing grant is loaded into the cache"
    );
    assert!(
        !posts.authorize(&ctx_for("admin"), Action::Read),
        "the admin block must NOT have been seeded over existing rows"
    );
}
