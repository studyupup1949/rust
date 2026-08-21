use activitystreams_vocabulary::{impl_default, impl_display};
use serde::{Deserialize, Serialize};

/// Represents a [Ticket](crate::Ticket) depended-by relationship.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DependedBy {
    /// Identifies one or more tickets on which this [Ticket](crate::Ticket) depends, i.e. it can’t be resolved without those tickets being resolved too.
    DependedBy,
}

impl DependedBy {
    pub const DEPENDED_BY: &str = "dependedBy";

    /// Creates a new [DependedBy].
    pub const fn new() -> Self {
        Self::DependedBy
    }

    /// Gets the [DependedBy] string representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::DependedBy => Self::DEPENDED_BY,
        }
    }
}

impl_default!(DependedBy);
impl_display!(DependedBy, str);

/// Represents a [Ticket](crate::Ticket) depends-on relationship.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DependsOn {
    /// Identifies one or more tickets which depend on this [Ticket](crate::Ticket), i.e. they can’t be resolved without this tickets being resolved too.
    DependsOn,
}

impl DependsOn {
    pub const DEPENDS_ON: &str = "dependsOn";

    /// Creates a new [DependsOn].
    pub const fn new() -> Self {
        Self::DependsOn
    }

    /// Gets the [DependsOn] string representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::DependsOn => Self::DEPENDS_ON,
        }
    }
}

impl_default!(DependsOn);
impl_display!(DependsOn, str);

/// Represents a [Ticket](crate::Ticket) dependency relationship.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DependencyRelationship {
    /// Identifies one or more tickets on which this [Ticket](crate::Ticket) depends, i.e. it can’t be resolved without those tickets being resolved too.
    DependedBy,
    /// Identifies one or more tickets which depend on this [Ticket](crate::Ticket), i.e. they can’t be resolved without this tickets being resolved too.
    DependsOn,
}

impl DependencyRelationship {
    pub const DEPENDED_BY: &str = "dependedBy";
    pub const DEPENDS_ON: &str = "dependsOn";

    /// Creates a new [DependedBy].
    pub const fn new() -> Self {
        Self::DependedBy
    }

    /// Gets the [DependedBy] string representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::DependedBy => Self::DEPENDED_BY,
            Self::DependsOn => Self::DEPENDS_ON,
        }
    }
}

impl_default!(DependencyRelationship);
impl_display!(DependencyRelationship, str);
