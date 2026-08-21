//! # actix-web-admin
//!
//! A powerful admin panel library for Actix-web 4 with a pluggable authentication system.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use actix_web::{web, App, HttpServer};
//! use actix_session::storage::CookieSessionStore;
//! use actix_session::SessionMiddleware;
//! use actix_web_admin::auth::{UserStore, JsonUserStore};
//! use actix_web_admin::{AdminRegistry, AdminSite, init_templates};
//! use std::sync::Arc;
//!
//! #[actix_web::main]
//! async fn main() -> std::io::Result<()> {
//!     let templates = init_templates();
//!
//!     let store = Arc::new(JsonUserStore::new("users.json"));
//!     if store.find_by_username("admin").await.unwrap().is_none() {
//!         store.create_user("admin", "admin@example.com", "Admin", "admin", true).await.unwrap();
//!     }
//!
//!     HttpServer::new(move || {
//!         let mut registry = AdminRegistry::new();
//!         // Register your resources...
//!
//!         App::new()
//!             .app_data(web::Data::new(templates.clone()))
//!             .app_data(web::Data::new(store.clone() as Arc<dyn UserStore>))
//!             .wrap(SessionMiddleware::new(CookieSessionStore::default(), actix_web::cookie::Key::generate()))
//!             .configure(|cfg| {
//!                 AdminSite::new("/admin")
//!                     .with_user_store(store.clone())
//!                     .mount(cfg, registry)
//!             })
//!     })
//!     .bind(("127.0.0.1", 8080))?
//!     .run()
//!     .await
//! }
//! ```

pub mod auth;
pub mod cli;
pub mod resource;
pub mod registry;
pub mod site;
pub mod types;
pub mod handlers;

pub use resource::AdminResource;
pub use registry::AdminRegistry;
pub use site::AdminSite;

use tera::Tera;
use std::sync::Arc;

/// Helper to create a Tera instance with embedded templates.
pub fn init_templates() -> Tera {
    let mut tera = Tera::default();

    tera.add_raw_template("base.html", include_str!("templates/base.html")).unwrap();
    tera.add_raw_template("login.html", include_str!("templates/login.html")).unwrap();
    tera.add_raw_template("dashboard.html", include_str!("templates/dashboard.html")).unwrap();
    tera.add_raw_template("list.html", include_str!("templates/list.html")).unwrap();
    tera.add_raw_template("form.html", include_str!("templates/form.html")).unwrap();

    tera
}

pub type AdminTemplates = Arc<Tera>;
