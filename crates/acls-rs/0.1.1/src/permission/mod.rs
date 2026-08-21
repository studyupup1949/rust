//! Permission types and structures.

pub mod atomic;
pub mod composite;
pub mod delta;
pub mod denial;
pub mod temporal;

pub use atomic::AtomicPermission;
pub use composite::PermissionSet;
pub use delta::PermissionDelta;
pub use denial::{DenialSet, GrantDenialPair};
pub use temporal::{
    current_timestamp_millis, TemporalPermission, TemporalPermissionSet, Timestamp,
};
