use std::sync::Arc;
use actix_web::web::Data;
use crate::registry::ServiceRegistry;
use crate::service::Service;

#[derive(Clone)]
pub struct AppState {
    registry: Arc<ServiceRegistry>,
}

impl AppState {
    pub fn new(registry: ServiceRegistry) -> Self {
        Self {
            registry: Arc::new(registry),
        }
    }

    pub fn get<T: Service + 'static>(&self) -> Option<Arc<T>> {
        self.registry.get::<T>()
    }
}

// Convert AppState into actix_web::web::Data
impl From<AppState> for Data<AppState> {
    fn from(state: AppState) -> Self {
        Data::new(state)
    }
}