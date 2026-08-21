use serde::{Deserialize, Serialize};

use super::ActivityVocabulary;

use crate::{impl_default, impl_display};

/// Represents a generic activity vocabulary type.
///
/// Mostly intended to be used in contexts where expected activity type is unknown.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
pub struct GenericType(String);

impl GenericType {
    /// Creates a new [GenericType].
    pub const fn new() -> Self {
        Self(String::new())
    }

    /// Gets a reference to the inner string.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Converts a string into a [GenericType].
    pub fn from_string<S: Into<String>>(s: S) -> Self {
        Self(s.into())
    }
}

impl From<&str> for GenericType {
    fn from(val: &str) -> Self {
        Self::from_string(val)
    }
}

impl<'a> From<&'a GenericType> for &'a str {
    fn from(val: &'a GenericType) -> Self {
        val.as_str()
    }
}

impl From<GenericType> for String {
    fn from(val: GenericType) -> Self {
        val.to_string()
    }
}

impl From<String> for GenericType {
    fn from(val: String) -> Self {
        Self::from_string(val)
    }
}

impl ActivityVocabulary for GenericType {
    type Type = String;

    fn kind(&self) -> String {
        self.to_string()
    }

    fn contains(&self, kind: &str) -> bool {
        self.as_str() == kind
    }
}

impl_default!(GenericType);
impl_display!(GenericType, str);

#[cfg(test)]
mod test {
    use super::*;
    use crate::tests::TestType;

    #[test]
    fn test_generic() {
        let ty_str = "custom";
        let ty = GenericType::from_string(ty_str);
        let json_str = serde_json::to_string(&ty).unwrap();

        assert_eq!(ty.as_str(), ty_str);
        assert_eq!(ty.kind(), ty_str);
        assert_eq!(ty.as_type(), Ok(ty_str.to_owned()));

        let test_ty = serde_json::from_str::<TestType<GenericType>>(json_str.as_str()).unwrap();
        assert_eq!(test_ty.as_type().unwrap().as_str(), ty.as_str());

        assert_eq!(<&str>::from(&ty), ty_str);
        assert_eq!(String::from(ty), ty_str.to_owned());
    }
}
