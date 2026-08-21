use serde::{Deserialize, Serialize};

use crate::{ActivityVocabulary, VocabularyType, VocabularyTypes, impl_default, impl_display};

/// Represents the ActivityStream vocabulary type variants for "objects".
///
/// Actor types are Object types that are capable of performing activities.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
pub enum ActorType {
    Application,
    Group,
    Organization,
    Person,
    Service,
}

impl ActorType {
    /// Represents the string for the [Application](Self::Application) type.
    pub const APPLICATION: &str = "Application";
    /// Represents the string for the [Group](Self::Group) type.
    pub const GROUP: &str = "Group";
    /// Represents the string for the [Organization](Self::Organization) type.
    pub const ORGANIZATION: &str = "Organization";
    /// Represents the string for the [Person](Self::Person) type.
    pub const PERSON: &str = "Person";
    /// Represents the string for the [Service](Self::Service) type.
    pub const SERVICE: &str = "Service";

    /// Creates a new [ActorType].
    #[inline]
    pub const fn new() -> Self {
        Self::Application
    }

    /// Gets the string representation of the [ActorType].
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Application => Self::APPLICATION,
            Self::Group => Self::GROUP,
            Self::Organization => Self::ORGANIZATION,
            Self::Person => Self::PERSON,
            Self::Service => Self::SERVICE,
        }
    }

    /// Converts the [ActorType] to a [VocabularyType].
    #[inline]
    pub const fn to_vocabulary(self) -> VocabularyType {
        VocabularyType::Actor(self)
    }

    /// Converts the [ActorType] to a [VocabularyTypes].
    #[inline]
    pub const fn to_vocabulary_types(self) -> VocabularyTypes {
        VocabularyTypes::Single(self.to_vocabulary())
    }
}

impl_default!(ActorType);
impl_display!(ActorType, str);

impl ActivityVocabulary for ActorType {
    type Type = ActorType;

    fn kind(&self) -> String {
        self.to_string()
    }

    fn contains(&self, kind: &str) -> bool {
        self.as_str() == kind
    }
}

impl From<ActorType> for &'static str {
    fn from(val: ActorType) -> Self {
        val.as_str()
    }
}

impl From<ActorType> for VocabularyType {
    fn from(val: ActorType) -> Self {
        val.to_vocabulary()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestType;

    #[test]
    fn test_actor() {
        [
            (ActorType::Application, ActorType::APPLICATION),
            (ActorType::Group, ActorType::GROUP),
            (ActorType::Organization, ActorType::ORGANIZATION),
            (ActorType::Person, ActorType::PERSON),
            (ActorType::Service, ActorType::SERVICE),
        ]
        .into_iter()
        .for_each(|(ty, ty_str)| {
            assert_eq!(ty.as_str(), ty_str);
            assert_eq!(ty.kind(), ty_str);
            assert_eq!(ty.as_type(), Ok(ty));

            let json_str = format!(r#""{ty_str}""#);
            assert_eq!(serde_json::to_string(&ty).unwrap(), json_str);
            assert_eq!(
                serde_json::from_str::<ActorType>(json_str.as_str()).unwrap(),
                ty
            );

            let test_ty = serde_json::from_str::<TestType<ActorType>>(json_str.as_str()).unwrap();
            assert_eq!(test_ty.as_type().unwrap(), ty);
        });
    }
}
