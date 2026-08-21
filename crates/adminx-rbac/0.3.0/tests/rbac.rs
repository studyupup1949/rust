// End-to-end RBAC: seed a central ability block into a mock DB, load the cache,
// then assert per-(role, action, resource) that the real Resource authorization
// path allows/denies correctly — custom actions, export, resource scoping, and
// the multi-role union.

mod common;

use adminx_core::authz::Action;
use adminx_core::prelude::*;
use adminx_rbac::Ability;
use common::{ctx_for, configure_test_auth, Mock, Posts, Reports};

#[tokio::test]
async fn db_grants_gate_every_action() {
    set_storage(Box::new(Mock::with(Vec::new())));
    configure_test_auth();

    adminx_rbac::init(vec![
        Ability::role("admin").can_manage_all(),
        Ability::role("editor")
            .can("list", "posts")
            .can("read", "posts")
            .can("create", "posts")
            .can("update", "posts")
            .can("publish", "posts"), // custom action, by name
        Ability::role("viewer").can_read_all(),
    ])
    .await
    .expect("rbac init");

    let posts = Posts;
    let reports = Reports;

    // admin: manage-all everywhere
    let admin = ctx_for("admin");
    for a in [
        Action::List,
        Action::Read,
        Action::Create,
        Action::Update,
        Action::Delete,
        Action::Export,
        Action::Custom("publish"),
    ] {
        assert!(posts.authorize(&admin, a), "admin allowed {a:?} on posts");
    }
    assert!(reports.authorize(&admin, Action::Delete), "admin manage-all covers reports");

    // editor: scoped to posts, specific actions
    let editor = ctx_for("editor");
    assert!(posts.authorize(&editor, Action::List));
    assert!(posts.authorize(&editor, Action::Create));
    assert!(posts.authorize(&editor, Action::Update));
    assert!(posts.authorize(&editor, Action::Custom("publish")));
    assert!(!posts.authorize(&editor, Action::Delete), "no delete grant");
    assert!(!posts.authorize(&editor, Action::Export), "no export grant");
    assert!(!posts.authorize(&editor, Action::Custom("archive")), "only 'publish' granted");
    assert!(!reports.authorize(&editor, Action::Read), "editor grants are posts-only");

    // viewer: read-only wildcard across resources
    let viewer = ctx_for("viewer");
    assert!(posts.authorize(&viewer, Action::Read));
    assert!(reports.authorize(&viewer, Action::Read), "'*' resource grant");
    assert!(!reports.authorize(&viewer, Action::Update), "read-only");

    // unknown role: nothing (restrictive allowed_roles fallback is bypassed)
    assert!(!posts.authorize(&ctx_for("ghost"), Action::Read));

    // end-to-end through real methods: status codes, not just the bool
    assert_eq!(posts.list(&editor).await.status, 200, "editor list -> 200");
    assert_eq!(posts.delete(&editor, "1").await.status, 401, "editor delete -> 401");
    assert_eq!(posts.delete(&admin, "1").await.status, 200, "admin delete -> 200");
    assert_eq!(reports.list_page(&editor).await.status, 303, "editor reports UI -> redirect");
}
