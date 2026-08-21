use serde::{Deserialize, Serialize};

use crate::{ActivityVocabulary, VocabularyType, VocabularyTypes, impl_default, impl_display};

/// Represents the ActivityStream vocabulary type variants for "links".
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
pub enum LinkType {
    Mention,
}

impl LinkType {
    /// Represents the string for the [Mention](Self::Mention) type.
    pub const MENTION: &str = "Mention";

    /// Creates a new [LinkType].
    #[inline]
    pub const fn new() -> Self {
        Self::Mention
    }

    /// Gets the string representation of the [LinkType].
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Mention => Self::MENTION,
        }
    }

    /// Converts the [LinkType] to a [VocabularyType].
    #[inline]
    pub const fn to_vocabulary(self) -> VocabularyType {
        VocabularyType::Link(self)
    }

    /// Converts the [LinkType] to a [VocabularyTypes].
    #[inline]
    pub const fn to_vocabulary_types(self) -> VocabularyTypes {
        VocabularyTypes::Single(self.to_vocabulary())
    }
}

impl_default!(LinkType);
impl_display!(LinkType, str);

impl ActivityVocabulary for LinkType {
    type Type = LinkType;

    fn kind(&self) -> String {
        self.to_string()
    }

    fn contains(&self, kind: &str) -> bool {
        self.as_str() == kind
    }
}

impl From<LinkType> for &'static str {
    fn from(val: LinkType) -> Self {
        val.as_str()
    }
}

impl From<LinkType> for VocabularyType {
    fn from(val: LinkType) -> Self {
        val.to_vocabulary()
    }
}

impl From<LinkType> for VocabularyTypes {
    fn from(val: LinkType) -> Self {
        Self::Single(val.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestType;

    #[test]
    fn test_core() {
        [(LinkType::Mention, LinkType::MENTION)]
            .into_iter()
            .for_each(|(ty, ty_str)| {
                assert_eq!(ty.as_str(), ty_str);
                assert_eq!(ty.kind(), ty_str);
                assert_eq!(ty.as_type(), Ok(ty));

                let json_str = format!(r#""{ty_str}""#);
                assert_eq!(serde_json::to_string(&ty).unwrap(), json_str);
                assert_eq!(
                    serde_json::from_str::<LinkType>(json_str.as_str()).unwrap(),
                    ty
                );

                let test_ty =
                    serde_json::from_str::<TestType<LinkType>>(json_str.as_str()).unwrap();
                assert_eq!(test_ty.as_type().unwrap(), ty);
            });
    }
}
