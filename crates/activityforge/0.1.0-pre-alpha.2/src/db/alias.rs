use crate::Role;
use crate::app::oauth::{OAuthGrantType, ScopeList};
use crate::crypto::Password;
use crate::db::{ActorType, DateTime, Iri, RoleFilter, TableEntry, Uuid};

/// Convenience alias for an [ActorType] list.
pub type ActorTypeList = Vec<ActorType>;

/// Convenience alias for an [Iri] list.
pub type IriList = Vec<Iri>;

/// Convenience alias for an [OAuthGrantType] list.
pub type OAuthGrantTypeList = Vec<OAuthGrantType>;

/// Convenience alias for a [Role] list.
pub type RoleList = Vec<Role>;

/// Convenience alias for a [RoleFilter] list.
pub type RoleFilterList = Vec<RoleFilter>;

/// Convenience alias for a string list.
pub type StringList = Vec<String>;

/// Convenience alias for an [TableEntry] list.
pub type TableEntryList = Vec<TableEntry>;

/// Convenience alias for an [Uuid] list.
pub type UuidList = Vec<Uuid>;

/// Convenience alias for an optional bool.
pub type OptionalBool = Option<bool>;

/// Convenience alias for an optional date-time.
pub type OptionalDateTime = Option<DateTime>;

/// Convenience alias for an optional [Iri].
pub type OptionalIri = Option<Iri>;

/// Convenience alias for an optional password hash.
pub type OptionalPassword = Option<Password>;

/// Convenience alias for an optional scope list.
pub type OptionalScopeList = Option<ScopeList>;

/// Convenience alias for an optional string.
pub type OptionalString = Option<String>;

/// Convenience alias for an optional [i64].
pub type OptionalI64 = Option<i64>;

/// Convenience alias for an optional [u64].
pub type OptionalU64 = Option<u64>;

/// Convenience alias for an optional [Uuid].
pub type OptionalUuid = Option<Uuid>;

/// Convenience alias for an optional [TableEntry].
pub type OptionalTableEntry = Option<TableEntry>;
