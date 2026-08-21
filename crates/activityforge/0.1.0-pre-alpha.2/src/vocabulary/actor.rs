use activitystreams_vocabulary::{
    impl_activity_vocabulary, impl_default, impl_display, impl_into_vocabulary,
};
use serde::{Deserialize, Serialize};

use crate::db::TableType;
use crate::{Error, Result};

/// Represents the [ForgeFed](https://forgefed.org/ns) `Actor` type variants.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize, sqlx::Type)]
#[sqlx(type_name = "actor_type")]
pub enum ActorType {
    Factory,
    Repository,
    PatchTracker,
    ReleaseTracker,
    Roadmap,
    TicketTracker,
    Project,
    Team,
    Workflow,
    Application,
    Person,
}

impl ActorType {
    /// String representation of the [Factory](Self::Factory) variant.
    pub const FACTORY: &str = "Factory";
    /// String representation of the [Repository](Self::Repository) variant.
    pub const REPOSITORY: &str = "Repository";
    /// String representation of the [PatchTracker](Self::PatchTracker) variant.
    pub const PATCH_TRACKER: &str = "PatchTracker";
    /// String representation of the [ReleaseTracker](Self::ReleaseTracker) variant.
    pub const RELEASE_TRACKER: &str = "ReleaseTracker";
    /// String representation of the [Roadmap](Self::Roadmap) variant.
    pub const ROADMAP: &str = "Roadmap";
    /// String representation of the [TicketTracker](Self::TicketTracker) variant.
    pub const TICKET_TRACKER: &str = "TicketTracker";
    /// String representation of the [Project](Self::Project) variant.
    pub const PROJECT: &str = "Project";
    /// String representation of the [Team](Self::Team) variant.
    pub const TEAM: &str = "Team";
    /// String representation of the [Workflow](Self::Workflow) variant.
    pub const WORKFLOW: &str = "Workflow";
    /// String representation of the [Application](Self::Application) variant.
    pub const APPLICATION: &str = "Application";
    /// String representation of the [Person](Self::Person) variant.
    pub const PERSON: &str = "Person";

    /// Creates a new [ForgeAc
    pub const fn new() -> Self {
        Self::Factory
    }

    /// Gets the [ActorType] string representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Factory => Self::FACTORY,
            Self::Repository => Self::REPOSITORY,
            Self::PatchTracker => Self::PATCH_TRACKER,
            Self::ReleaseTracker => Self::RELEASE_TRACKER,
            Self::Roadmap => Self::ROADMAP,
            Self::TicketTracker => Self::TICKET_TRACKER,
            Self::Project => Self::PROJECT,
            Self::Team => Self::TEAM,
            Self::Workflow => Self::WORKFLOW,
            Self::Application => Self::APPLICATION,
            Self::Person => Self::PERSON,
        }
    }

    /// Attempts to convert a string into an [ActorType].
    pub fn try_from_str(val: &str) -> Result<Self> {
        match val {
            Self::FACTORY => Ok(Self::Factory),
            Self::REPOSITORY => Ok(Self::Repository),
            Self::PATCH_TRACKER => Ok(Self::PatchTracker),
            Self::RELEASE_TRACKER => Ok(Self::ReleaseTracker),
            Self::ROADMAP => Ok(Self::Roadmap),
            Self::TICKET_TRACKER => Ok(Self::TicketTracker),
            Self::PROJECT => Ok(Self::Project),
            Self::TEAM => Ok(Self::Team),
            Self::WORKFLOW => Ok(Self::Workflow),
            Self::APPLICATION => Ok(Self::Application),
            Self::PERSON => Ok(Self::Person),
            _ => Err(Error::vocabulary(format!("invalid actor type: {val}"))),
        }
    }
}

impl_default!(ActorType);
impl_display!(ActorType, str);
impl_activity_vocabulary!(ActorType);
impl_into_vocabulary!(ActorType);

impl<'a> From<&'a ActorType> for &'static str {
    fn from(val: &'a ActorType) -> Self {
        val.as_str()
    }
}

impl TryFrom<&str> for ActorType {
    type Error = Error;

    fn try_from(val: &str) -> Result<Self> {
        Self::try_from_str(val)
    }
}

impl TryFrom<String> for ActorType {
    type Error = Error;

    fn try_from(val: String) -> Result<Self> {
        val.as_str().try_into()
    }
}

impl TryFrom<TableType> for ActorType {
    type Error = Error;

    fn try_from(val: TableType) -> Result<Self> {
        match val {
            TableType::Factory => Ok(Self::Factory),
            TableType::Repository => Ok(Self::Repository),
            TableType::Team => Ok(Self::Team),
            TableType::PatchTracker => Ok(Self::PatchTracker),
            TableType::TicketTracker => Ok(Self::TicketTracker),
            TableType::Application => Ok(Self::Application),
            TableType::Person => Ok(Self::Person),
            _ => Err(Error::actor(format!("invalid table type: {val}"))),
        }
    }
}

/// Represents a list of [ActorType]s.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ActorTypes {
    Single(ActorType),
    List(Vec<ActorType>),
}

impl ActorTypes {
    /// Creates a new [ActorTypes].
    pub fn new() -> Self {
        Self::Single(ActorType::new())
    }

    /// Creates a new [ActorTypes] [Single](Self::Single) variant.
    pub fn single<I: Into<ActorType>>(val: I) -> Self {
        Self::Single(val.into())
    }

    /// Gets whether the [ActorTypes] contains a [Single](Self::Single) variant.
    pub const fn is_single(&self) -> bool {
        matches!(self, Self::Single(_))
    }

    /// Attempts to get a reference to the [Single](Self::Single) variant.
    pub fn as_single(&self) -> Result<&ActorType> {
        match self {
            Self::Single(ty) => Ok(ty),
            _ => Err(Error::vocabulary("invalid keys type")),
        }
    }

    /// Attempts to convert to a [Single](Self::Single) variant.
    pub fn into_single(self) -> Result<ActorType> {
        match self {
            Self::Single(ty) => Ok(ty),
            _ => Err(Error::vocabulary("invalid keys type")),
        }
    }

    /// Creates a new [ActorTypes] [Single](Self::Single) variant.
    pub fn list<T, I>(val: I) -> Self
    where
        T: Into<ActorType>,
        I: IntoIterator<Item = T>,
    {
        Self::List(val.into_iter().map(|i| i.into()).collect())
    }

    /// Gets whether the [ActorTypes] contains a [List](Self::List) variant.
    pub const fn is_list(&self) -> bool {
        matches!(self, Self::List(_))
    }

    /// Attempts to get a reference to the [List](Self::List) variant.
    pub fn as_list(&self) -> Result<&[ActorType]> {
        match self {
            Self::List(tys) => Ok(tys),
            _ => Err(Error::vocabulary("invalid keys type")),
        }
    }

    /// Attempts to convert to a [List](Self::List) variant.
    pub fn into_list(self) -> Result<Vec<ActorType>> {
        match self {
            Self::List(tys) => Ok(tys),
            _ => Err(Error::vocabulary("invalid keys type")),
        }
    }
}

impl<T: Into<ActorType>> From<T> for ActorTypes {
    fn from(val: T) -> Self {
        Self::single(val)
    }
}

impl<T: Into<ActorType>> From<Vec<T>> for ActorTypes {
    fn from(val: Vec<T>) -> Self {
        Self::list(val)
    }
}

impl<T: Into<ActorType> + Clone> From<&[T]> for ActorTypes {
    fn from(val: &[T]) -> Self {
        Self::list(val.iter().cloned())
    }
}

impl<T: Into<ActorType> + Clone, const N: usize> From<&[T; N]> for ActorTypes {
    fn from(val: &[T; N]) -> Self {
        Self::list(val.iter().cloned())
    }
}

impl<T: Into<ActorType>, const N: usize> From<[T; N]> for ActorTypes {
    fn from(val: [T; N]) -> Self {
        Self::list(val)
    }
}

impl<'a> TryFrom<&'a ActorTypes> for &'a ActorType {
    type Error = Error;

    fn try_from(val: &'a ActorTypes) -> Result<Self> {
        val.as_single()
    }
}

impl TryFrom<ActorTypes> for ActorType {
    type Error = Error;

    fn try_from(val: ActorTypes) -> Result<Self> {
        val.into_single()
    }
}

impl<'a> TryFrom<&'a ActorTypes> for &'a [ActorType] {
    type Error = Error;

    fn try_from(val: &'a ActorTypes) -> Result<Self> {
        val.as_list()
    }
}

impl TryFrom<ActorTypes> for Vec<ActorType> {
    type Error = Error;

    fn try_from(val: ActorTypes) -> Result<Self> {
        val.into_list()
    }
}

impl_default!(ActorTypes);
impl_display!(ActorTypes, json);

#[cfg(test)]
mod tests {
    use activitystreams_vocabulary::ActivityVocabulary;

    use super::*;
    use crate::tests::TestType;

    #[test]
    fn test_actor() {
        [
            (ActorType::Factory, ActorType::FACTORY),
            (ActorType::Repository, ActorType::REPOSITORY),
            (ActorType::PatchTracker, ActorType::PATCH_TRACKER),
            (ActorType::ReleaseTracker, ActorType::RELEASE_TRACKER),
            (ActorType::Roadmap, ActorType::ROADMAP),
            (ActorType::TicketTracker, ActorType::TICKET_TRACKER),
            (ActorType::Project, ActorType::PROJECT),
            (ActorType::Team, ActorType::TEAM),
            (ActorType::Workflow, ActorType::WORKFLOW),
            (ActorType::Application, ActorType::APPLICATION),
            (ActorType::Person, ActorType::PERSON),
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
