use core::fmt;

use serde::{de, ser};

/// Represents an ordered list.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct OrderedList<T>(Vec<T>)
where
    T: Clone + fmt::Debug + Eq + PartialEq;

impl<T> OrderedList<T>
where
    T: Clone + fmt::Debug + Eq + PartialEq,
{
    /// Creates a new [OrderedList].
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    /// Creates a new [OrderedList] with the provided capacity.
    pub fn with_capacity(cap: usize) -> Self {
        Self(Vec::with_capacity(cap))
    }

    /// Gets an iterator over the [OrderedList].
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter()
    }

    /// Removes an item to the [OrderedList].
    pub fn pop(&mut self) -> Option<T> {
        self.0.pop()
    }

    /// Constructs an [OrderedList] from a list of items.
    ///
    /// # Invariants
    ///
    /// Caller must ensure items are unique, and in the desired order.
    pub fn from_items<TI, I>(items: I) -> Self
    where
        TI: Into<T>,
        I: IntoIterator<Item = TI>,
    {
        Self(items.into_iter().map(|t| t.into()).collect())
    }
}

impl<T> OrderedList<T>
where
    T: Clone + fmt::Debug + Eq + PartialEq + Ord + PartialOrd,
{
    /// Adds an item to the [OrderedList].
    pub fn push(&mut self, val: T) {
        if !self.0.iter().any(|t| t == &val) {
            self.0.push(val);
            self.0.sort();
        }
    }

    /// Converts a list into an [OrderedList].
    pub fn from_list<TI, I>(list: I) -> Self
    where
        TI: Into<T>,
        I: IntoIterator<Item = TI>,
    {
        let mut list: Vec<T> = list.into_iter().map(|i| i.into()).collect();
        list.sort();
        list.dedup();
        Self(list)
    }
}

impl<T> Default for OrderedList<T>
where
    T: Clone + fmt::Debug + Eq + PartialEq,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> core::fmt::Display for OrderedList<T>
where
    T: Clone + fmt::Debug + Eq + PartialEq + ser::Serialize,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        serde_json::to_string(self)
            .map_err(|_| core::fmt::Error)
            .and_then(|s| write!(f, "{s}"))
    }
}

impl<T> AsRef<[T]> for OrderedList<T>
where
    T: Clone + fmt::Debug + Eq + PartialEq,
{
    fn as_ref(&self) -> &[T] {
        self.0.as_ref()
    }
}

impl<T> IntoIterator for OrderedList<T>
where
    T: Clone + fmt::Debug + Eq + PartialEq,
{
    type Item = T;
    type IntoIter = <Vec<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T> core::iter::FromIterator<T> for OrderedList<T>
where
    T: Clone + fmt::Debug + Eq + PartialEq,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(Vec::<T>::from_iter(iter))
    }
}

impl<T> ser::Serialize for OrderedList<T>
where
    T: Clone + fmt::Debug + Eq + PartialEq + ser::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de, T> de::Deserialize<'de> for OrderedList<T>
where
    T: Clone + fmt::Debug + Eq + PartialEq + de::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        Vec::deserialize(deserializer).map(Self)
    }
}
