use serde::{Deserialize, Serialize};

use crate::{Error, Iri, Result, impl_default, impl_display};

mod activity;
mod actor;
mod core_ty;
mod link;
mod object;
mod security;

pub use activity::ActivityType;
pub use actor::ActorType;
pub use core_ty::CoreType;
pub use link::LinkType;
pub use object::ObjectType;
pub use security::SecurityType;

/// Represents the base set of ActivityStreams vocabulary types.
///
/// Based on the Go implementation [go-ap/activitypub#ActivityVocabularyType](https://pkg.go.dev/github.com/go-ap/activitypub#ActivityVocabularyType).
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(untagged)]
pub enum VocabularyType {
    Activity(ActivityType),
    Core(CoreType),
    Object(ObjectType),
    Actor(ActorType),
    Link(LinkType),
    Security(SecurityType),
    Iri(Iri),
}

impl VocabularyType {
    /// Creates a new [VocabularyType].
    pub const fn new() -> Self {
        Self::Core(CoreType::new())
    }

    /// Gets the string representation of the [VocabularyType].
    pub fn as_str(&self) -> &str {
        match self {
            Self::Activity(ty) => ty.as_str(),
            Self::Core(ty) => ty.as_str(),
            Self::Object(ty) => ty.as_str(),
            Self::Actor(ty) => ty.as_str(),
            Self::Link(ty) => ty.as_str(),
            Self::Security(ty) => ty.as_str(),
            Self::Iri(ty) => ty.as_str(),
        }
    }

    /// Creates a custom [Iri] [VocabularyType].
    pub fn iri<E, I>(val: I) -> Result<Self>
    where
        E: core::error::Error,
        I: TryInto<Iri, Error = E>,
    {
        val.try_into()
            .map_err(|err| Error::vocabulary(format!("invalid IRI: {err}")))
            .map(Self::Iri)
    }

    /// Attempts to convert to a [CoreType].
    ///
    /// Returns an [Error] if [VocabularyType] is not a [CoreType].
    #[inline]
    pub fn to_core(&self) -> Result<CoreType> {
        match self {
            Self::Core(ty) => Ok(*ty),
            ty => Err(Error::vocabulary(format!("invalid core type: {ty}"))),
        }
    }

    /// Attempts to convert to an [ObjectType].
    ///
    /// Returns an [Error] if [VocabularyType] is not an [ObjectType].
    #[inline]
    pub fn to_object(&self) -> Result<ObjectType> {
        match self {
            Self::Object(ty) => Ok(*ty),
            ty => Err(Error::vocabulary(format!("invalid object type: {ty}"))),
        }
    }

    /// Attempts to convert to an [ActivityType].
    ///
    /// Returns an [Error] if [VocabularyType] is not an [ActivityType].
    #[inline]
    pub fn to_activity(&self) -> Result<ActivityType> {
        match self {
            Self::Activity(ty) => Ok(*ty),
            ty => Err(Error::vocabulary(format!("invalid activity type: {ty}"))),
        }
    }

    /// Attempts to convert to an [ActorType].
    ///
    /// Returns an [Error] if [VocabularyType] is not an [ActorType].
    #[inline]
    pub fn to_actor(&self) -> Result<ActorType> {
        match self {
            Self::Actor(ty) => Ok(*ty),
            ty => Err(Error::vocabulary(format!("invalid actor type: {ty}"))),
        }
    }

    /// Attempts to convert to an [LinkType].
    ///
    /// Returns an [Error] if [VocabularyType] is not an [LinkType].
    #[inline]
    pub fn to_link(&self) -> Result<LinkType> {
        match self {
            Self::Link(ty) => Ok(*ty),
            ty => Err(Error::vocabulary(format!("invalid link type: {ty}"))),
        }
    }

    /// Attempts to convert to an [SecurityType].
    ///
    /// Returns an [Error] if [VocabularyType] is not an [SecurityType].
    #[inline]
    pub fn to_security(&self) -> Result<SecurityType> {
        match self {
            Self::Security(ty) => Ok(*ty),
            ty => Err(Error::vocabulary(format!("invalid security type: {ty}"))),
        }
    }

    /// Attempts to convert to a custom type.
    ///
    /// Returns an [Error] if [VocabularyTypes] is not an [Iri](Self::Iri) variant.
    #[inline]
    pub fn to_iri(&self) -> Result<Iri> {
        match self {
            Self::Iri(ty) => Ok(ty.clone()),
            ty => Err(Error::vocabulary(format!("invalid IRI type: {ty}"))),
        }
    }
}

impl_default!(VocabularyType);
impl_display!(VocabularyType, str);

impl ActivityVocabulary for VocabularyType {
    type Type = VocabularyTypes;

    fn kind(&self) -> String {
        self.to_string()
    }

    fn contains(&self, kind: &str) -> bool {
        self.as_str() == kind
    }
}

impl TryFrom<VocabularyType> for CoreType {
    type Error = Error;

    fn try_from(val: VocabularyType) -> Result<Self> {
        val.to_core()
    }
}

/// Convenience alias for a list of [VocabularyType]s.
///
/// Based on the Go implementation [go-ap/activitypub#ActivityVocabularyTypes](https://pkg.go.dev/github.com/go-ap/activitypub#ActivityVocabularyTypes).
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(untagged)]
pub enum VocabularyTypes {
    Single(VocabularyType),
    List(Vec<VocabularyType>),
}

impl VocabularyTypes {
    /// Creates a new [VocabularyTypes].
    pub const fn new() -> Self {
        Self::Single(VocabularyType::new())
    }

    /// Gets the inner [VocabularyType].
    ///
    /// If the instance holds a [List](Self::List) variant, the first item will be returned.
    /// If the list is empty, [`None`] is returned.
    pub fn kind(&self) -> Option<VocabularyType> {
        match self {
            Self::Single(ty) => Some(ty.clone()),
            Self::List(tys) => tys.first().cloned(),
        }
    }

    /// Gets an owned list of the inner [VocabularyType]s.
    pub fn kind_list(&self) -> Vec<VocabularyType> {
        match self {
            Self::Single(ty) => vec![ty.clone()],
            Self::List(tys) => tys.clone(),
        }
    }

    /// Converts a list of [VocabularyType] into a [VocabularyTypes].
    pub fn from_list<I: Into<Vec<VocabularyType>>>(list: I) -> Self {
        Self::List(list.into())
    }

    /// Gets whether [VocabularyTypes] instance contains the provided type.
    pub fn contains<T: Into<VocabularyType>>(&self, t: T) -> bool {
        let oth_ty = t.into();

        match self {
            Self::Single(ty) => ty == &oth_ty,
            Self::List(tys) => tys.iter().any(|ty| ty == &oth_ty),
        }
    }
}

impl ActivityVocabulary for VocabularyTypes {
    type Type = Self;

    fn kind(&self) -> String {
        self.to_string()
    }

    fn contains(&self, kind: &str) -> bool {
        match self {
            Self::Single(ty) => ty.as_str() == kind,
            Self::List(tys) => tys.iter().any(|ty| ty.as_str() == kind),
        }
    }
}

impl From<VocabularyType> for VocabularyTypes {
    fn from(val: VocabularyType) -> Self {
        Self::Single(val)
    }
}

impl_default!(VocabularyTypes);

impl core::fmt::Display for VocabularyTypes {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Single(ty) => write!(f, "{ty}"),
            Self::List(tys) => serde_json::to_string(tys)
                .map_err(|_| core::fmt::Error)
                .and_then(|s| write!(f, "{s}")),
        }
    }
}

/// Marker trait for ActivityStreams 2.0 vocabulary types.
///
/// This trait is [`dyn` compatible](https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility).
///
/// Based on the Go implementation [go-ap/activitypub#Typer](https://pkg.go.dev/github.com/go-ap/activitypub#Typer)
pub trait ActivityVocabulary:
    for<'de> Deserialize<'de> + Serialize + Clone + core::fmt::Debug + Default + Eq + PartialEq
{
    /// Represents the concrete ActivityStream vocabulary type expected.
    ///
    /// For contexts where the expected type is unknown, use [GenericType](crate::GenericType).
    type Type: for<'de> Deserialize<'de>;

    /// Gets the string representation of the ActivityStream vocabulary type.
    ///
    /// Should be the unquoted string.
    ///
    /// Example:
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{ActivityVocabulary, ObjectType};
    ///
    /// let obj = ObjectType::Article;
    /// assert_eq!(obj.kind(), "Article");
    /// assert!(obj.kind() != r#""Article""#);
    /// ```
    fn kind(&self) -> String;

    /// Attempts to convert the trait into the concrete [Type](Self::Type).
    fn as_type(&self) -> Result<Self::Type> {
        serde_json::from_str::<Self::Type>(format!(r#""{}""#, self.kind()).as_str())
            .map_err(Error::from)
    }

    /// Gets whether the Activity `type` contains the `kind` Activity type.
    fn contains(&self, kind: &str) -> bool;
}
