use activitystreams_vocabulary::{impl_default, impl_display};
use serde::{Deserialize, Serialize};

/// Represents a mode of using a [Grant](crate::Grant) as an Object Capability (OCAP).
///
/// There are two conceptual operations for [Grant](crate::Grant)s: Invocation (acting on the resource under the specified role),
/// and Delegation (passing on the access to more actors, possibly with reduced privileges).
///
/// A value of this type refers to one or both of these operations, and possibly to more specific conditions and restrictions on applying them.
///
/// # Example
///
/// ```rust
/// use activityforge::CapabilityUsage;
///
/// # fn main() {
/// [
///     (CapabilityUsage::GatherAndConvey, CapabilityUsage::GATHER_AND_CONVEY),
///     (CapabilityUsage::Distribute, CapabilityUsage::DISTRIBUTE),
///     (CapabilityUsage::Invoke, CapabilityUsage::INVOKE),
/// ]
/// .into_iter()
/// .for_each(|(cap, cap_str)| {
///     let json_str = format!(r#""{cap_str}""#);
///
///     assert_eq!(serde_json::to_string_pretty(&cap).unwrap(), json_str);
///     assert_eq!(
///         serde_json::from_str::<CapabilityUsage>(json_str.as_str()).unwrap(),
///         cap,
///     );
/// });
/// # }
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CapabilityUsage {
    /// Gathers the [Grant](crate::Grant)'s capabilities, and conveys to related targets.
    /// - The [Grant](crate::Grant)’s `target` **MUST** be a [Project](crate::Project)
    /// - It may delegate the [Grant](crate::Grant), allowing only `gatherAndConvey`, to parent projects
    /// - It may delegate the [Grant](crate::Grant), allowing only `distribute`, to teams to which it allows to access it
    /// - It may delegate the [Grant](crate::Grant), allowing `invoke` only, to people to which it allows to access it
    GatherAndConvey,
    /// Distributes the [Grant](crate::Grant)'s capabilities to target subteams.
    /// - The [Grant](crate::Grant)’s `target` **MUST** be a [Team](crate::Team)
    /// - It may delegate the [Grant](crate::Grant), allowing `distribute` only, to its subteams
    /// - It may delegate the [Grant](crate::Grant), allowing `invoke` only, to its members
    Distribute,
    /// Invokes the [Grant](crate::Grant)'s capabilities, this is a leaf `target`.
    /// - The [Grant](crate::Grant)’s target may invoke it, i.e. use it as the capability in another activity, that requests to access or modify the resource specified by the [Grant](crate::Grant)’s `context`
    Invoke,
}

impl CapabilityUsage {
    /// Represents the [GatherAndConvey](Self::GatherAndConvey) string.
    pub const GATHER_AND_CONVEY: &str = "gatherAndConvey";
    /// Represents the [Distribute](Self::Distribute) string.
    pub const DISTRIBUTE: &str = "distribute";
    /// Represents the [Invoke](Self::Invoke) string.
    pub const INVOKE: &str = "invoke";

    /// Creates a new [CapabilityUsage].
    pub const fn new() -> Self {
        Self::Invoke
    }

    /// Gets the [CapabilityUsage] string representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::GatherAndConvey => Self::GATHER_AND_CONVEY,
            Self::Distribute => Self::DISTRIBUTE,
            Self::Invoke => Self::INVOKE,
        }
    }
}

impl_default!(CapabilityUsage);
impl_display!(CapabilityUsage, str);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_usage() {
        [
            (
                CapabilityUsage::GatherAndConvey,
                CapabilityUsage::GATHER_AND_CONVEY,
            ),
            (CapabilityUsage::Distribute, CapabilityUsage::DISTRIBUTE),
            (CapabilityUsage::Invoke, CapabilityUsage::INVOKE),
        ]
        .into_iter()
        .for_each(|(cap, cap_str)| {
            let json_str = format!(r#""{cap_str}""#);

            assert_eq!(serde_json::to_string_pretty(&cap).unwrap(), json_str);
            assert_eq!(
                serde_json::from_str::<CapabilityUsage>(json_str.as_str()).unwrap(),
                cap,
            );
        });
    }
}
