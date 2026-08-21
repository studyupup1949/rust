use crate::Permission;

/// Permission builder
pub struct Builder {
    pub(crate) permissions: Vec<Box<dyn Permission + 'static>>,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            permissions: vec![],
        }
    }

    /// Appends item to permissions
    ///
    /// # Arguments
    /// * `permission` - permission
    pub fn and<P>(mut self, permission: P) -> Self
    where
        P: Permission + 'static,
    {
        self.permissions.push(Box::new(permission));
        Self {
            permissions: self.permissions,
        }
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}
