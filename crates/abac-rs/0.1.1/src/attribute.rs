//! Attribute types and values for ABAC evaluation.
//!
//! This module provides the core attribute abstractions that enable generic,
//! dimension-agnostic ABAC evaluation.

use std::any::Any;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::net::IpAddr;
use std::sync::Arc;

use ahash::AHashSet;
use ipnetwork::IpNetwork;

/// Type alias for the HashSet type used in AttributeValue.
/// Uses AHash for better performance than the default SipHash.
pub type AttributeSet = AHashSet<AttributeType>;

/// Trait for custom attribute types.
///
/// Implement this trait to add domain-specific attribute types beyond the
/// built-in variants.
pub trait AttributeTypeTrait: Send + Sync {
    /// Downcast to `Any` for type introspection
    fn as_any(&self) -> &dyn Any;

    /// Equality check against another `AttributeTypeTrait`
    fn eq(&self, other: &dyn AttributeTypeTrait) -> bool;

    /// Hash value for this attribute
    fn hash(&self) -> u64;

    /// Clone into a new `Arc`
    fn clone_trait(&self) -> Arc<dyn AttributeTypeTrait>;
}

/// Strongly-typed attribute value supporting multiple data types.
///
/// Used in both rule definitions (what values are allowed) and requests
/// (what values are present).
///
/// # Examples
///
/// ```
/// use abac_rs::AttributeType;
/// use std::net::IpAddr;
///
/// let user = AttributeType::String("alice".into());
/// let age = AttributeType::Integer(25);
/// let ip = AttributeType::IpAddr("10.1.2.3".parse().unwrap());
/// ```
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))]
pub enum AttributeType {
    /// UTF-8 string (most common)
    String(String),

    /// Signed 64-bit integer
    Integer(i64),

    /// IEEE 754 double-precision float
    Float(f64),

    /// IPv4 or IPv6 address
    IpAddr(IpAddr),

    /// IP network (CIDR notation)
    IpCidr(IpNetwork),

    /// Custom type via trait object
    #[cfg_attr(feature = "serde", serde(skip))]
    Custom(Arc<dyn AttributeTypeTrait>),
}

// Manual Debug implementation since trait objects don't auto-derive
impl std::fmt::Debug for AttributeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeType::String(s) => f.debug_tuple("String").field(s).finish(),
            AttributeType::Integer(i) => f.debug_tuple("Integer").field(i).finish(),
            AttributeType::Float(fl) => f.debug_tuple("Float").field(fl).finish(),
            AttributeType::IpAddr(ip) => f.debug_tuple("IpAddr").field(ip).finish(),
            AttributeType::IpCidr(cidr) => f.debug_tuple("IpCidr").field(cidr).finish(),
            AttributeType::Custom(_) => f.debug_tuple("Custom").field(&"<custom>").finish(),
        }
    }
}

impl Hash for AttributeType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            AttributeType::String(s) => {
                0u8.hash(state);
                s.hash(state);
            }
            AttributeType::Integer(i) => {
                1u8.hash(state);
                i.hash(state);
            }
            AttributeType::Float(f) => {
                2u8.hash(state);
                f.to_bits().hash(state);
            }
            AttributeType::IpAddr(ip) => {
                3u8.hash(state);
                ip.hash(state);
            }
            AttributeType::IpCidr(cidr) => {
                4u8.hash(state);
                cidr.hash(state);
            }
            AttributeType::Custom(c) => {
                5u8.hash(state);
                c.hash().hash(state);
            }
        }
    }
}

impl PartialEq for AttributeType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AttributeType::String(a), AttributeType::String(b)) => a == b,
            (AttributeType::Integer(a), AttributeType::Integer(b)) => a == b,
            (AttributeType::Float(a), AttributeType::Float(b)) => a.to_bits() == b.to_bits(),
            (AttributeType::IpAddr(a), AttributeType::IpAddr(b)) => a == b,
            (AttributeType::IpCidr(a), AttributeType::IpCidr(b)) => a == b,
            (AttributeType::Custom(a), AttributeType::Custom(b)) => a.eq(b.as_ref()),
            _ => false,
        }
    }
}

impl Eq for AttributeType {}

impl AttributeType {
    /// Parse a string as an IP CIDR network.
    ///
    /// # Arguments
    ///
    /// * `s` - CIDR notation string (e.g., "10.0.0.0/8" or "2001:db8::/32")
    ///
    /// # Errors
    ///
    /// Returns an error if the string is not valid CIDR notation.
    ///
    /// # Examples
    ///
    /// ```
    /// use abac_rs::AttributeType;
    ///
    /// let cidr = AttributeType::from_cidr_str("10.0.0.0/8").unwrap();
    /// ```
    pub fn from_cidr_str(s: &str) -> Result<Self, ipnetwork::IpNetworkError> {
        let network = s.parse()?;
        Ok(AttributeType::IpCidr(network))
    }
}

/// Attribute value specification for rules.
///
/// Either matches all possible values (wildcard) or a specific set.
///
/// # Examples
///
/// ```
/// use abac_rs::{AttributeValue, AttributeType};
/// use ahash::AHashSet as HashSet;
///
/// // Match any value
/// let any_user = AttributeValue::All;
///
/// // Match specific values or groups
/// let mut admins = HashSet::new();
/// admins.insert(AttributeType::String("group:admins".into()));
/// let admin_only = AttributeValue::Specific(admins);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AttributeValue {
    /// Matches any value (wildcard, like hbac-rs `category=all`)
    All,

    /// Matches specific values (uses AHashSet for O(1) membership checks with fast hashing)
    Specific(AttributeSet),
}

#[cfg(feature = "serde")]
impl serde::Serialize for AttributeValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        match self {
            AttributeValue::All => serializer.serialize_str("all"),
            AttributeValue::Specific(set) => {
                let mut seq = serializer.serialize_seq(Some(set.len()))?;
                for item in set {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for AttributeValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct AttributeValueVisitor;

        impl<'de> serde::de::Visitor<'de> for AttributeValueVisitor {
            type Value = AttributeValue;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("'all' or a sequence of attributes")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v == "all" {
                    Ok(AttributeValue::All)
                } else {
                    Err(E::custom("expected 'all'"))
                }
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut set = AttributeSet::new();
                while let Some(item) = seq.next_element()? {
                    set.insert(item);
                }
                Ok(AttributeValue::Specific(set))
            }
        }

        deserializer.deserialize_any(AttributeValueVisitor)
    }
}

impl AttributeValue {
    /// Create a specific attribute value from an iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use abac_rs::{AttributeValue, AttributeType};
    ///
    /// let admins = AttributeValue::from_values([
    ///     AttributeType::String("alice".into()),
    ///     AttributeType::String("bob".into()),
    /// ]);
    /// ```
    pub fn from_values<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = AttributeType>,
    {
        AttributeValue::Specific(iter.into_iter().collect())
    }

    /// Check if this value is `All` (wildcard).
    pub fn is_all(&self) -> bool {
        matches!(self, AttributeValue::All)
    }

    /// Check if this value is `Specific`.
    pub fn is_specific(&self) -> bool {
        matches!(self, AttributeValue::Specific(_))
    }

    /// Get the specific values if this is `Specific`, otherwise `None`.
    pub fn as_specific(&self) -> Option<&AttributeSet> {
        match self {
            AttributeValue::Specific(set) => Some(set),
            AttributeValue::All => None,
        }
    }
}

// Compatibility: Allow converting from std::collections::HashSet to AttributeValue
impl From<HashSet<AttributeType>> for AttributeValue {
    fn from(set: HashSet<AttributeType>) -> Self {
        AttributeValue::Specific(set.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attribute_type_hash_eq() {
        let s1 = AttributeType::String("alice".into());
        let s2 = AttributeType::String("alice".into());
        let s3 = AttributeType::String("bob".into());

        assert_eq!(s1, s2);
        assert_ne!(s1, s3);

        // Can be used in HashSet
        let mut set = HashSet::new();
        set.insert(s1.clone());
        assert!(set.contains(&s2));
        assert!(!set.contains(&s3));
    }

    #[test]
    fn test_attribute_type_float_eq() {
        let f1 = AttributeType::Float(1.5);
        let f2 = AttributeType::Float(1.5);
        let f3 = AttributeType::Float(2.5);

        assert_eq!(f1, f2);
        assert_ne!(f1, f3);

        // NaN comparison via bit representation
        let nan1 = AttributeType::Float(f64::NAN);
        let nan2 = AttributeType::Float(f64::NAN);
        assert_eq!(nan1, nan2); // Same bit pattern
    }

    #[test]
    fn test_attribute_type_ip() {
        let ip1 = AttributeType::IpAddr("10.1.2.3".parse().unwrap());
        let ip2 = AttributeType::IpAddr("10.1.2.3".parse().unwrap());
        let ip3 = AttributeType::IpAddr("10.1.2.4".parse().unwrap());

        assert_eq!(ip1, ip2);
        assert_ne!(ip1, ip3);
    }

    #[test]
    fn test_attribute_type_cidr() {
        let cidr1 = AttributeType::IpCidr("10.0.0.0/8".parse().unwrap());
        let cidr2 = AttributeType::IpCidr("10.0.0.0/8".parse().unwrap());
        let cidr3 = AttributeType::IpCidr("192.168.0.0/16".parse().unwrap());

        assert_eq!(cidr1, cidr2);
        assert_ne!(cidr1, cidr3);
    }

    #[test]
    fn test_attribute_value_all() {
        let all = AttributeValue::All;
        assert!(all.is_all());
        assert!(!all.is_specific());
        assert!(all.as_specific().is_none());
    }

    #[test]
    fn test_attribute_value_specific() {
        let specific = AttributeValue::from_values([
            AttributeType::String("alice".into()),
            AttributeType::String("bob".into()),
        ]);

        assert!(!specific.is_all());
        assert!(specific.is_specific());

        let set = specific.as_specific().unwrap();
        assert_eq!(set.len(), 2);
        assert!(set.contains(&AttributeType::String("alice".into())));
        assert!(set.contains(&AttributeType::String("bob".into())));
    }
}
