//! # actix-admin
//! 
//! A powerful admin panel library for Actix-web 4 applications that automatically generates CRUD interfaces.
//! 
//! ## Quick Start
//! 
//! ```rust,no_run
//! use actix_web_admin::{AdminRegistry, AdminSite, AdminResource, init_templates};
//! use async_trait::async_trait;
//! use std::collections::HashMap;
//! use std::sync::{Arc, Mutex};
//! 
//! #[actix_web::main]
//! async fn main() -> std::io::Result<()> {
//!     let templates = init_templates();
//!     let mut registry = AdminRegistry::new();
//!     // Register your resources...
//!     
//!     HttpServer::new(move || {
//!         App::new()
//!             .app_data(web::Data::new(templates.clone()))
//!             .configure(|cfg| {
//!                 AdminSite::new("/admin").mount(cfg, registry)
//!             })
//!     })
//!     .bind(("127.0.0.1", 8080))?
//!     .run()
//!     .await
//! }
//! ```

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
