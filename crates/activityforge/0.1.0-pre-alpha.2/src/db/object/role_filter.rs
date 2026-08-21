use activitystreams_vocabulary::{field_access, impl_default};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::db::{Iri, TableEntry, Uuid, util};
use crate::{Error, Result, Role, impl_sql_object};

/// Represents a role filter for a [Team](crate::db::Team).
///
/// Allows for delegating specific roles to different team members for different resources.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RoleFilter {
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    uuid: Uuid,
    id: Iri,
    key: TableEntry,
    value: Role,
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    team: Uuid,
}

impl RoleFilter {
    /// Creates a new [RoleFilter].
    pub const fn new() -> Self {
        Self {
            uuid: Uuid::nil(),
            id: Iri::new(),
            key: TableEntry::new(),
            value: Role::new(),
            team: Uuid::nil(),
        }
    }

    /// Performs checks for record invariants.
    pub fn check_db(&self) -> Result<()> {
        if self.key.is_empty() {
            Err(Error::db("role_filter: empty key entry"))
        } else if self.team.is_nil() {
            Err(Error::db("role_filter: nil team UUID"))
        } else {
            Ok(())
        }
    }
}

field_access! {
    RoleFilter {
        /// Represents the [Uuid] primary key of the table entry.
        uuid: Uuid,
        /// Represents a [Project](crate::db::Project) or sub-component [Team](crate::db::Team) resource.
        key: TableEntry,
        /// Represents the [Role] being delegated by the [Team](crate::db::Team).
        value: Role,
        /// Represents the [Uuid] primary key of the [Team](crate::db::Team).
        team: Uuid,
    }
}

field_access! {
    RoleFilter {
        /// Represents the IRI used to fetch the [RoleFilter].
        id: as_ref { Iri },
    }
}

impl_sql_object! {
    RoleFilter {
        id: { "id" Iri },
        key: { "key" TableEntry },
        value: { "value" Role },
        team: { "team" Uuid },
    }
}

impl_default!(RoleFilter);
