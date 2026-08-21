use activitystreams_vocabulary::{Iri, Item, impl_default, impl_display};
use serde::{Deserialize, Serialize};

use crate::db::Iri as DbIri;
use crate::{Error, Result};

/// Represents a role that an actor has within a [Team](crate::Team), or a role defining the level of access an actor has to a resource.
///
/// # Example
///
/// ```rust
/// use activityforge::Role;
///
/// # fn main() {
/// [
///     (Role::Visit, Role::VISIT),
///     (Role::Report, Role::REPORT),
///     (Role::Triage, Role::TRIAGE),
///     (Role::Write, Role::WRITE),
///     (Role::Maintain, Role::MAINTAIN),
///     (Role::Admin, Role::ADMIN),
///     (Role::Delegate, Role::DELEGATE),
///     (Role::Public, Role::PUBLIC),
/// ]
/// .into_iter()
/// .for_each(|(cap, cap_str)| {
///     let json_str = format!(r#""{cap_str}""#);
///
///     assert_eq!(serde_json::to_string_pretty(&cap).unwrap(), json_str);
///     assert_eq!(
///         serde_json::from_str::<Role>(json_str.as_str()).unwrap(),
///         cap,
///     );
/// });
/// # }
/// ```
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize, sqlx::Type,
)]
#[serde(rename_all = "camelCase")]
#[sqlx(type_name = "role")]
pub enum Role {
    /// Represents that the resource is public to all actors, unless a `Block` activity exists.
    Public,
    /// Authorizes the [Grant](crate::Grant) recipient (i.e. `target`) to view the [Grant](crate::Grant) resource (i.e. context), which includes retrieving objects via HTTP and pulling/cloning VCS repos.
    Visit,
    /// Authorizes the Grant recipient (i.e. target) to do on the Grant resource (i.e. context) anything that the visit role authorizes.
    ///
    /// Open an issue, submit a PR, create comments and discussion threads, edit public wikis, submit PR reviews.
    Report,
    /// Authorizes the [Grant](crate::Grant) recipient (i.e. `target`) to do on the [Grant](crate::Grant) resource (i.e. context) anything that the report role authorizes.
    ///
    /// Also to:
    ///
    /// - edit issue/PR propeties (labels, milestones, due dates, etc.)
    /// - close and reopen issues and PRs
    /// - assign and unassign people to issues and PRs
    /// - request PR reviews
    /// - hide disruptive comments (a moderation action)
    /// - lock and move discussions
    Triage,
    /// Authorizes the [Grant](crate::Grant) recipient (i.e. `target`) to do on the [Grant](crate::Grant) resource (i.e. `context`) anything that the triage role authorizes.
    ///
    /// Also to:
    ///
    /// - apply PR suggested changes
    /// - edit non-public wikis
    /// - create/edit/delete labels
    /// - merge a PR
    /// - push to VCS repositories
    /// - create/edit/run/cancel CI recipes
    /// - manage releases
    /// - publish packages
    /// - create web IDE coding sessions.
    Write,
    /// Authorizes the [Grant](crate::Grant) recipient (i.e. `target`) to do on the [Grant](crate::Grant) resource (i.e. context) anything that the write role authorizes.
    ///
    /// Also to:
    ///
    /// - edit project
    /// - edit component descriptions
    /// - edit settings unrelated to access
    /// - enable/disable components
    /// - configure "Pages" publishing of static websites from repos
    /// - push to repository's protected branches
    Maintain,
    /// Authorizes the [Grant](crate::Grant) recipient (i.e. `target`) to do on the [Grant](crate::Grant) resource (i.e. `context`) anything that the maintain role authorizes.
    ///
    ///
    /// Also to:
    ///
    /// - manage access to projects
    /// - components and teams
    /// - merge PRs even without reviews
    /// - delete issues
    /// - change project/component/team visibility
    /// - edit project/component/team access-related settings
    /// - change a repository’s default branch
    /// - manage webhooks and deployment
    /// - move components and projects between projects
    /// - archive projects/components
    /// - delete components/projects/teams
    Admin,
    /// Authorizes the [Grant](crate::Grant) recipient (i.e. `target`) to send access delegations to the [Grant](crate::Grant) sender (i.e. `actor`).
    Delegate,
    /// Represents denial to the resource for any actor.
    Deny,
}

impl Role {
    /// Represents the [Visit](Self::Visit) string.
    pub const VISIT: &str = "visit";
    /// Represents the [Report](Self::Report) string.
    pub const REPORT: &str = "report";
    /// Represents the [Triage](Self::Triage) string.
    pub const TRIAGE: &str = "triage";
    /// Represents the [Write](Self::Write) string.
    pub const WRITE: &str = "write";
    /// Represents the [Maintain](Self::Maintain) string.
    pub const MAINTAIN: &str = "maintain";
    /// Represents the [Admin](Self::Admin) string.
    pub const ADMIN: &str = "admin";
    /// Represents the [Delegate](Self::Delegate) string.
    pub const DELEGATE: &str = "delegate";
    /// Represents the [Public](Self::Public) string.
    pub const PUBLIC: &str = "public";
    /// Represents the [Deny](Self::Deny) string.
    pub const DENY: &str = "deny";

    /// Creates a new [Role].
    pub const fn new() -> Self {
        Self::Triage
    }

    /// Gets the [Role] string representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Visit => Self::VISIT,
            Self::Report => Self::REPORT,
            Self::Triage => Self::TRIAGE,
            Self::Write => Self::WRITE,
            Self::Maintain => Self::MAINTAIN,
            Self::Admin => Self::ADMIN,
            Self::Delegate => Self::DELEGATE,
            Self::Public => Self::PUBLIC,
            Self::Deny => Self::DENY,
        }
    }
}

impl_default!(Role);
impl_display!(Role, str);

impl From<Role> for Item {
    fn from(val: Role) -> Self {
        (&val).into()
    }
}

impl From<&Role> for Item {
    fn from(val: &Role) -> Self {
        Self::iri(Iri::try_from(val.as_str()).unwrap_or_default())
    }
}

impl TryFrom<&str> for Role {
    type Error = Error;

    fn try_from(val: &str) -> Result<Self> {
        match val.to_lowercase().as_str() {
            Self::VISIT => Ok(Self::Visit),
            Self::REPORT => Ok(Self::Report),
            Self::TRIAGE => Ok(Self::Triage),
            Self::WRITE => Ok(Self::Write),
            Self::MAINTAIN => Ok(Self::Maintain),
            Self::ADMIN => Ok(Self::Admin),
            Self::DELEGATE => Ok(Self::Delegate),
            Self::PUBLIC => Ok(Self::Public),
            Self::DENY => Ok(Self::Deny),
            role => Err(Error::vocabulary(format!("role: invalid variant: {role}"))),
        }
    }
}

impl TryFrom<String> for Role {
    type Error = Error;

    fn try_from(val: String) -> Result<Self> {
        val.as_str().try_into()
    }
}

impl TryFrom<Iri> for Role {
    type Error = Error;

    fn try_from(val: Iri) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&Iri> for Role {
    type Error = Error;

    fn try_from(val: &Iri) -> Result<Self> {
        val.as_str().try_into()
    }
}

impl TryFrom<DbIri> for Role {
    type Error = Error;

    fn try_from(val: DbIri) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&DbIri> for Role {
    type Error = Error;

    fn try_from(val: &DbIri) -> Result<Self> {
        val.as_str().try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role() {
        [
            (Role::Visit, Role::VISIT),
            (Role::Report, Role::REPORT),
            (Role::Triage, Role::TRIAGE),
            (Role::Write, Role::WRITE),
            (Role::Maintain, Role::MAINTAIN),
            (Role::Admin, Role::ADMIN),
            (Role::Delegate, Role::DELEGATE),
            (Role::Public, Role::PUBLIC),
            (Role::Deny, Role::DENY),
        ]
        .into_iter()
        .for_each(|(cap, cap_str)| {
            let json_str = format!(r#""{cap_str}""#);

            assert_eq!(serde_json::to_string_pretty(&cap).unwrap(), json_str);
            assert_eq!(
                serde_json::from_str::<Role>(json_str.as_str()).unwrap(),
                cap,
            );
        });

        assert!(Role::Public < Role::Visit);
        assert!(Role::Public < Role::Report);
        assert!(Role::Public < Role::Triage);
        assert!(Role::Public < Role::Write);
        assert!(Role::Public < Role::Maintain);
        assert!(Role::Public < Role::Admin);
        assert!(Role::Public < Role::Delegate);
        assert!(Role::Public < Role::Deny);

        assert!(Role::Visit < Role::Report);
        assert!(Role::Visit < Role::Triage);
        assert!(Role::Visit < Role::Write);
        assert!(Role::Visit < Role::Maintain);
        assert!(Role::Visit < Role::Admin);
        assert!(Role::Visit < Role::Delegate);
        assert!(Role::Visit < Role::Deny);

        assert!(Role::Report < Role::Triage);
        assert!(Role::Report < Role::Write);
        assert!(Role::Report < Role::Maintain);
        assert!(Role::Report < Role::Admin);
        assert!(Role::Report < Role::Delegate);
        assert!(Role::Report < Role::Deny);

        assert!(Role::Triage < Role::Write);
        assert!(Role::Triage < Role::Maintain);
        assert!(Role::Triage < Role::Admin);
        assert!(Role::Triage < Role::Delegate);
        assert!(Role::Triage < Role::Deny);

        assert!(Role::Write < Role::Maintain);
        assert!(Role::Write < Role::Admin);
        assert!(Role::Write < Role::Delegate);
        assert!(Role::Write < Role::Deny);

        assert!(Role::Maintain < Role::Admin);
        assert!(Role::Maintain < Role::Delegate);
        assert!(Role::Maintain < Role::Deny);

        assert!(Role::Admin < Role::Delegate);
        assert!(Role::Admin < Role::Deny);

        assert!(Role::Delegate < Role::Deny);
    }
}
