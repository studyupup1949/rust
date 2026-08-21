use serde::{Deserialize, Serialize};

use crate::{ActivityVocabulary, VocabularyType, VocabularyTypes, impl_default, impl_display};

/// Represents the ActivityStream vocabulary type variants for "objects".
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
pub enum ObjectType {
    Article,
    Audio,
    Document,
    Event,
    Image,
    Note,
    Page,
    Place,
    Profile,
    Relationship,
    Tombstone,
    Video,
}

impl ObjectType {
    /// Represents the string for the [Article](Self::Article) type.
    pub const ARTICLE: &str = "Article";
    /// Represents the string for the [Audio](Self::Audio) type.
    pub const AUDIO: &str = "Audio";
    /// Represents the string for the [Document](Self::Document) type.
    pub const DOCUMENT: &str = "Document";
    /// Represents the string for the [Event](Self::Event) type.
    pub const EVENT: &str = "Event";
    /// Represents the string for the [Image](Self::Image) type.
    pub const IMAGE: &str = "Image";
    /// Represents the string for the [Note](Self::Note) type.
    pub const NOTE: &str = "Note";
    /// Represents the string for the [Page](Self::Page) type.
    pub const PAGE: &str = "Page";
    /// Represents the string for the [Place](Self::Place) type.
    pub const PLACE: &str = "Place";
    /// Represents the string for the [Profile](Self::Profile) type.
    pub const PROFILE: &str = "Profile";
    /// Represents the string for the [Relationship](Self::Relationship) type.
    pub const RELATIONSHIP: &str = "Relationship";
    /// Represents the string for the [Tombstone](Self::Tombstone) type.
    pub const TOMBSTONE: &str = "Tombstone";
    /// Represents the string for the [Video](Self::Video) type.
    pub const VIDEO: &str = "Video";

    /// Creates a new [ObjectType].
    #[inline]
    pub const fn new() -> Self {
        Self::Article
    }

    /// Gets the string representation of the [ObjectType].
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Article => Self::ARTICLE,
            Self::Audio => Self::AUDIO,
            Self::Document => Self::DOCUMENT,
            Self::Event => Self::EVENT,
            Self::Image => Self::IMAGE,
            Self::Note => Self::NOTE,
            Self::Page => Self::PAGE,
            Self::Place => Self::PLACE,
            Self::Profile => Self::PROFILE,
            Self::Relationship => Self::RELATIONSHIP,
            Self::Tombstone => Self::TOMBSTONE,
            Self::Video => Self::VIDEO,
        }
    }

    /// Converts the [ObjectType] to a [VocabularyType].
    #[inline]
    pub const fn to_vocabulary(self) -> VocabularyType {
        VocabularyType::Object(self)
    }

    /// Converts the [ObjectType] to a [VocabularyTypes].
    #[inline]
    pub const fn to_vocabulary_types(self) -> VocabularyTypes {
        VocabularyTypes::Single(self.to_vocabulary())
    }
}

impl_default!(ObjectType);
impl_display!(ObjectType, str);

impl ActivityVocabulary for ObjectType {
    type Type = ObjectType;

    fn kind(&self) -> String {
        self.to_string()
    }

    fn contains(&self, kind: &str) -> bool {
        self.as_str() == kind
    }
}

impl From<ObjectType> for &'static str {
    fn from(val: ObjectType) -> Self {
        val.as_str()
    }
}

impl From<ObjectType> for VocabularyType {
    fn from(val: ObjectType) -> Self {
        val.to_vocabulary()
    }
}

impl From<ObjectType> for VocabularyTypes {
    fn from(val: ObjectType) -> Self {
        val.to_vocabulary_types()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestType;

    #[test]
    fn test_object() {
        [
            (ObjectType::Article, ObjectType::ARTICLE),
            (ObjectType::Audio, ObjectType::AUDIO),
            (ObjectType::Document, ObjectType::DOCUMENT),
            (ObjectType::Event, ObjectType::EVENT),
            (ObjectType::Image, ObjectType::IMAGE),
            (ObjectType::Note, ObjectType::NOTE),
            (ObjectType::Page, ObjectType::PAGE),
            (ObjectType::Place, ObjectType::PLACE),
            (ObjectType::Profile, ObjectType::PROFILE),
            (ObjectType::Relationship, ObjectType::RELATIONSHIP),
            (ObjectType::Tombstone, ObjectType::TOMBSTONE),
            (ObjectType::Video, ObjectType::VIDEO),
        ]
        .into_iter()
        .for_each(|(ty, ty_str)| {
            assert_eq!(ty.as_str(), ty_str);
            assert_eq!(ty.kind(), ty_str);
            assert_eq!(ty.as_type(), Ok(ty));

            let json_str = format!(r#""{ty_str}""#);
            assert_eq!(serde_json::to_string(&ty).unwrap(), json_str);
            assert_eq!(
                serde_json::from_str::<ObjectType>(json_str.as_str()).unwrap(),
                ty
            );

            let test_ty = serde_json::from_str::<TestType<ObjectType>>(json_str.as_str()).unwrap();
            assert_eq!(test_ty.as_type().unwrap(), ty);
        });
    }
}
