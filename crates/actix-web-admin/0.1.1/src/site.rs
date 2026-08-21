use crate::auth::UserStore;
use crate::handlers;
use crate::registry::{AdminRegistry, SharedRegistry};
use crate::resource::{AdminPrefix, AdminTitle};
use actix_web::web;
use std::sync::Arc;

/// Main entry point for configuring the admin site.
pub struct AdminSite {
    prefix: String,
    title: String,
    user_store: Option<Arc<dyn UserStore>>,
}

impl AdminSite {
    /// Create a new admin site with the given URL prefix.
    pub fn new(prefix: &str) -> Self {
        let prefix = prefix.trim_start_matches('/');
        Self {
            prefix: format!("/{}", prefix),
            title: "Admin".to_string(),
            user_store: None,
        }
    }

    /// Set the title of the admin site.
    pub fn title(mut self, t: &str) -> Self {
        self.title = t.to_string();
        self
    }

    /// Provide a UserStore for authentication (required for login to work).
    pub fn with_user_store(mut self, store: Arc<dyn UserStore>) -> Self {
        self.user_store = Some(store);
        self
    }

    /// Mount the admin site onto an Actix-Web application.
    pub fn mount(self, cfg: &mut web::ServiceConfig, registry: AdminRegistry) {
        let shared_registry: SharedRegistry = Arc::new(registry);
        let prefix = self.prefix.clone();
        let title = self.title.clone();

        cfg.app_data(web::Data::new(shared_registry.clone()));
        cfg.app_data(web::Data::new(AdminTitle(title)));
        cfg.app_data(web::Data::new(AdminPrefix(prefix.trim_start_matches('/').to_string())));

        if let Some(ref store) = self.user_store {
            cfg.app_data(web::Data::new(store.clone()));
        } else {
            // No UserStore provided — inject a no-op store that rejects all logins
            log::warn!(
                "No UserStore configured for admin site at {}. \
                 Login will reject all attempts. \
                 Use AdminSite::with_user_store() to provide one.",
                prefix
            );
            let noop: Arc<dyn UserStore> = Arc::new(NoopUserStore);
            cfg.app_data(web::Data::new(noop));
        }

        cfg.service(
            web::scope(&prefix)
                .wrap(crate::auth::RequireAuth::new(&prefix))
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

struct NoopUserStore;

#[async_trait::async_trait]
impl UserStore for NoopUserStore {
    async fn find_by_username(&self, _: &str) -> Result<Option<crate::auth::User>, crate::auth::AuthError> {
        Ok(None)
    }

    async fn create_user(
        &self,
        _: &str,
        _: &str,
        _: &str,
        _: &str,
        _: bool,
    ) -> Result<crate::auth::User, crate::auth::AuthError> {
        Err(crate::auth::AuthError::Storage("NoopUserStore: cannot create users".to_string()))
    }

    async fn delete_user(&self, _: &str) -> Result<(), crate::auth::AuthError> {
        Err(crate::auth::AuthError::Storage("NoopUserStore: cannot delete users".to_string()))
    }
}
