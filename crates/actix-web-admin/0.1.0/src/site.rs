use crate::handlers;
use crate::registry::{AdminRegistry, SharedRegistry};
use crate::resource::{AdminPrefix, AdminTitle};
use actix_web::web;
use std::sync::Arc;

/// Main entry point for configuring the admin site.
pub struct AdminSite {
    prefix: String,
    title: String,
}

impl AdminSite {
    /// Create a new admin site with the given URL prefix.
    pub fn new(prefix: &str) -> Self {
        let prefix = prefix.trim_start_matches('/');
        Self {
            prefix: format!("/{}", prefix),
            title: "Admin".to_string(),
        }
    }

    /// Set the title of the admin site.
    pub fn title(mut self, t: &str) -> Self {
        self.title = t.to_string();
        self
    }

    /// Mount the admin site onto an Actix-Web application.
    pub fn mount(self, cfg: &mut web::ServiceConfig, registry: AdminRegistry) {
        let shared_registry: SharedRegistry = Arc::new(registry);
        let prefix = self.prefix.clone();
        let title = self.title.clone();

        cfg.app_data(web::Data::new(shared_registry.clone()));
        cfg.app_data(web::Data::new(AdminTitle(title)));
        cfg.app_data(web::Data::new(AdminPrefix(prefix.clone())));

        cfg.service(
            web::scope(&prefix)
                .route("", web::get().to(handlers::dashboard::index))
                .route("/", web::get().to(handlers::dashboard::index))
                .route("/login", web::get().to(handlers::auth::login_page))
                .route("/login", web::post().to(handlers::auth::login))
                .route("/logout", web::get().to(handlers::auth::logout))
                .service(
                    web::scope("/{slug}")
                        .route("", web::get().to(handlers::resource::list))
                        .route("/", web::get().to(handlers::resource::list))
                        .route("/new", web::get().to(handlers::resource::new))
                        .route("/new", web::post().to(handlers::resource::create))
                        .route("/{id}", web::get().to(handlers::resource::edit))
                        .route("/{id}", web::post().to(handlers::resource::update))
                        .route("/{id}/delete", web::post().to(handlers::resource::delete)),
                ),
        );
    }
}
