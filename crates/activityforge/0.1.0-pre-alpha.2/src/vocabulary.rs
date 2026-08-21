use activitystreams_vocabulary::{
    ActivityVocabulary, VocabularyTypes as ActivityStreamsVocabularyTypes,
    impl_activity_vocabulary, impl_default, impl_display, impl_into_vocabulary,
};
use serde::{Deserialize, Serialize};

use crate::{Error, Result};

mod activity;
mod actor;
mod object;

pub use activity::ActivityType;
pub use actor::{ActorType, ActorTypes};
pub use object::ObjectType;

/// Represents the base set of ActivityStreams vocabulary types.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum VocabularyType {
    Activity(ActivityType),
    Actor(ActorType),
    Object(ObjectType),
}

impl VocabularyType {
    /// Creates a new [VocabularyType].
    pub const fn new() -> Self {
        Self::Activity(ActivityType::new())
    }

    /// Gets the [VocabularyType] string representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Activity(ty) => ty.as_str(),
            Self::Actor(ty) => ty.as_str(),
            Self::Object(ty) => ty.as_str(),
        }
    }

    /// Creates a new [Activity](Self::Activity) variant.
    #[inline]
    pub fn activity<I: Into<ActivityType>>(val: I) -> Self {
        Self::Activity(val.into())
    }

    /// Gets whether the [VocabularyType] is an [Activity](Self::Activity).
    pub const fn is_activity(&self) -> bool {
        matches!(self, Self::Activity(_))
    }

    /// Attempts to convert the [VocabularyType] as an [ActivityType].
    pub fn as_activity(&self) -> Result<ActivityType> {
        match self {
            Self::Activity(ty) => Ok(*ty),
            ty => Err(Error::vocabulary(format!(
                "not a valid activity type: {ty}"
            ))),
        }
    }

    /// Creates a new [Actor](Self::Actor) variant.
    #[inline]
    pub fn actor<I: Into<ActorType>>(val: I) -> Self {
        Self::Actor(val.into())
    }

    /// Gets whether the [VocabularyType] is an [Actor](Self::Actor).
    pub const fn is_actor(&self) -> bool {
        matches!(self, Self::Actor(_))
    }

    /// Attempts to convert the [VocabularyType] as an [ActorType].
    pub fn as_actor(&self) -> Result<ActorType> {
        match self {
            Self::Actor(ty) => Ok(*ty),
            ty => Err(Error::vocabulary(format!("not a valid actor type: {ty}"))),
        }
    }

    /// Creates a new [Object](Self::Object) variant.
    #[inline]
    pub fn object<I: Into<ObjectType>>(val: I) -> Self {
        Self::Object(val.into())
    }

    /// Gets whether the [VocabularyType] is an [Object](Self::Object).
    pub const fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }

    /// Attempts to convert the [VocabularyType] as an [ObjectType].
    pub fn as_object(&self) -> Result<ObjectType> {
        match self {
            Self::Object(ty) => Ok(*ty),
            ty => Err(Error::vocabulary(format!("not a valid object type: {ty}"))),
        }
    }
}

impl From<ActivityType> for VocabularyType {
    fn from(val: ActivityType) -> Self {
        Self::Activity(val)
    }
}

impl From<ActorType> for VocabularyType {
    fn from(val: ActorType) -> Self {
        Self::Actor(val)
    }
}

impl From<ObjectType> for VocabularyType {
    fn from(val: ObjectType) -> Self {
        Self::Object(val)
    }
}

impl TryFrom<VocabularyType> for ActivityType {
    type Error = Error;

    fn try_from(val: VocabularyType) -> Result<Self> {
        val.as_activity()
    }
}

impl TryFrom<VocabularyType> for ActorType {
    type Error = Error;

    fn try_from(val: VocabularyType) -> Result<Self> {
        val.as_actor()
    }
}

impl TryFrom<VocabularyType> for ObjectType {
    type Error = Error;

    fn try_from(val: VocabularyType) -> Result<Self> {
        val.as_object()
    }
}

impl_default!(VocabularyType);
impl_display!(VocabularyType, str);
impl_activity_vocabulary!(VocabularyType);
impl_into_vocabulary!(VocabularyType);

/// Convenience alias for a list of [VocabularyType]s.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
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

    /// Creates a new [Single](Self::Single) variant.
    pub fn single<I: Into<VocabularyType>>(val: I) -> Self {
        Self::Single(val.into())
    }

    /// Gets whether the [VocabularyTypes] is a [Single](Self::Single) variant.
    pub const fn is_single(&self) -> bool {
        matches!(self, Self::Single(_))
    }

    /// Attempts to convert the [VocabularyTypes] into a [VocabularyType].
    pub fn as_single(&self) -> Result<VocabularyType> {
        match self {
            Self::Single(ty) => Ok(*ty),
            _ => Err(Error::vocabulary("invalid VocabularyTypes single variant")),
        }
    }

    /// Creates a new [List](Self::List) variant.
    pub fn list<T, I>(list: I) -> Self
    where
        T: Into<VocabularyType>,
        I: IntoIterator<Item = T>,
    {
        Self::List(list.into_iter().map(|i| i.into()).collect())
    }

    /// Gets whether the [VocabularyTypes] is a [List](Self::List) variant.
    pub const fn is_list(&self) -> bool {
        matches!(self, Self::List(_))
    }

    /// Attempts to convert the [VocabularyTypes] into a [VocabularyType] list.
    pub fn as_list(&self) -> Result<Vec<VocabularyType>> {
        match self {
            Self::List(tys) => Ok(tys.clone()),
            _ => Err(Error::vocabulary("invalid VocabularyTypes list variant")),
        }
    }

    /// Gets whether the [VocabularyTypes] variant contains the provided voabulary type.
    pub fn contains<T: Into<VocabularyType>>(&self, t: T) -> bool {
        let oth_ty = t.into();

        match self {
            Self::Single(ty) => ty == &oth_ty,
            Self::List(tys) => tys.iter().any(|ty| ty == &oth_ty),
        }
    }
}

impl_default!(VocabularyTypes);

impl<I: Into<VocabularyType>> From<I> for VocabularyTypes {
    fn from(val: I) -> Self {
        Self::Single(val.into())
    }
}

impl<I: Into<VocabularyType>> From<Vec<I>> for VocabularyTypes {
    fn from(val: Vec<I>) -> Self {
        Self::list(val)
    }
}

impl<I: Into<VocabularyType>, const N: usize> From<[I; N]> for VocabularyTypes {
    fn from(val: [I; N]) -> Self {
        Self::list(val)
    }
}

impl<I: Into<VocabularyType> + Clone, const N: usize> From<&[I; N]> for VocabularyTypes {
    fn from(val: &[I; N]) -> Self {
        Self::list(val.iter().cloned())
    }
}

impl<I: Into<VocabularyType> + Clone> From<&[I]> for VocabularyTypes {
    fn from(val: &[I]) -> Self {
        Self::list(val.iter().cloned())
    }
}

impl TryFrom<VocabularyTypes> for VocabularyType {
    type Error = Error;

    fn try_from(val: VocabularyTypes) -> Result<Self> {
        val.as_single()
    }
}

impl TryFrom<VocabularyTypes> for Vec<VocabularyType> {
    type Error = Error;

    fn try_from(val: VocabularyTypes) -> Result<Self> {
        val.as_list()
    }
}

impl From<VocabularyTypes> for ActivityStreamsVocabularyTypes {
    fn from(val: VocabularyTypes) -> Self {
        match val {
            VocabularyTypes::Single(ty) => Self::Single(ty.into()),
            VocabularyTypes::List(tys) => Self::List(tys.into_iter().map(|i| i.into()).collect()),
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
