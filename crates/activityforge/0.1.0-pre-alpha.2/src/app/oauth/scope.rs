use std::str::FromStr;

use oauth::primitives::scope::{ParseScopeErr, Scope as OAuthScope};
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgHasArrayType, PgTypeInfo, Postgres};

use activitystreams_vocabulary::{impl_default, impl_display};

use crate::{Error, Role};

mod endpoint;
mod list;

pub use endpoint::*;
pub use list::*;

/// Represents the OAuth-2.0 scopes for accessing application resources.
///
/// Follows the [Mastodon OAuth scopes](https://docs.joinmastodon.org/api/oauth-scopes).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
pub enum Scope {
    #[serde(rename = "profile")]
    Profile,
    #[serde(rename = "push")]
    Push,
    #[serde(rename = "visit")]
    Visit,
    #[serde(rename = "report")]
    Report,
    #[serde(rename = "triage")]
    Triage,
    #[serde(rename = "maintain")]
    Maintain,
    #[serde(rename = "delegate")]
    Delegate,
    #[serde(rename = "register")]
    Register,
    #[serde(untagged)]
    Read(ReadScope),
    #[serde(untagged)]
    Write(WriteScope),
    #[serde(untagged)]
    Admin(AdminScope),
}

impl Scope {
    /// String representation of the [Profile](Self::Profile) variant.
    pub const PROFILE: &str = "profile";
    /// String representation of the [Push](Self::Push) variant.
    pub const PUSH: &str = "push";
    /// String representation of the [Visit](Self::Visit) variant.
    pub const VISIT: &str = "visit";
    /// String representation of the [Report](Self::Report) variant.
    pub const REPORT: &str = "report";
    /// String representation of the [Triage](Self::Triage) variant.
    pub const TRIAGE: &str = "triage";
    /// String representation of the [Maintain](Self::Maintain) variant.
    pub const MAINTAIN: &str = "maintain";
    /// String representation of the [Delegate](Self::Delegate) variant.
    pub const DELEGATE: &str = "delegate";
    /// String representation of the [Register](Self::Register) variant.
    pub const REGISTER: &str = "register";

    /// Creates a new [Scope].
    #[inline]
    pub const fn new() -> Self {
        Self::Profile
    }

    /// Gets the [Scope] string representation.
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Profile => Self::PROFILE,
            Self::Push => Self::PUSH,
            Self::Visit => Self::VISIT,
            Self::Report => Self::REPORT,
            Self::Triage => Self::TRIAGE,
            Self::Maintain => Self::MAINTAIN,
            Self::Delegate => Self::DELEGATE,
            Self::Register => Self::REGISTER,
            Self::Read(s) => s.as_str(),
            Self::Write(s) => s.as_str(),
            Self::Admin(s) => s.as_str(),
        }
    }
}

impl From<Scope> for Role {
    fn from(val: Scope) -> Self {
        (&val).into()
    }
}

impl From<&Scope> for Role {
    fn from(val: &Scope) -> Self {
        match val {
            Scope::Profile | Scope::Register | Scope::Visit | Scope::Read(_) => Self::Visit,
            Scope::Push | Scope::Report => Self::Report,
            Scope::Triage => Self::Triage,
            Scope::Write(_) => Self::Write,
            Scope::Maintain => Self::Maintain,
            Scope::Admin(_) => Self::Admin,
            Scope::Delegate => Self::Delegate,
        }
    }
}

impl TryFrom<Scope> for OAuthScope {
    type Error = ParseScopeErr;

    fn try_from(val: Scope) -> Result<Self, Self::Error> {
        Self::from_str(val.as_str())
    }
}

impl TryFrom<&str> for Scope {
    type Error = Error;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        match val {
            Self::PROFILE => Ok(Self::Profile),
            Self::PUSH => Ok(Self::Push),
            Self::VISIT => Ok(Self::Visit),
            Self::REPORT => Ok(Self::Report),
            Self::TRIAGE => Ok(Self::Triage),
            Self::MAINTAIN => Ok(Self::Maintain),
            Self::DELEGATE => Ok(Self::Delegate),
            Self::REGISTER => Ok(Self::Register),
            _ if val.starts_with("read") => ReadScope::try_from(val).map(Self::Read),
            _ if val.starts_with("write") => WriteScope::try_from(val).map(Self::Write),
            _ if val.starts_with("admin") => AdminScope::try_from(val).map(Self::Admin),
            _ => Err(Error::http(format!("oauth: invalid scope: {val}"))),
        }
    }
}

impl<'r, DB> sqlx::Decode<'r, DB> for Scope
where
    DB: sqlx::Database,
    &'r str: sqlx::Decode<'r, DB>,
{
    fn decode(
        value: <DB as sqlx::Database>::ValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let value = <&str as sqlx::Decode<DB>>::decode(value)?;

        Ok(Self::try_from(value)?)
    }
}

impl<'r, DB: sqlx::Database> sqlx::Encode<'r, DB> for Scope
where
    DB: sqlx::Database,
    &'r str: sqlx::Encode<'r, DB>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as sqlx::Database>::ArgumentBuffer<'r>,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        self.as_str().encode_by_ref(buf)
    }
}

impl_default!(Scope);
impl_display!(Scope, str);

/// Represents the SQL type info for [sqlx] conversion.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ScopeTypeInfo;

impl ScopeTypeInfo {
    pub const NAME: &str = "scope";

    pub const fn as_str(&self) -> &'static str {
        Self::NAME
    }
}

impl_display!(ScopeTypeInfo, str);

impl sqlx::TypeInfo for ScopeTypeInfo {
    fn is_null(&self) -> bool {
        false
    }

    fn name(&self) -> &str {
        self.as_str()
    }
}

impl sqlx::Type<Postgres> for Scope {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        PgTypeInfo::with_name(ScopeTypeInfo.as_str())
    }
}

impl PgHasArrayType for Scope {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("scope[]")
    }
}

/// Represents the OAuth-2.0 `read` scopes for accessing application resources.
///
/// Follows the [Mastodon OAuth scopes](https://docs.joinmastodon.org/api/oauth-scopes).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
pub enum ReadScope {
    #[serde(rename = "read")]
    Read,
    #[serde(rename = "read:accounts")]
    Accounts,
    #[serde(rename = "read:blocks")]
    Blocks,
    #[serde(rename = "read:bookmarks")]
    Bookmarks,
    #[serde(rename = "read:favourites")]
    Favourites,
    #[serde(rename = "read:filters")]
    Filters,
    #[serde(rename = "read:follows")]
    Follows,
    #[serde(rename = "read:lists")]
    Lists,
    #[serde(rename = "read:mutes")]
    Mutes,
    #[serde(rename = "read:notifications")]
    Notifications,
    #[serde(rename = "read:search")]
    Search,
    #[serde(rename = "read:statuses")]
    Statuses,
}

impl ReadScope {
    /// String representation of the [Read](Self::Read) variant.
    pub const READ: &str = "read";
    /// String representation of the [Accounts](Self::Accounts) variant.
    pub const ACCOUNTS: &str = "read:accounts";
    /// String representation of the [Blocks](Self::Blocks) variant.
    pub const BLOCKS: &str = "read:blocks";
    /// String representation of the [Bookmarks](Self::Bookmarks) variant.
    pub const BOOKMARKS: &str = "read:bookmarks";
    /// String representation of the [Favourites](Self::Favourites) variant.
    pub const FAVOURITES: &str = "read:favourites";
    /// String representation of the [Filters](Self::Filters) variant.
    pub const FILTERS: &str = "read:filters";
    /// String representation of the [Follows](Self::Follows) variant.
    pub const FOLLOWS: &str = "read:follows";
    /// String representation of the [Lists](Self::Lists) variant.
    pub const LISTS: &str = "read:lists";
    /// String representation of the [Mutes](Self::Mutes) variant.
    pub const MUTES: &str = "read:mutes";
    /// String representation of the [Notifications](Self::Notifications) variant.
    pub const NOTIFICATIONS: &str = "read:notifications";
    /// String representation of the [Search](Self::Search) variant.
    pub const SEARCH: &str = "read:search";
    /// String representation of the [Statuses](Self::Statuses) variant.
    pub const STATUSES: &str = "read:statuses";

    /// Creates a new [ReadScope].
    #[inline]
    pub const fn new() -> Self {
        Self::Read
    }

    /// Gets the [ReadScope] string representation.
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Read => Self::READ,
            Self::Accounts => Self::ACCOUNTS,
            Self::Blocks => Self::BLOCKS,
            Self::Bookmarks => Self::BOOKMARKS,
            Self::Favourites => Self::FAVOURITES,
            Self::Filters => Self::FILTERS,
            Self::Follows => Self::FOLLOWS,
            Self::Lists => Self::LISTS,
            Self::Mutes => Self::MUTES,
            Self::Notifications => Self::NOTIFICATIONS,
            Self::Search => Self::SEARCH,
            Self::Statuses => Self::STATUSES,
        }
    }
}

impl TryFrom<&str> for ReadScope {
    type Error = Error;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        match val {
            Self::READ => Ok(Self::Read),
            Self::ACCOUNTS => Ok(Self::Accounts),
            Self::BLOCKS => Ok(Self::Blocks),
            Self::BOOKMARKS => Ok(Self::Bookmarks),
            Self::FAVOURITES => Ok(Self::Favourites),
            Self::FILTERS => Ok(Self::Filters),
            Self::FOLLOWS => Ok(Self::Follows),
            Self::LISTS => Ok(Self::Lists),
            Self::MUTES => Ok(Self::Mutes),
            Self::NOTIFICATIONS => Ok(Self::Notifications),
            Self::SEARCH => Ok(Self::Search),
            Self::STATUSES => Ok(Self::Statuses),
            _ => Err(Error::http(format!("oauth: invalid read scope: {val}"))),
        }
    }
}

impl_default!(ReadScope);
impl_display!(ReadScope, str);

/// Represents the OAuth-2.0 `write` scopes for accessing application resources.
///
/// Follows the [Mastodon OAuth scopes](https://docs.joinmastodon.org/api/oauth-scopes).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
pub enum WriteScope {
    #[serde(rename = "write")]
    Write,
    #[serde(rename = "write:accounts")]
    Accounts,
    #[serde(rename = "write:blocks")]
    Blocks,
    #[serde(rename = "write:bookmarks")]
    Bookmarks,
    #[serde(rename = "write:favourites")]
    Favourites,
    #[serde(rename = "write:filters")]
    Filters,
    #[serde(rename = "write:follows")]
    Follows,
    #[serde(rename = "write:lists")]
    Lists,
    #[serde(rename = "write:mutes")]
    Mutes,
    #[serde(rename = "write:notifications")]
    Notifications,
    #[serde(rename = "write:search")]
    Search,
    #[serde(rename = "write:statuses")]
    Statuses,
}

impl WriteScope {
    /// String representation of the [Write](Self::Write) variant.
    pub const WRITE: &str = "write";
    /// String representation of the [Accounts](Self::Accounts) variant.
    pub const ACCOUNTS: &str = "write:accounts";
    /// String representation of the [Blocks](Self::Blocks) variant.
    pub const BLOCKS: &str = "write:blocks";
    /// String representation of the [Bookmarks](Self::Bookmarks) variant.
    pub const BOOKMARKS: &str = "write:bookmarks";
    /// String representation of the [Favourites](Self::Favourites) variant.
    pub const FAVOURITES: &str = "write:favourites";
    /// String representation of the [Filters](Self::Filters) variant.
    pub const FILTERS: &str = "write:filters";
    /// String representation of the [Follows](Self::Follows) variant.
    pub const FOLLOWS: &str = "write:follows";
    /// String representation of the [Lists](Self::Lists) variant.
    pub const LISTS: &str = "write:lists";
    /// String representation of the [Mutes](Self::Mutes) variant.
    pub const MUTES: &str = "write:mutes";
    /// String representation of the [Notifications](Self::Notifications) variant.
    pub const NOTIFICATIONS: &str = "write:notifications";
    /// String representation of the [Search](Self::Search) variant.
    pub const SEARCH: &str = "write:search";
    /// String representation of the [Statuses](Self::Statuses) variant.
    pub const STATUSES: &str = "write:statuses";

    /// Creates a new [WriteScope].
    #[inline]
    pub const fn new() -> Self {
        Self::Write
    }

    /// Gets the [WriteScope] string representation.
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Write => Self::WRITE,
            Self::Accounts => Self::ACCOUNTS,
            Self::Blocks => Self::BLOCKS,
            Self::Bookmarks => Self::BOOKMARKS,
            Self::Favourites => Self::FAVOURITES,
            Self::Filters => Self::FILTERS,
            Self::Follows => Self::FOLLOWS,
            Self::Lists => Self::LISTS,
            Self::Mutes => Self::MUTES,
            Self::Notifications => Self::NOTIFICATIONS,
            Self::Search => Self::SEARCH,
            Self::Statuses => Self::STATUSES,
        }
    }
}

impl TryFrom<&str> for WriteScope {
    type Error = Error;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        match val {
            Self::WRITE => Ok(Self::Write),
            Self::ACCOUNTS => Ok(Self::Accounts),
            Self::BLOCKS => Ok(Self::Blocks),
            Self::BOOKMARKS => Ok(Self::Bookmarks),
            Self::FAVOURITES => Ok(Self::Favourites),
            Self::FILTERS => Ok(Self::Filters),
            Self::FOLLOWS => Ok(Self::Follows),
            Self::LISTS => Ok(Self::Lists),
            Self::MUTES => Ok(Self::Mutes),
            Self::NOTIFICATIONS => Ok(Self::Notifications),
            Self::SEARCH => Ok(Self::Search),
            Self::STATUSES => Ok(Self::Statuses),
            _ => Err(Error::http(format!("oauth: invalid write scope: {val}"))),
        }
    }
}

impl_default!(WriteScope);
impl_display!(WriteScope, str);

/// Represents the OAuth-2.0 `admin` scopes for accessing application resources.
///
/// Follows the [Mastodon OAuth scopes](https://docs.joinmastodon.org/api/oauth-scopes).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
pub enum AdminScope {
    #[serde(rename = "admin")]
    Admin,
    #[serde(untagged)]
    Read(AdminReadScope),
    #[serde(untagged)]
    Write(AdminWriteScope),
}

impl AdminScope {
    /// String representation of the [Admin](Self::Admin) variant.
    pub const ADMIN: &str = "admin";

    /// Creates a new [AdminScope].
    #[inline]
    pub const fn new() -> Self {
        Self::Admin
    }

    /// Gets the [AdminScope] string representation.
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => Self::ADMIN,
            Self::Read(s) => s.as_str(),
            Self::Write(s) => s.as_str(),
        }
    }
}

impl TryFrom<&str> for AdminScope {
    type Error = Error;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        match val {
            Self::ADMIN => Ok(Self::Admin),
            _ if val.starts_with("admin:read") => AdminReadScope::try_from(val).map(Self::Read),
            _ if val.starts_with("admin:write") => AdminWriteScope::try_from(val).map(Self::Write),
            _ => Err(Error::http(format!("oauth: invalid admin scope: {val}"))),
        }
    }
}

impl_default!(AdminScope);
impl_display!(AdminScope, str);

/// Represents the OAuth-2.0 `admin:read` scopes for accessing application resources.
///
/// Follows the [Mastodon OAuth scopes](https://docs.joinmastodon.org/api/oauth-scopes).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
pub enum AdminReadScope {
    #[serde(rename = "admin:read")]
    Read,
    #[serde(rename = "admin:read:accounts")]
    Accounts,
    #[serde(rename = "admin:read:canonical_email_blocks")]
    CanonicalEmailBlocks,
    #[serde(rename = "admin:read:domain_allows")]
    DomainAllows,
    #[serde(rename = "admin:read:domain_blocks")]
    DomainBlocks,
    #[serde(rename = "admin:read:email_domain_blocks")]
    EmailDomainBlocks,
    #[serde(rename = "admin:read:ip_blocks")]
    IpBlocks,
    #[serde(rename = "admin:read:reports")]
    Reports,
}

impl AdminReadScope {
    /// String representation of the [Read](Self::Read) variant.
    pub const READ: &str = "admin:read";
    /// String representation of the [Accounts](Self::Accounts) variant.
    pub const ACCOUNTS: &str = "admin:read:accounts";
    /// String representation of the [CanonicalEmailBlocks](Self::CanonicalEmailBlocks) variant.
    pub const CANONICAL_EMAIL_BLOCKS: &str = "admin:read:canonical_email_blocks";
    /// String representation of the [DomainAllows](Self::DomainAllows) variant.
    pub const DOMAIN_ALLOWS: &str = "admin:read:domain_allows";
    /// String representation of the [DomainBlocks](Self::DomainBlocks) variant.
    pub const DOMAIN_BLOCKS: &str = "admin:read:domain_blocks";
    /// String representation of the [EmailDomainBlocks](Self::EmailDomainBlocks) variant.
    pub const EMAIL_DOMAIN_BLOCKS: &str = "admin:read:email_domain_blocks";
    /// String representation of the [IpBlocks](Self::IpBlocks) variant.
    pub const IP_BLOCKS: &str = "admin:read:ip_blocks";
    /// String representation of the [Reports](Self::Reports) variant.
    pub const REPORTS: &str = "admin:read:reports";

    /// Creates a new [ReadScope].
    #[inline]
    pub const fn new() -> Self {
        Self::Read
    }

    /// Gets the [ReadScope] string representation.
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Read => Self::READ,
            Self::Accounts => Self::ACCOUNTS,
            Self::CanonicalEmailBlocks => Self::CANONICAL_EMAIL_BLOCKS,
            Self::DomainAllows => Self::DOMAIN_ALLOWS,
            Self::DomainBlocks => Self::DOMAIN_BLOCKS,
            Self::EmailDomainBlocks => Self::EMAIL_DOMAIN_BLOCKS,
            Self::IpBlocks => Self::IP_BLOCKS,
            Self::Reports => Self::REPORTS,
        }
    }
}

impl TryFrom<&str> for AdminReadScope {
    type Error = Error;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        match val {
            Self::READ => Ok(Self::Read),
            Self::ACCOUNTS => Ok(Self::Accounts),
            Self::CANONICAL_EMAIL_BLOCKS => Ok(Self::CanonicalEmailBlocks),
            Self::DOMAIN_ALLOWS => Ok(Self::DomainAllows),
            Self::DOMAIN_BLOCKS => Ok(Self::DomainBlocks),
            Self::EMAIL_DOMAIN_BLOCKS => Ok(Self::EmailDomainBlocks),
            Self::IP_BLOCKS => Ok(Self::IpBlocks),
            Self::REPORTS => Ok(Self::Reports),
            _ => Err(Error::http(format!(
                "oauth: invalid admin read scope: {val}"
            ))),
        }
    }
}

impl_default!(AdminReadScope);
impl_display!(AdminReadScope, str);

/// Represents the OAuth-2.0 `admin:write` scopes for accessing application resources.
///
/// Follows the [Mastodon OAuth scopes](https://docs.joinmastodon.org/api/oauth-scopes).
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize, sqlx::Type,
)]
pub enum AdminWriteScope {
    #[serde(rename = "admin:write")]
    #[sqlx(rename = "admin:write")]
    Write,
    #[serde(rename = "admin:write:accounts")]
    #[sqlx(rename = "admin:write:accounts")]
    Accounts,
    #[serde(rename = "admin:write:canonical_email_blocks")]
    #[sqlx(rename = "admin:write:canonical_email_blocks")]
    CanonicalEmailBlocks,
    #[serde(rename = "admin:write:domain_allows")]
    #[sqlx(rename = "admin:write:domain_allows")]
    DomainAllows,
    #[serde(rename = "admin:write:domain_blocks")]
    #[sqlx(rename = "admin:write:domain_blocks")]
    DomainBlocks,
    #[serde(rename = "admin:write:email_domain_blocks")]
    #[sqlx(rename = "admin:write:email_domain_blocks")]
    EmailDomainBlocks,
    #[serde(rename = "admin:write:ip_blocks")]
    #[sqlx(rename = "admin:write:ip_blocks")]
    IpBlocks,
    #[serde(rename = "admin:write:reports")]
    #[sqlx(rename = "admin:write:reports")]
    Reports,
}

impl AdminWriteScope {
    /// String representation of the [Write](Self::Write) variant.
    pub const WRITE: &str = "admin:write";
    /// String representation of the [Accounts](Self::Accounts) variant.
    pub const ACCOUNTS: &str = "admin:write:accounts";
    /// String representation of the [CanonicalEmailBlocks](Self::CanonicalEmailBlocks) variant.
    pub const CANONICAL_EMAIL_BLOCKS: &str = "admin:write:canonical_email_blocks";
    /// String representation of the [DomainAllows](Self::DomainAllows) variant.
    pub const DOMAIN_ALLOWS: &str = "admin:write:domain_allows";
    /// String representation of the [DomainBlocks](Self::DomainBlocks) variant.
    pub const DOMAIN_BLOCKS: &str = "admin:write:domain_blocks";
    /// String representation of the [EmailDomainBlocks](Self::EmailDomainBlocks) variant.
    pub const EMAIL_DOMAIN_BLOCKS: &str = "admin:write:email_domain_blocks";
    /// String representation of the [IpBlocks](Self::IpBlocks) variant.
    pub const IP_BLOCKS: &str = "admin:write:ip_blocks";
    /// String representation of the [Reports](Self::Reports) variant.
    pub const REPORTS: &str = "admin:write:reports";

    /// Creates a new [WriteScope].
    #[inline]
    pub const fn new() -> Self {
        Self::Write
    }

    /// Gets the [WriteScope] string representation.
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Write => Self::WRITE,
            Self::Accounts => Self::ACCOUNTS,
            Self::CanonicalEmailBlocks => Self::CANONICAL_EMAIL_BLOCKS,
            Self::DomainAllows => Self::DOMAIN_ALLOWS,
            Self::DomainBlocks => Self::DOMAIN_BLOCKS,
            Self::EmailDomainBlocks => Self::EMAIL_DOMAIN_BLOCKS,
            Self::IpBlocks => Self::IP_BLOCKS,
            Self::Reports => Self::REPORTS,
        }
    }
}

impl TryFrom<&str> for AdminWriteScope {
    type Error = Error;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        match val {
            Self::WRITE => Ok(Self::Write),
            Self::ACCOUNTS => Ok(Self::Accounts),
            Self::CANONICAL_EMAIL_BLOCKS => Ok(Self::CanonicalEmailBlocks),
            Self::DOMAIN_ALLOWS => Ok(Self::DomainAllows),
            Self::DOMAIN_BLOCKS => Ok(Self::DomainBlocks),
            Self::EMAIL_DOMAIN_BLOCKS => Ok(Self::EmailDomainBlocks),
            Self::IP_BLOCKS => Ok(Self::IpBlocks),
            Self::REPORTS => Ok(Self::Reports),
            _ => Err(Error::http(format!(
                "oauth: invalid admin write scope: {val}"
            ))),
        }
    }
}

impl_default!(AdminWriteScope);
impl_display!(AdminWriteScope, str);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope() {
        [
            (Scope::Profile, Scope::PROFILE),
            (Scope::Push, Scope::PUSH),
            (Scope::Visit, Scope::VISIT),
            (Scope::Report, Scope::REPORT),
            (Scope::Triage, Scope::TRIAGE),
            (Scope::Maintain, Scope::MAINTAIN),
            (Scope::Delegate, Scope::DELEGATE),
        ]
        .into_iter()
        .for_each(|(scope, scope_str)| {
            let exp_json = format!(r#""{scope_str}""#);

            assert_eq!(scope.as_str(), scope_str);
            assert_eq!(Scope::try_from(scope_str).unwrap(), scope);
            assert_eq!(serde_json::to_string(&scope).unwrap(), exp_json);
            assert_eq!(serde_json::from_str::<Scope>(&exp_json).unwrap(), scope);
        });
    }

    #[test]
    fn test_read_scope() {
        [
            (ReadScope::Read, ReadScope::READ),
            (ReadScope::Accounts, ReadScope::ACCOUNTS),
            (ReadScope::Blocks, ReadScope::BLOCKS),
            (ReadScope::Bookmarks, ReadScope::BOOKMARKS),
            (ReadScope::Favourites, ReadScope::FAVOURITES),
            (ReadScope::Filters, ReadScope::FILTERS),
            (ReadScope::Follows, ReadScope::FOLLOWS),
            (ReadScope::Lists, ReadScope::LISTS),
            (ReadScope::Mutes, ReadScope::MUTES),
            (ReadScope::Notifications, ReadScope::NOTIFICATIONS),
            (ReadScope::Search, ReadScope::SEARCH),
            (ReadScope::Statuses, ReadScope::STATUSES),
        ]
        .into_iter()
        .for_each(|(scope, scope_str)| {
            let exp_json = format!(r#""{scope_str}""#);

            assert_eq!(scope.as_str(), scope_str);
            assert_eq!(ReadScope::try_from(scope_str).unwrap(), scope);
            assert_eq!(serde_json::to_string(&scope).unwrap(), exp_json);
            assert_eq!(serde_json::from_str::<ReadScope>(&exp_json).unwrap(), scope);

            let scope = Scope::Read(scope);

            assert_eq!(scope.as_str(), scope_str);
            assert_eq!(Scope::try_from(scope_str).unwrap(), scope);
            assert_eq!(serde_json::to_string(&scope).unwrap(), exp_json);
            assert_eq!(serde_json::from_str::<Scope>(&exp_json).unwrap(), scope);
        });
    }

    #[test]
    fn test_write_scope() {
        [
            (WriteScope::Write, WriteScope::WRITE),
            (WriteScope::Accounts, WriteScope::ACCOUNTS),
            (WriteScope::Blocks, WriteScope::BLOCKS),
            (WriteScope::Bookmarks, WriteScope::BOOKMARKS),
            (WriteScope::Favourites, WriteScope::FAVOURITES),
            (WriteScope::Filters, WriteScope::FILTERS),
            (WriteScope::Follows, WriteScope::FOLLOWS),
            (WriteScope::Lists, WriteScope::LISTS),
            (WriteScope::Mutes, WriteScope::MUTES),
            (WriteScope::Notifications, WriteScope::NOTIFICATIONS),
            (WriteScope::Search, WriteScope::SEARCH),
            (WriteScope::Statuses, WriteScope::STATUSES),
        ]
        .into_iter()
        .for_each(|(scope, scope_str)| {
            let exp_json = format!(r#""{scope_str}""#);

            assert_eq!(scope.as_str(), scope_str);
            assert_eq!(WriteScope::try_from(scope_str).unwrap(), scope);
            assert_eq!(serde_json::to_string(&scope).unwrap(), exp_json);
            assert_eq!(
                serde_json::from_str::<WriteScope>(&exp_json).unwrap(),
                scope
            );

            let scope = Scope::Write(scope);

            assert_eq!(scope.as_str(), scope_str);
            assert_eq!(Scope::try_from(scope_str).unwrap(), scope);
            assert_eq!(serde_json::to_string(&scope).unwrap(), exp_json);
            assert_eq!(serde_json::from_str::<Scope>(&exp_json).unwrap(), scope);
        });
    }

    #[test]
    fn test_admin_scope() {
        [(AdminScope::Admin, AdminScope::ADMIN)]
            .into_iter()
            .for_each(|(scope, scope_str)| {
                let exp_json = format!(r#""{scope_str}""#);

                assert_eq!(scope.as_str(), scope_str);
                assert_eq!(AdminScope::try_from(scope_str).unwrap(), scope);
                assert_eq!(serde_json::to_string(&scope).unwrap(), exp_json);
                assert_eq!(
                    serde_json::from_str::<AdminScope>(&exp_json).unwrap(),
                    scope
                );

                let scope = Scope::Admin(scope);

                assert_eq!(scope.as_str(), scope_str);
                assert_eq!(Scope::try_from(scope_str).unwrap(), scope);
                assert_eq!(serde_json::to_string(&scope).unwrap(), exp_json);
                assert_eq!(serde_json::from_str::<Scope>(&exp_json).unwrap(), scope);
            });
    }

    #[test]
    fn test_admin_read_scope() {
        [
            (AdminReadScope::Read, AdminReadScope::READ),
            (AdminReadScope::Accounts, AdminReadScope::ACCOUNTS),
            (
                AdminReadScope::CanonicalEmailBlocks,
                AdminReadScope::CANONICAL_EMAIL_BLOCKS,
            ),
            (AdminReadScope::DomainAllows, AdminReadScope::DOMAIN_ALLOWS),
            (AdminReadScope::DomainBlocks, AdminReadScope::DOMAIN_BLOCKS),
            (
                AdminReadScope::EmailDomainBlocks,
                AdminReadScope::EMAIL_DOMAIN_BLOCKS,
            ),
            (AdminReadScope::IpBlocks, AdminReadScope::IP_BLOCKS),
            (AdminReadScope::Reports, AdminReadScope::REPORTS),
        ]
        .into_iter()
        .for_each(|(scope, scope_str)| {
            let exp_json = format!(r#""{scope_str}""#);

            assert_eq!(scope.as_str(), scope_str);
            assert_eq!(AdminReadScope::try_from(scope_str).unwrap(), scope);
            assert_eq!(serde_json::to_string(&scope).unwrap(), exp_json);
            assert_eq!(
                serde_json::from_str::<AdminReadScope>(&exp_json).unwrap(),
                scope
            );

            let scope = AdminScope::Read(scope);

            assert_eq!(scope.as_str(), scope_str);
            assert_eq!(AdminScope::try_from(scope_str).unwrap(), scope);
            assert_eq!(serde_json::to_string(&scope).unwrap(), exp_json);
            assert_eq!(
                serde_json::from_str::<AdminScope>(&exp_json).unwrap(),
                scope
            );

            let scope = Scope::Admin(scope);

            assert_eq!(scope.as_str(), scope_str);
            assert_eq!(serde_json::to_string(&scope).unwrap(), exp_json);
            assert_eq!(serde_json::from_str::<Scope>(&exp_json).unwrap(), scope);
        });
    }

    #[test]
    fn test_admin_write_scope() {
        [
            (AdminWriteScope::Write, AdminWriteScope::WRITE),
            (AdminWriteScope::Accounts, AdminWriteScope::ACCOUNTS),
            (
                AdminWriteScope::CanonicalEmailBlocks,
                AdminWriteScope::CANONICAL_EMAIL_BLOCKS,
            ),
            (
                AdminWriteScope::DomainAllows,
                AdminWriteScope::DOMAIN_ALLOWS,
            ),
            (
                AdminWriteScope::DomainBlocks,
                AdminWriteScope::DOMAIN_BLOCKS,
            ),
            (
                AdminWriteScope::EmailDomainBlocks,
                AdminWriteScope::EMAIL_DOMAIN_BLOCKS,
            ),
            (AdminWriteScope::IpBlocks, AdminWriteScope::IP_BLOCKS),
            (AdminWriteScope::Reports, AdminWriteScope::REPORTS),
        ]
        .into_iter()
        .for_each(|(scope, scope_str)| {
            let exp_json = format!(r#""{scope_str}""#);

            assert_eq!(scope.as_str(), scope_str);
            assert_eq!(AdminWriteScope::try_from(scope_str).unwrap(), scope);
            assert_eq!(serde_json::to_string(&scope).unwrap(), exp_json);
            assert_eq!(
                serde_json::from_str::<AdminWriteScope>(&exp_json).unwrap(),
                scope
            );

            let scope = AdminScope::Write(scope);

            assert_eq!(scope.as_str(), scope_str);
            assert_eq!(AdminScope::try_from(scope_str).unwrap(), scope);
            assert_eq!(serde_json::to_string(&scope).unwrap(), exp_json);
            assert_eq!(
                serde_json::from_str::<AdminScope>(&exp_json).unwrap(),
                scope
            );

            let scope = Scope::Admin(scope);

            assert_eq!(scope.as_str(), scope_str);
            assert_eq!(serde_json::to_string(&scope).unwrap(), exp_json);
            assert_eq!(serde_json::from_str::<Scope>(&exp_json).unwrap(), scope);
        });
    }
}
