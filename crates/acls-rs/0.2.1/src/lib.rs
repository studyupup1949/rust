#![warn(missing_docs)]
//! Algebraically-correct permissions system with RBAC, ABAC, and temporal support.

pub mod algebra;
pub mod calculation;
pub mod permission;
pub mod policy;
pub mod resource;
pub mod subject;
pub mod sync;

pub use resource::Resource;
pub use subject::{BuilderError, Subject};

/// Commonly used types and traits, re-exported for convenience.
pub mod prelude {
    pub use crate::algebra::{
        JoinSemilattice, Lattice, MeetSemilattice, Monoid, MonoidAction, Semigroup,
    };
    pub use crate::calculation::{
        HasPermissions, PermissionEffect, PermissionEffectBuilder, PermissionPreview,
    };
    pub use crate::permission::{
        AtomicPermission, DenialSet, GrantDenialPair, PermissionDelta, PermissionMapping,
        PermissionSet, TemporalPermission, TemporalPermissionSet,
    };
    pub use crate::policy::{
        AbacPolicy, AttributeContext, AttributePermission, AttributeRule, ConflictResolver,
        JoinResolver, MeetResolver, PriorityResolver, RbacError, RbacPolicy, Role,
    };
    pub use crate::resource::Resource;
    pub use crate::subject::Subject;
}
