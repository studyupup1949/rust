use serde::{Deserialize, Serialize};

use crate::{ActivityVocabulary, VocabularyType, VocabularyTypes, impl_default, impl_display};

/// Represents the ActivityStream vocabulary type variants for "objects".
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
pub enum CoreType {
    Object,
    Link,
    Activity,
    IntransitiveActivity,
    Collection,
    OrderedCollection,
    CollectionPage,
    OrderedCollectionPage,
}

impl CoreType {
    /// Represents the string for the [Object](Self::Object) type.
    pub const OBJECT: &str = "Object";
    /// Represents the string for the [Link](Self::Link) type.
    pub const LINK: &str = "Link";
    /// Represents the string for the [Activity](Self::Activity) type.
    pub const ACTIVITY: &str = "Activity";
    /// Represents the string for the [IntransitiveActivity](Self::IntransitiveActivity) type.
    pub const INTRANSITIVE_ACTIVITY: &str = "IntransitiveActivity";
    /// Represents the string for the [Collection](Self::Collection) type.
    pub const COLLECTION: &str = "Collection";
    /// Represents the string for the [OrderedCollection](Self::OrderedCollection) type.
    pub const ORDERED_COLLECTION: &str = "OrderedCollection";
    /// Represents the string for the [CollectionPage](Self::CollectionPage) type.
    pub const COLLECTION_PAGE: &str = "CollectionPage";
    /// Represents the string for the [OrderedCollectionPage](Self::OrderedCollectionPage) type.
    pub const ORDERED_COLLECTION_PAGE: &str = "OrderedCollectionPage";

    /// Creates a new [CoreType].
    #[inline]
    pub const fn new() -> Self {
        Self::Object
    }

    /// Gets the string representation of the [CoreType].
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Object => Self::OBJECT,
            Self::Link => Self::LINK,
            Self::Activity => Self::ACTIVITY,
            Self::IntransitiveActivity => Self::INTRANSITIVE_ACTIVITY,
            Self::Collection => Self::COLLECTION,
            Self::OrderedCollection => Self::ORDERED_COLLECTION,
            Self::CollectionPage => Self::COLLECTION_PAGE,
            Self::OrderedCollectionPage => Self::ORDERED_COLLECTION_PAGE,
        }
    }

    /// Converts the [CoreType] to a [VocabularyType].
    #[inline]
    pub const fn to_vocabulary(self) -> VocabularyType {
        VocabularyType::Core(self)
    }

    /// Converts the [CoreType] to a [VocabularyTypes].
    #[inline]
    pub const fn to_vocabulary_types(self) -> VocabularyTypes {
        VocabularyTypes::Single(self.to_vocabulary())
    }
}

impl_default!(CoreType);
impl_display!(CoreType, str);

impl ActivityVocabulary for CoreType {
    type Type = CoreType;

    fn kind(&self) -> String {
        self.to_string()
    }

    fn contains(&self, kind: &str) -> bool {
        self.as_str() == kind
    }
}

impl From<CoreType> for &'static str {
    fn from(val: CoreType) -> Self {
        val.as_str()
    }
}

impl From<CoreType> for VocabularyType {
    fn from(val: CoreType) -> Self {
        val.to_vocabulary()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestType;

    #[test]
    fn test_core() {
        [
            (CoreType::Object, CoreType::OBJECT),
            (CoreType::Link, CoreType::LINK),
            (CoreType::Activity, CoreType::ACTIVITY),
            (
                CoreType::IntransitiveActivity,
                CoreType::INTRANSITIVE_ACTIVITY,
            ),
            (CoreType::Collection, CoreType::COLLECTION),
            (CoreType::OrderedCollection, CoreType::ORDERED_COLLECTION),
            (CoreType::CollectionPage, CoreType::COLLECTION_PAGE),
            (
                CoreType::OrderedCollectionPage,
                CoreType::ORDERED_COLLECTION_PAGE,
            ),
        ]
        .into_iter()
        .for_each(|(ty, ty_str)| {
            assert_eq!(ty.as_str(), ty_str);
            assert_eq!(ty.kind(), ty_str);
            assert_eq!(ty.as_type(), Ok(ty));

            let json_str = format!(r#""{ty_str}""#);
            assert_eq!(serde_json::to_string(&ty).unwrap(), json_str);
            assert_eq!(
                serde_json::from_str::<CoreType>(json_str.as_str()).unwrap(),
                ty
            );

            let test_ty = serde_json::from_str::<TestType<CoreType>>(json_str.as_str()).unwrap();
            assert_eq!(test_ty.as_type().unwrap(), ty);
        });
    }
}
