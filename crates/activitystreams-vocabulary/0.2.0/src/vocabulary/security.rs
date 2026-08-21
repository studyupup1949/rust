use serde::{Deserialize, Serialize};

use crate::{ActivityVocabulary, VocabularyType, VocabularyTypes, impl_default, impl_display};

/// Represents the ActivityStream vocabulary type variants for security vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
pub enum SecurityType {
    /// Controlled Identifier [Multikey](https://www.w3.org/TR/cid-1.0/#Multikey).
    Multikey,
}

impl SecurityType {
    /// Represents the string for the [Multikey](Self::Multikey) type.
    pub const MULTIKEY: &str = "Multikey";

    /// Creates a new [SecurityType].
    #[inline]
    pub const fn new() -> Self {
        Self::Multikey
    }

    /// Gets the string representation of the [SecurityType].
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Multikey => Self::MULTIKEY,
        }
    }

    /// Converts the [SecurityType] to a [VocabularyType].
    #[inline]
    pub const fn to_vocabulary(self) -> VocabularyType {
        VocabularyType::Security(self)
    }

    /// Converts the [SecurityType] to a [VocabularyTypes].
    #[inline]
    pub const fn to_vocabulary_types(self) -> VocabularyTypes {
        VocabularyTypes::Single(self.to_vocabulary())
    }
}

impl_default!(SecurityType);
impl_display!(SecurityType, str);

impl ActivityVocabulary for SecurityType {
    type Type = SecurityType;

    fn kind(&self) -> String {
        self.to_string()
    }

    fn contains(&self, kind: &str) -> bool {
        self.as_str() == kind
    }
}

impl From<SecurityType> for &'static str {
    fn from(val: SecurityType) -> Self {
        val.as_str()
    }
}

impl From<SecurityType> for VocabularyType {
    fn from(val: SecurityType) -> Self {
        val.to_vocabulary()
    }
}

impl From<SecurityType> for VocabularyTypes {
    fn from(val: SecurityType) -> Self {
        val.to_vocabulary_types()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestType;

    #[test]
    fn test_actor() {
        [(SecurityType::Multikey, SecurityType::MULTIKEY)]
            .into_iter()
            .for_each(|(ty, ty_str)| {
                assert_eq!(ty.as_str(), ty_str);
                assert_eq!(ty.kind(), ty_str);
                assert_eq!(ty.as_type(), Ok(ty));

                let json_str = format!(r#""{ty_str}""#);
                assert_eq!(serde_json::to_string(&ty).unwrap(), json_str);
                assert_eq!(
                    serde_json::from_str::<SecurityType>(json_str.as_str()).unwrap(),
                    ty
                );

                let test_ty =
                    serde_json::from_str::<TestType<SecurityType>>(json_str.as_str()).unwrap();
                assert_eq!(test_ty.as_type().unwrap(), ty);
            });
    }
}
