//! Predictive calculation for permission changes.
//!
//! This module provides tools for previewing permission changes before applying them,
//! tracking effects, and building complex permission modifications safely.

pub mod effects;
pub mod preview;

pub use effects::{PermissionEffect, PermissionEffectBuilder};
pub use preview::PermissionPreview;

use crate::permission::{GrantDenialPair, PermissionSet, Timestamp};

/// Trait for types that have permissions.
///
/// This enables generic preview operations across different types.
pub trait HasPermissions {
    /// Get the current permissions.
    fn permissions(&self) -> &GrantDenialPair;

    /// Get the current permissions mutably.
    fn permissions_mut(&mut self) -> &mut GrantDenialPair;

    /// Compute effective permissions (grants - denials).
    fn effective_permissions(&self) -> PermissionSet {
        self.permissions().effective_permissions()
    }

    /// Compute effective permissions at a specific time (if temporal permissions are supported).
    fn effective_permissions_at(&self, _time: Timestamp) -> PermissionSet {
        // Default implementation ignores time - override for temporal support
        self.effective_permissions()
    }
}
