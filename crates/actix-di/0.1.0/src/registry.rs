use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info};
use crate::error::ServiceError;
use crate::service::{Service, DependencyProvider};

#[derive(Default)]
pub struct ServiceRegistry {
    services: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    initialization_order: Vec<TypeId>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
            initialization_order: Vec::new(),
        }
    }

    pub fn register<T: Service + DependencyProvider + 'static>(
        &mut self,
        service: Arc<T>
    ) -> Result<(), ServiceError> {
        let type_id = TypeId::of::<T>();
        debug!("Registering service: {:?}", type_id);

        // Check dependencies
        for dep_type in T::required_services() {
            if !self.services.contains_key(&dep_type) {
                error!("Missing dependency: {:?}", dep_type);
                return Err(ServiceError::MissingDependency(dep_type));
            }
        }

        self.services.insert(type_id, Box::new(service));
        self.initialization_order.push(type_id);
        info!("Service registered successfully: {:?}", type_id);
        Ok(())
    }

    pub fn get<T: Service + 'static>(&self) -> Option<Arc<T>> {
        self.services
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<Arc<T>>())
            .cloned()
    }

    pub async fn init_all(&self) -> Result<(), ServiceError> {
        for type_id in &self.initialization_order {
            debug!("Initializing service: {:?}", type_id);
            if let Some(service) = self.services.get(type_id) {
                let service = service.downcast_ref::<Arc<dyn Service>>()
                    .ok_or_else(|| ServiceError::InitializationError(
                        "Invalid service type".to_string()
                    ))?;
                service.init().await?;
            }
        }
        info!("All services initialized successfully");
        Ok(())
    }

    pub async fn shutdown_all(&self) -> Result<(), ServiceError> {
        for type_id in self.initialization_order.iter().rev() {
            debug!("Shutting down service: {:?}", type_id);
            if let Some(service) = self.services.get(type_id) {
                let service = service.downcast_ref::<Arc<dyn Service>>()
                    .ok_or_else(|| ServiceError::ShutdownError(
                        "Invalid service type".to_string()
                    ))?;
                service.shutdown().await?;
            }
        }
        info!("All services shut down successfully");
        Ok(())
    }
}