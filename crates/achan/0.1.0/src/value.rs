use alloc::{collections::BTreeMap, string::String, vec::Vec};

/// Represents any value.
#[derive(Clone, Debug, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Value {
    /// Represents the absence of a value.
    #[default]
    Null,
    /// Represents `true` or `false`.
    Boolean(bool),
    /// Represents any number.
    Number(f64),
    /// Represents any string of characters.
    String(String),
    /// Represents any sequence of values.
    List(Vec<Value>),
    /// Represents any mapping from strings to values.  
    Map(BTreeMap<String, Value>),
}

impl Value {
    /// Returns `true` if this `Value` represents a null value.
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Returns `true` if this `Value` represents a boolean.
    pub fn is_boolean(&self) -> bool {
        matches!(self, Self::Boolean(_))
    }

    /// Returns `true` if this `Value` represents a number.
    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }

    /// Returns `true` if this `Value` represents a string.
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    /// Returns `true` if this `Value` represents a list.
    pub fn is_list(&self) -> bool {
        matches!(self, Self::List(_))
    }

    /// Returns `true` if this `Value` represents a map.
    pub fn is_map(&self) -> bool {
        matches!(self, Self::Map(_))
    }
}

impl Value {
    /// Returns `Some` if this `Value` represents a null value, otherwise returns `None`.
    pub fn as_null(&self) -> Option<()> {
        match self {
            Self::Null => Some(()),
            _ => None,
        }
    }

    /// Returns the underlying boolean, if that's what this `Value` represents.
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Self::Boolean(v) => Some(*v),
            _ => None,
        }
    }

    /// Returns the underlying number, if that's what this `Value` represents.
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(v) => Some(*v),
            _ => None,
        }
    }

    /// Returns a reference to the underlying string, if that's what this `Value` represents.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(v) => Some(v),
            _ => None,
        }
    }

    /// Returns a reference to the underlying list, if that's what this `Value` represents.
    pub fn as_slice(&self) -> Option<&[Self]> {
        match self {
            Self::List(v) => Some(v),
            _ => None,
        }
    }

    /// Returns a reference to the underlying map, if that's what this `Value` represents.
    pub fn as_map(&self) -> Option<&BTreeMap<String, Self>> {
        match self {
            Self::Map(v) => Some(v),
            _ => None,
        }
    }
}

impl Value {
    /// Consumes the `Value`, returning the contained string if it represented one.
    pub fn into_string(self) -> Option<String> {
        match self {
            Self::String(v) => Some(v),
            _ => None,
        }
    }

    /// Consumes the `Value`, returning the contained list if it represented one.
    pub fn into_list(self) -> Option<Vec<Self>> {
        match self {
            Self::List(v) => Some(v),
            _ => None,
        }
    }

    /// Consumes the `Value`, returning the contained map if it represented one.
    pub fn into_map(self) -> Option<BTreeMap<String, Self>> {
        match self {
            Self::Map(v) => Some(v),
            _ => None,
        }
    }
}

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Self::Null
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<u8> for Value {
    fn from(value: u8) -> Self {
        Self::Number(value.into())
    }
}

impl From<i8> for Value {
    fn from(value: i8) -> Self {
        Self::Number(value.into())
    }
}

impl From<u16> for Value {
    fn from(value: u16) -> Self {
        Self::Number(value.into())
    }
}

impl From<i16> for Value {
    fn from(value: i16) -> Self {
        Self::Number(value.into())
    }
}

impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Self::Number(value.into())
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Self::Number(value.into())
    }
}

impl From<u64> for Value {
    fn from(value: u64) -> Self {
        Self::Number(value as f64)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self::Number(value as f64)
    }
}

impl From<usize> for Value {
    fn from(value: usize) -> Self {
        Self::Number(value as f64)
    }
}

impl From<isize> for Value {
    fn from(value: isize) -> Self {
        Self::Number(value as f64)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Self::Number(value.into())
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl<V: Clone + Into<Value>> From<&[V]> for Value {
    fn from(value: &[V]) -> Self {
        Self::List(value.into_iter().map(|v| v.clone().into()).collect())
    }
}

impl<V: Into<Value>> From<Vec<V>> for Value {
    fn from(value: Vec<V>) -> Self {
        Self::List(value.into_iter().map(|v| v.into()).collect())
    }
}

impl<V: Into<Value>> FromIterator<V> for Value {
    fn from_iter<T: IntoIterator<Item = V>>(iter: T) -> Self {
        Self::List(iter.into_iter().map(|v| v.into()).collect())
    }
}

impl<K: Clone + Into<String>, V: Clone + Into<Value>> From<&[(K, V)]> for Value {
    fn from(value: &[(K, V)]) -> Self {
        Self::Map(
            value
                .into_iter()
                .map(|(k, v)| (k.clone().into(), v.clone().into()))
                .collect(),
        )
    }
}

impl<V: Into<Value>> From<BTreeMap<String, V>> for Value {
    fn from(value: BTreeMap<String, V>) -> Self {
        Self::Map(value.into_iter().map(|(k, v)| (k, v.into())).collect())
    }
}

impl<K: Into<String>, V: Into<Value>> FromIterator<(K, V)> for Value {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self::Map(
            iter.into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        )
    }
}

impl<V: Into<Value>> From<Option<V>> for Value {
    fn from(value: Option<V>) -> Self {
        match value {
            Some(v) => v.into(),
            None => Self::Null,
        }
    }
}

impl core::fmt::Display for Value {
    /// Display a `Value` as a string.
    ///
    /// For simplicity, the JSON string representation is chosen.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use alloc::string::ToString;
        use core::fmt::Write;

        match self {
            Self::Null => f.write_str("null"),
            Self::Boolean(v) => f.write_str(v.to_string().as_str()),
            Self::Number(v) => {
                f.write_char('"')?;
                f.write_str(v.to_string().as_str())?;
                f.write_char('"')?;
                Ok(())
            }
            Self::String(v) => {
                f.write_char('"')?;
                f.write_str(v.replace('\\', r#"\\"#).replace('"', r#"\""#).as_str())?;
                f.write_char('"')?;
                Ok(())
            }
            Self::List(v) => {
                f.write_char('[')?;
                for x in v {
                    f.write_str(x.to_string().as_str())?;
                    f.write_char(',')?;
                }
                f.write_char(']')?;
                Ok(())
            }
            Self::Map(v) => {
                f.write_char('{')?;
                for (k, x) in v {
                    f.write_char('"')?;
                    f.write_str(k.as_str())?;
                    f.write_char('"')?;
                    f.write_char(':')?;
                    f.write_str(x.to_string().as_str())?;
                    f.write_char(',')?;
                }
                f.write_char('}')?;
                Ok(())
            }
        }
    }
}
