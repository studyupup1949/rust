// Verify the Axum router assembles without route conflicts. axum/matchit panics
// on overlapping routes at *registration* time (not compile time), so building
// the real router — with CRUD, UI, custom-action, and login routes — is the only
// way to catch a conflict.

use adminx_core::actions::{ActionFuture, CustomAction};
use adminx_core::prelude::*;
use adminx_core::response::ApiResponse;
use async_trait::async_trait;
use serde_json::{json, Value};

fn publish(_ctx: ReqCtx, id: String, _body: Value) -> ActionFuture {
    Box::pin(async move { ApiResponse::ok(json!({ "published": id })) })
}

#[derive(Clone)]
struct Post;

#[async_trait]
impl Resource for Post {
    fn resource_name(&self) -> &'static str {
        "Posts"
    }
    fn base_path(&self) -> &'static str {
        "posts"
    }
    fn table_name(&self) -> &'static str {
        "posts"
    }
    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
    fn custom_actions(&self) -> Vec<CustomAction> {
        vec![CustomAction::labeled("publish", "Publish", publish)]
    }
}

#[test]
fn router_builds_without_route_conflicts() {
    register_resource(Box::new(Post));
    // Panics here if any routes overlap (e.g. /view/:id vs /:id/action/:name).
    let _router = adminx_axum::router();
}
