//! ABAC access request types.

use std::collections::HashMap;

use crate::attribute::AttributeType;

/// Errors that can occur when building an ABAC request.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum RequestError {
    /// Too many attributes in request (DoS protection limit exceeded).
    #[error("too many attributes: maximum {0}")]
    TooManyAttributes(usize),
    /// Too many groups in a single attribute (DoS protection limit exceeded).
    #[error("too many groups in attribute: maximum {0}")]
    TooManyGroups(usize),
}

/// Maximum number of attributes (dimensions) allowed in a request.
/// Prevents DoS attacks via requests with excessive dimensions.
const MAX_ATTRIBUTES: usize = 100;

/// Maximum number of group memberships per attribute.
/// Prevents DoS attacks via requests with excessive group lists.
const MAX_GROUPS_PER_ATTRIBUTE: usize = 1000;

/// Access request with arbitrary attribute dimensions.
///
/// Represents a single access control decision point: an entity requesting
/// access with specific attribute values across multiple dimensions.
///
/// # Examples
///
/// ```
/// use abac_rs::{AbacRequest, AttributeType};
///
/// let mut request = AbacRequest::new();
///
/// // User dimension: alice in group:engineers
/// request.add_attribute(
///     "user",
///     AttributeType::String("alice".into()),
///     vec![AttributeType::String("group:engineers".into())],
/// ).unwrap();
///
/// // Resource dimension: specific database
/// request.add_attribute(
///     "resource",
///     AttributeType::String("prod:db-01".into()),
///     vec![],
/// ).unwrap();
///
/// // Action dimension: read operation
/// request.add_attribute(
///     "action",
///     AttributeType::String("read".into()),
///     vec![],
/// ).unwrap();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct AbacRequest {
    /// Map of dimension name → (primary value, group memberships).
    ///
    /// The primary value is the main attribute (e.g., user ID "alice").
    /// Group memberships are additional values that can match (e.g., "group:engineers").
    ///
    /// Private to enforce validation via `add_attribute()`.
    attributes: HashMap<String, (AttributeType, Vec<AttributeType>)>,
}

impl AbacRequest {
    /// Create a new empty request.
    pub fn new() -> Self {
        Self {
            attributes: HashMap::new(),
        }
    }

    /// Add an attribute dimension to this request.
    ///
    /// # Limits
    ///
    /// - Maximum 100 attributes (dimensions) per request
    /// - Maximum 1000 groups per attribute
    ///
    /// These limits prevent denial-of-service attacks via maliciously crafted requests.
    ///
    /// # Errors
    ///
    /// Returns [`RequestError::TooManyAttributes`] if adding this attribute would exceed
    /// MAX_ATTRIBUTES (100).
    ///
    /// Returns [`RequestError::TooManyGroups`] if the groups list exceeds
    /// MAX_GROUPS_PER_ATTRIBUTE (1000).
    ///
    /// # Examples
    ///
    /// ```
    /// use abac_rs::{AbacRequest, AttributeType};
    ///
    /// let mut request = AbacRequest::new();
    /// request.add_attribute(
    ///     "user",
    ///     AttributeType::String("alice".into()),
    ///     vec![AttributeType::String("group:admins".into())],
    /// )?;
    /// # Ok::<(), abac_rs::RequestError>(())
    /// ```
    pub fn add_attribute(
        &mut self,
        dimension: impl Into<String>,
        value: AttributeType,
        groups: Vec<AttributeType>,
    ) -> Result<(), RequestError> {
        if self.attributes.len() >= MAX_ATTRIBUTES {
            return Err(RequestError::TooManyAttributes(MAX_ATTRIBUTES));
        }
        if groups.len() > MAX_GROUPS_PER_ATTRIBUTE {
            return Err(RequestError::TooManyGroups(MAX_GROUPS_PER_ATTRIBUTE));
        }
        self.attributes.insert(dimension.into(), (value, groups));
        Ok(())
    }

    /// Get read-only access to all attributes.
    ///
    /// This provides access to the internal attribute map without allowing modification
    /// that would bypass validation.
    pub fn attributes(&self) -> &HashMap<String, (AttributeType, Vec<AttributeType>)> {
        &self.attributes
    }

    /// Iterator over all attributes (dimension, value, groups).
    pub fn iter_attributes(
        &self,
    ) -> impl Iterator<Item = (&str, &AttributeType, &[AttributeType])> {
        self.attributes
            .iter()
            .map(|(k, (v, g))| (k.as_str(), v, g.as_slice()))
    }

    /// Get the attribute value for a dimension.
    pub fn get_attribute(&self, dimension: &str) -> Option<&(AttributeType, Vec<AttributeType>)> {
        self.attributes.get(dimension)
    }

    /// Get the primary value for a dimension.
    pub fn get_value(&self, dimension: &str) -> Option<&AttributeType> {
        self.attributes.get(dimension).map(|(v, _)| v)
    }

    /// Get the groups for a dimension.
    pub fn get_groups(&self, dimension: &str) -> Option<&[AttributeType]> {
        self.attributes.get(dimension).map(|(_, g)| g.as_slice())
    }

    /// Iterator over dimension names.
    pub fn dimensions(&self) -> impl Iterator<Item = &str> {
        self.attributes.keys().map(|s| s.as_str())
    }

    /// Get attributes in a specific dimension order (optimized for compiled evaluator).
    ///
    /// Returns attributes in the same order as `dimension_order`, avoiding repeated HashMap lookups.
    /// This is significantly faster than calling `get_attribute` multiple times.
    #[inline]
    pub fn get_attributes_ordered(
        &self,
        dimension_order: &[String],
    ) -> Vec<Option<(&AttributeType, &[AttributeType])>> {
        dimension_order
            .iter()
            .map(|dim| self.attributes.get(dim).map(|(v, g)| (v, g.as_slice())))
            .collect()
    }
}

impl AbacRequest {
    /// Validate that this request respects DoS protection limits.
    ///
    /// Returns `Ok(())` if the request is within limits, or an error describing
    /// the violation. This is called automatically during deserialization.
    pub fn validate(&self) -> Result<(), RequestError> {
        if self.attributes.len() > MAX_ATTRIBUTES {
            return Err(RequestError::TooManyAttributes(MAX_ATTRIBUTES));
        }
        for (_, groups) in self.attributes.values() {
            if groups.len() > MAX_GROUPS_PER_ATTRIBUTE {
                return Err(RequestError::TooManyGroups(MAX_GROUPS_PER_ATTRIBUTE));
            }
        }
        Ok(())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for AbacRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct Raw {
            attributes: HashMap<String, (AttributeType, Vec<AttributeType>)>,
        }
        let raw = Raw::deserialize(deserializer)?;
        let request = AbacRequest {
            attributes: raw.attributes,
        };
        request.validate().map_err(serde::de::Error::custom)?;
        Ok(request)
    }
}

impl Default for AbacRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_new() {
        let request = AbacRequest::new();
        assert!(request.attributes.is_empty());
    }

    #[test]
    fn test_request_add_attribute() {
        let mut request = AbacRequest::new();

        request
            .add_attribute(
                "user",
                AttributeType::String("alice".into()),
                vec![AttributeType::String("group:admins".into())],
            )
            .unwrap();

        let (value, groups) = request.get_attribute("user").unwrap();
        assert_eq!(value, &AttributeType::String("alice".into()));
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0], AttributeType::String("group:admins".into()));
    }

    #[test]
    fn test_request_get_value() {
        let mut request = AbacRequest::new();
        request
            .add_attribute("user", AttributeType::String("alice".into()), vec![])
            .unwrap();

        assert_eq!(
            request.get_value("user"),
            Some(&AttributeType::String("alice".into()))
        );
        assert_eq!(request.get_value("resource"), None);
    }

    #[test]
    fn test_request_get_groups() {
        let mut request = AbacRequest::new();
        request
            .add_attribute(
                "user",
                AttributeType::String("alice".into()),
                vec![
                    AttributeType::String("group:admins".into()),
                    AttributeType::String("group:users".into()),
                ],
            )
            .unwrap();

        let groups = request.get_groups("user").unwrap();
        assert_eq!(groups.len(), 2);
    }

    #[test]
    fn test_request_dimensions() {
        let mut request = AbacRequest::new();
        request
            .add_attribute("user", AttributeType::String("alice".into()), vec![])
            .unwrap();
        request
            .add_attribute("resource", AttributeType::String("db-01".into()), vec![])
            .unwrap();
        request
            .add_attribute("action", AttributeType::String("read".into()), vec![])
            .unwrap();

        let dims: Vec<_> = request.dimensions().collect();
        assert_eq!(dims.len(), 3);
        assert!(dims.contains(&"user"));
        assert!(dims.contains(&"resource"));
        assert!(dims.contains(&"action"));
    }

    #[test]
    fn test_request_too_many_attributes() {
        let mut request = AbacRequest::new();
        // Add 100 attributes successfully
        for i in 0..100 {
            request
                .add_attribute(
                    format!("dim{}", i),
                    AttributeType::String("value".into()),
                    vec![],
                )
                .unwrap();
        }
        // 101st attribute should fail
        let result = request.add_attribute("dim100", AttributeType::String("value".into()), vec![]);
        assert!(matches!(result, Err(RequestError::TooManyAttributes(100))));
    }

    #[test]
    fn test_request_too_many_groups() {
        let mut request = AbacRequest::new();
        let groups: Vec<_> = (0..=1000)
            .map(|i| AttributeType::String(format!("group{}", i)))
            .collect();
        let result = request.add_attribute("user", AttributeType::String("alice".into()), groups);
        assert!(matches!(result, Err(RequestError::TooManyGroups(1000))));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_request_serde_round_trip() {
        let mut request = AbacRequest::new();
        request
            .add_attribute(
                "user",
                AttributeType::String("alice".into()),
                vec![AttributeType::String("group:admins".into())],
            )
            .unwrap();
        request
            .add_attribute("resource", AttributeType::String("db-01".into()), vec![])
            .unwrap();

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: AbacRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(request, deserialized);
    }
}
