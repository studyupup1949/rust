use std::str::FromStr;

use activitystreams_vocabulary::{impl_default, impl_display};
use oauth::primitives::scope::{ParseScopeErr, Scope as OAuthScope};
use serde::{de, ser};
use sqlx::postgres::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueRef, Postgres};

use super::Scope;
use crate::{Error, Result};

/// Represents a list of OAuth-2.0 scopes for accessing application resources.
///
/// Follows the [Mastodon OAuth scopes](https://docs.joinmastodon.org/api/oauth-scopes).
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ScopeList(Vec<Scope>);

impl ScopeList {
    /// Creates a new [ScopeList].
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    /// Adds a [Scope] to the [ScopeList].
    pub fn add_scope(&mut self, scope: Scope) -> Result<()> {
        if self.0.contains(&scope) {
            Err(Error::http(format!(
                "oauth: scope list already contains: {scope}"
            )))
        } else {
            self.0.push(scope);
            self.0.sort();
            Ok(())
        }
    }

    /// Gets whether the [ScopeList] contains the `scope`.
    pub fn contains(&self, scope: &Scope) -> bool {
        self.0.contains(scope)
    }

    /// Pushes a `scope` into the [ScopeList].
    pub fn push(&mut self, scope: Scope) {
        self.0.push(scope);
    }

    /// Sorts the [ScopeList].
    pub fn sort(&mut self) {
        self.0.sort();
    }

    /// Deduplicates the [ScopeList].
    ///
    /// If sorted first, all duplicates are removed.
    pub fn dedup(&mut self) {
        self.0.dedup();
    }

    /// Appends another [ScopeList], draining its contents.
    pub fn append(&mut self, oth: &mut Vec<Scope>) {
        oth.sort();
        oth.dedup();

        self.0.append(oth);
    }

    /// Gets a reference to the list of [Scope]s.
    pub fn as_slice(&self) -> &[Scope] {
        self.0.as_slice()
    }

    /// Thin wrapper around the inner [extract_if](Vec::extract_if) method.
    pub fn extract_if<F, R>(&mut self, range: R, filter: F) -> std::vec::ExtractIf<'_, Scope, F>
    where
        F: FnMut(&mut Scope) -> bool,
        R: std::ops::RangeBounds<usize>,
    {
        self.0.extract_if(range, filter)
    }

    /// Gets the [ScopeList] length.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Gets whether the [ScopeList] is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Gets a iterator over the [ScopeList].
    pub fn iter(&self) -> impl Iterator<Item = &Scope> {
        self.0.iter()
    }

    /// Converts the [ScopeList] into its inner vector.
    pub fn into_vec(self) -> Vec<Scope> {
        self.0
    }

    /// Gets the `x-www-form-urlencoded` representation of the [ScopeList].
    pub fn encode_url(&self) -> String {
        self.0
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("+")
    }
}

impl From<&[Scope]> for ScopeList {
    fn from(val: &[Scope]) -> Self {
        let mut list = val.to_vec();
        list.sort();
        list.dedup();
        Self(list)
    }
}

impl<const N: usize> From<&[Scope; N]> for ScopeList {
    fn from(val: &[Scope; N]) -> Self {
        val.as_ref().into()
    }
}

impl<const N: usize> From<[Scope; N]> for ScopeList {
    fn from(val: [Scope; N]) -> Self {
        val.as_ref().into()
    }
}

impl From<Vec<Scope>> for ScopeList {
    fn from(val: Vec<Scope>) -> Self {
        Self(val)
    }
}

impl From<ScopeList> for Vec<Scope> {
    fn from(val: ScopeList) -> Self {
        val.into_vec()
    }
}

impl From<&ScopeList> for Vec<Scope> {
    fn from(val: &ScopeList) -> Self {
        val.clone().into_vec()
    }
}

impl TryFrom<&ScopeList> for OAuthScope {
    type Error = ParseScopeErr;

    fn try_from(val: &ScopeList) -> core::result::Result<Self, Self::Error> {
        let scopes = val.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" ");
        Self::from_str(scopes.as_str())
    }
}

impl TryFrom<ScopeList> for OAuthScope {
    type Error = ParseScopeErr;

    fn try_from(val: ScopeList) -> core::result::Result<Self, Self::Error> {
        let scopes = val
            .into_iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        Self::from_str(scopes.as_str())
    }
}

impl TryFrom<OAuthScope> for ScopeList {
    type Error = Error;

    fn try_from(val: OAuthScope) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&OAuthScope> for ScopeList {
    type Error = Error;

    fn try_from(val: &OAuthScope) -> Result<Self> {
        let mut list = val
            .iter()
            .map(Scope::try_from)
            .collect::<Result<Vec<_>>>()?;

        list.sort();
        list.dedup();

        Ok(Self(list))
    }
}

impl IntoIterator for ScopeList {
    type Item = Scope;
    type IntoIter = <Vec<Scope> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'r> sqlx::Decode<'r, Postgres> for ScopeList {
    fn decode(
        value: PgValueRef,
    ) -> core::result::Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let values = <Vec<Scope> as sqlx::Decode<Postgres>>::decode(value)?;

        Ok(Self(values))
    }
}

impl<'r> sqlx::Encode<'r, Postgres> for ScopeList {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> core::result::Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        self.as_slice().encode_by_ref(buf)
    }
}

impl sqlx::Type<Postgres> for ScopeList {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        Self::array_type_info()
    }
}

impl PgHasArrayType for ScopeList {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("scope[]")
    }
}

impl_default!(ScopeList);
impl_display!(ScopeList, json);

impl AsRef<[Scope]> for ScopeList {
    fn as_ref(&self) -> &[Scope] {
        self.0.as_ref()
    }
}

impl std::ops::Deref for ScopeList {
    type Target = [Scope];

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl ser::Serialize for ScopeList {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        self.0
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" ")
            .serialize(serializer)
    }
}

impl<'de> de::Deserialize<'de> for ScopeList {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        <&str>::deserialize(deserializer).and_then(|s| {
            s.split(" ")
                .map(Scope::try_from)
                .collect::<Result<Vec<_>>>()
                .map(Self)
                .map_err(|err| de::Error::custom(format!("oauth: scope: invalid: {err}")))
        })
    }
}
