use activitystreams_vocabulary::{impl_default, impl_display};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::Role;

/// Represents a filter key used for a [RoleFilter].
///
/// Used to define fine-grained role access to [Team](crate::Team) components.
#[derive(
    Clone, Copy, Debug, Eq, Hash, PartialEq, Ord, PartialOrd, Deserialize, Serialize, sqlx::Type,
)]
#[serde(rename_all = "camelCase")]
#[sqlx(type_name = "filter_key")]
pub enum FilterKey {
    #[sqlx(rename = "members")]
    Members,
    #[sqlx(rename = "parent")]
    Parent,
    #[sqlx(rename = "subteams")]
    Subteams,
    #[sqlx(rename = "oversees")]
    Oversees,
    #[sqlx(rename = "overseen_by")]
    OverseenBy,
    #[sqlx(rename = "project")]
    Project,
}

impl FilterKey {
    /// Represents the string for the [Members](Self::Members) variant.
    pub const MEMBERS: &str = "members";
    /// Represents the string for the [Parent](Self::Parent) variant.
    pub const PARENT: &str = "parent";
    /// Represents the string for the [Subteams](Self::Subteams) variant.
    pub const SUBTEAMS: &str = "subteams";
    /// Represents the string for the [Oversees](Self::Oversees) variant.
    pub const OVERSEES: &str = "oversees";
    /// Represents the string for the [OverseenBy](Self::OverseenBy) variant.
    pub const OVERSEEN_BY: &str = "overseen_by";
    /// Represents the string for the [Project](Self::Project) variant.
    pub const PROJECT: &str = "project";

    /// Creates a new [FilterKey].
    pub const fn new() -> Self {
        Self::Members
    }

    /// Gets the [FilterKey] string representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Members => Self::MEMBERS,
            Self::Parent => Self::PARENT,
            Self::Subteams => Self::SUBTEAMS,
            Self::Oversees => Self::OVERSEES,
            Self::OverseenBy => Self::OVERSEEN_BY,
            Self::Project => Self::PROJECT,
        }
    }
}

impl_default!(FilterKey);
impl_display!(FilterKey, str);

/// Represents a role-filter mapping for fine-tuned [Team](crate::Team) role provisioning.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(transparent)]
pub struct RoleFilter(HashMap<FilterKey, Role>);

impl RoleFilter {
    /// Creates a new [RoleFilter].
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Creates a new [RoleFilter] from a map.
    pub fn create<I: Into<HashMap<FilterKey, Role>>>(val: I) -> Self {
        Self(val.into())
    }

    /// Adds an entry to the [RoleFilter].
    ///
    /// If the `key` already exists, the current value is overwritten with `value`.
    pub fn add_filter(&mut self, key: FilterKey, value: Role) {
        self.0.insert(key, value);
    }

    /// Removes an entry from the [RoleFilter].
    pub fn remove_filter(&mut self, key: FilterKey) -> Option<Role> {
        self.0.remove(&key)
    }

    /// Gets an iterator over the [RoleFilter].
    pub fn iter(&self) -> impl Iterator<Item = (&FilterKey, &Role)> {
        self.0.iter()
    }
}

impl_default!(RoleFilter);
impl_display!(RoleFilter, json);
