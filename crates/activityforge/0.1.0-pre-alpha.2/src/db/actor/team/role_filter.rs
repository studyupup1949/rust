use activitystreams_vocabulary::{field_access, impl_default, impl_display};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{FilterKey, Role, RoleFilter as VocabRoleFilter};

/// Represents a role-filter mapping for fine-tuned [Team](crate::Team) role provisioning.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize, sqlx::Type)]
#[sqlx(type_name = "role_filter")]
pub struct RoleFilter {
    key: FilterKey,
    value: Role,
}

impl RoleFilter {
    /// Creates a new [RoleFilter].
    pub const fn new() -> Self {
        Self {
            key: FilterKey::new(),
            value: Role::new(),
        }
    }

    /// Creates a new [RoleFilter] with the provided key-value pair.
    pub const fn create(key: FilterKey, value: Role) -> Self {
        Self { key, value }
    }
}

field_access! {
    RoleFilter {
        /// Represents the [Team](super::Team) subcomponent or `Project` being filtered.
        key: FilterKey,
        /// Represents the fine-tuned access role for the [Team](super::Team) subcomponent.
        value: Role,
    }
}

impl_default!(RoleFilter);
impl_display!(RoleFilter, json);

impl From<&[RoleFilter]> for VocabRoleFilter {
    fn from(val: &[RoleFilter]) -> Self {
        Self::create(
            val.iter()
                .map(|f| (f.key, f.value))
                .collect::<HashMap<_, _>>(),
        )
    }
}

impl From<Vec<RoleFilter>> for VocabRoleFilter {
    fn from(val: Vec<RoleFilter>) -> Self {
        val.as_slice().into()
    }
}

impl From<&Vec<RoleFilter>> for VocabRoleFilter {
    fn from(val: &Vec<RoleFilter>) -> Self {
        val.as_slice().into()
    }
}

impl<const N: usize> From<&[RoleFilter; N]> for VocabRoleFilter {
    fn from(val: &[RoleFilter; N]) -> Self {
        val.as_ref().into()
    }
}

impl<const N: usize> From<[RoleFilter; N]> for VocabRoleFilter {
    fn from(val: [RoleFilter; N]) -> Self {
        val.as_ref().into()
    }
}

impl From<VocabRoleFilter> for Vec<RoleFilter> {
    fn from(val: VocabRoleFilter) -> Self {
        (&val).into()
    }
}

impl From<&VocabRoleFilter> for Vec<RoleFilter> {
    fn from(val: &VocabRoleFilter) -> Self {
        val.iter()
            .map(|(&key, &value)| RoleFilter { key, value })
            .collect()
    }
}
