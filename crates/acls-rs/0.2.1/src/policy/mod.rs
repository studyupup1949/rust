//! Policy evaluation engines for RBAC and ABAC.

pub mod abac;
pub mod conflict;
pub mod error;
pub mod limits;
pub mod rbac;

pub use abac::{AbacPolicy, AttributeContext, AttributePermission, AttributeRule};
pub use conflict::{ConflictResolver, JoinResolver, MeetResolver, PriorityResolver};
pub use error::PolicyError;
pub use limits::RuleLimitedPolicy;
pub use rbac::{RbacError, RbacPolicy, Role};
