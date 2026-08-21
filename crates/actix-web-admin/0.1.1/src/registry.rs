use crate::resource::AdminResource;
use std::collections::HashMap;
use std::sync::Arc;

/// Registry of all admin resources.
pub struct AdminRegistry {
    resources: HashMap<String, Arc<dyn AdminResource>>,
    order: Vec<String>,
}

impl AdminRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
            order: Vec::new(),
        }
    }

    /// Register a resource. Panics if a resource with the same slug already exists.
    pub fn register<R: AdminResource>(&mut self, resource: R) {
        let slug = resource.slug().to_string();
        if self.resources.contains_key(&slug) {
            panic!("Resource with slug {} already registered", slug);
        }
        self.resources.insert(slug.clone(), Arc::new(resource));
        self.order.push(slug);
    }

    /// Get a resource by its slug.
    pub fn get(&self, slug: &str) -> Option<Arc<dyn AdminResource>> {
        self.resources.get(slug).cloned()
    }

    /// Get all registered resources in the order they were registered.
    pub fn all(&self) -> Vec<Arc<dyn AdminResource>> {
        self.order
            .iter()
            .filter_map(|slug| self.resources.get(slug).cloned())
            .collect()
    }
}

pub type SharedRegistry = Arc<AdminRegistry>;
