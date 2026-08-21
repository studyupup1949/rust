//! Request key for LRU cache lookup.
//!
//! Provides order-independent hashing of request attributes for cache keys.

use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

use crate::{AbacRequest, AttributeType};

/// Cache key for LRU memoization.
///
/// Normalizes request attributes so that the same logical request (with groups
/// in different order) produces the same hash.
///
/// # Order Independence
///
/// Group memberships are sorted before hashing to ensure:
/// ```text
/// request1: user=alice, groups=[admin, dev]
/// request2: user=alice, groups=[dev, admin]
/// ```
/// Both produce the same cache key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestKey {
    /// Sorted dimension → (value, sorted groups) mapping
    attributes: BTreeMap<String, (AttributeType, Vec<AttributeType>)>,
}

impl RequestKey {
    /// Create a cache key from a request.
    ///
    /// Optimization: Uses shallow clones and in-place sorting to minimize allocations.
    pub fn from_request(request: &AbacRequest) -> Self {
        let mut attributes = BTreeMap::new();

        for (dim, (value, groups)) in request.attributes() {
            // Optimization: Only sort groups if there are multiple
            // Most requests have 0-1 groups per dimension
            let sorted_groups = if groups.len() <= 1 {
                groups.clone() // Already sorted (0 or 1 element)
            } else {
                let mut sorted_groups = groups.clone();
                sorted_groups.sort_unstable_by(compare_attribute_types);
                sorted_groups
            };

            attributes.insert(dim.clone(), (value.clone(), sorted_groups));
        }

        RequestKey { attributes }
    }
}

/// Compare two AttributeTypes for sorting.
///
/// Orders by type discriminant first, then by value within the same type.
fn compare_attribute_types(a: &AttributeType, b: &AttributeType) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    use AttributeType::*;

    match (a, b) {
        (String(a), String(b)) => a.cmp(b),
        (Integer(a), Integer(b)) => a.cmp(b),
        (Float(a), Float(b)) => {
            // Compare floats by bit representation for total ordering
            a.to_bits().cmp(&b.to_bits())
        }
        (IpAddr(a), IpAddr(b)) => a.cmp(b),
        (IpCidr(a), IpCidr(b)) => {
            // Compare CIDR by network address then prefix length
            match a.network().cmp(&b.network()) {
                Ordering::Equal => a.prefix().cmp(&b.prefix()),
                other => other,
            }
        }
        (Custom(_), Custom(_)) => {
            // Custom types are not comparable - use identity
            Ordering::Equal
        }
        // Different variants - order by discriminant
        (String(_), _) => Ordering::Less,
        (_, String(_)) => Ordering::Greater,
        (Integer(_), _) => Ordering::Less,
        (_, Integer(_)) => Ordering::Greater,
        (Float(_), _) => Ordering::Less,
        (_, Float(_)) => Ordering::Greater,
        (IpAddr(_), _) => Ordering::Less,
        (_, IpAddr(_)) => Ordering::Greater,
        (IpCidr(_), _) => Ordering::Less,
        (_, IpCidr(_)) => Ordering::Greater,
    }
}

impl Hash for RequestKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // BTreeMap iteration is deterministic (sorted by key)
        for (dim, (value, groups)) in &self.attributes {
            dim.hash(state);
            value.hash(state);

            // Hash groups (already sorted)
            groups.len().hash(state);
            for group in groups {
                group.hash(state);
            }
        }
    }
}

impl From<&AbacRequest> for RequestKey {
    fn from(request: &AbacRequest) -> Self {
        RequestKey::from_request(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AbacRequest;
    use std::collections::hash_map::DefaultHasher;

    fn hash_key(key: &RequestKey) -> u64 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn test_request_key_order_independence() {
        // Request 1: groups in order [admin, dev]
        let mut request1 = AbacRequest::new();
        request1
            .add_attribute(
                "user",
                AttributeType::String("alice".into()),
                vec![
                    AttributeType::String("admin".into()),
                    AttributeType::String("dev".into()),
                ],
            )
            .unwrap();

        // Request 2: groups in reverse order [dev, admin]
        let mut request2 = AbacRequest::new();
        request2
            .add_attribute(
                "user",
                AttributeType::String("alice".into()),
                vec![
                    AttributeType::String("dev".into()),
                    AttributeType::String("admin".into()),
                ],
            )
            .unwrap();

        let key1 = RequestKey::from(&request1);
        let key2 = RequestKey::from(&request2);

        // Keys should be equal despite different group order
        assert_eq!(key1, key2);
        assert_eq!(hash_key(&key1), hash_key(&key2));
    }

    #[test]
    fn test_request_key_different_groups() {
        let mut request1 = AbacRequest::new();
        request1
            .add_attribute(
                "user",
                AttributeType::String("alice".into()),
                vec![AttributeType::String("admin".into())],
            )
            .unwrap();

        let mut request2 = AbacRequest::new();
        request2
            .add_attribute(
                "user",
                AttributeType::String("alice".into()),
                vec![AttributeType::String("dev".into())],
            )
            .unwrap();

        let key1 = RequestKey::from(&request1);
        let key2 = RequestKey::from(&request2);

        // Keys should be different with different groups
        assert_ne!(key1, key2);
        assert_ne!(hash_key(&key1), hash_key(&key2));
    }

    #[test]
    fn test_request_key_different_values() {
        let mut request1 = AbacRequest::new();
        request1
            .add_attribute("user", AttributeType::String("alice".into()), vec![])
            .unwrap();

        let mut request2 = AbacRequest::new();
        request2
            .add_attribute("user", AttributeType::String("bob".into()), vec![])
            .unwrap();

        let key1 = RequestKey::from(&request1);
        let key2 = RequestKey::from(&request2);

        // Keys should be different with different primary values
        assert_ne!(key1, key2);
        assert_ne!(hash_key(&key1), hash_key(&key2));
    }

    #[test]
    fn test_request_key_multi_dimension() {
        let mut request = AbacRequest::new();
        request
            .add_attribute(
                "user",
                AttributeType::String("alice".into()),
                vec![AttributeType::String("admin".into())],
            )
            .unwrap();
        request
            .add_attribute("resource", AttributeType::String("db-01".into()), vec![])
            .unwrap();
        request
            .add_attribute("action", AttributeType::String("read".into()), vec![])
            .unwrap();

        let key = RequestKey::from(&request);

        // Key should include all dimensions
        assert_eq!(key.attributes.len(), 3);
    }

    #[test]
    fn test_compare_attribute_types() {
        use std::cmp::Ordering;

        // Same type comparisons
        assert_eq!(
            compare_attribute_types(
                &AttributeType::String("a".into()),
                &AttributeType::String("b".into())
            ),
            Ordering::Less
        );

        assert_eq!(
            compare_attribute_types(&AttributeType::Integer(1), &AttributeType::Integer(2)),
            Ordering::Less
        );

        // Different types - String < Integer
        assert_eq!(
            compare_attribute_types(
                &AttributeType::String("a".into()),
                &AttributeType::Integer(1)
            ),
            Ordering::Less
        );

        assert_eq!(
            compare_attribute_types(
                &AttributeType::Integer(1),
                &AttributeType::String("a".into())
            ),
            Ordering::Greater
        );
    }
}
