use serde::{Deserialize, Serialize};

use crate::{Error, Iri, Link, Object, OrderedList, Result, impl_default, impl_display};

/// Represents the ActivityStream range of [Object] or [Link] types.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Item {
    Object(Box<Object>),
    Link(Box<Link>),
    Iri(Box<Iri>),
}

impl Item {
    /// Creates a new [Item].
    pub fn new() -> Self {
        Self::Object(Box::new(Object::new_inner()))
    }

    /// Gets whether the [Item] is an [Object] variant.
    #[inline]
    pub const fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }

    /// Attempts to convert the [Item] to an [Object].
    pub fn as_object(&self) -> Result<&Object> {
        match self {
            Self::Object(ty) => Ok(ty.as_ref()),
            ty => Err(Error::item(format!("invalid item object type: {ty}"))),
        }
    }

    /// Gets whether the [Item] is an [Link] variant.
    #[inline]
    pub const fn is_link(&self) -> bool {
        matches!(self, Self::Link(_))
    }

    /// Attempts to convert the [Item] to an [Link].
    pub fn as_link(&self) -> Result<&Link> {
        match self {
            Self::Link(ty) => Ok(ty.as_ref()),
            ty => Err(Error::item(format!("invalid item link type: {ty}"))),
        }
    }

    /// Gets whether the [Item] is an [Iri] variant.
    #[inline]
    pub const fn is_iri(&self) -> bool {
        matches!(self, Self::Iri(_))
    }

    /// Attempts to convert the [Item] to an [Iri].
    pub fn as_iri(&self) -> Result<&Iri> {
        match self {
            Self::Iri(ty) => Ok(ty.as_ref()),
            ty => Err(Error::item(format!("invalid item iri type: {ty}"))),
        }
    }
}

impl_default!(Item);
impl_display!(Item, json);

/// Represents the ActivityStream
/// [Items](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-items) type.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Items {
    Single(Item),
    List(Vec<Item>),
}

impl Items {
    /// Creates a new [Items].
    pub fn new() -> Self {
        Self::Single(Item::new())
    }

    /// Gets whether the [Items] contains a [Single](Self::Single) variant.
    pub const fn is_single(&self) -> bool {
        matches!(self, Self::Single(_))
    }

    /// Attempts to get a reference to the [Single](Self::Single) variant.
    pub fn as_single(&self) -> Result<&Item> {
        match self {
            Self::Single(ty) => Ok(ty),
            _ => Err(Error::item("invalid items type")),
        }
    }

    /// Gets whether the [Items] contains a [List](Self::List) variant.
    pub const fn is_list(&self) -> bool {
        matches!(self, Self::List(_))
    }

    /// Attempts to get a reference to the [List](Self::List) variant.
    pub fn as_list(&self) -> Result<&[Item]> {
        match self {
            Self::List(tys) => Ok(tys),
            _ => Err(Error::item("invalid items type")),
        }
    }
}

impl_default!(Items);
impl_display!(Items, json);

/// Represents the ActivityStream
/// [Items](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-items) type.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum OrderedItems {
    Single(Item),
    List(OrderedList<Item>),
}

impl OrderedItems {
    /// Creates a new [OrderedItems].
    pub fn new() -> Self {
        Self::Single(Item::new())
    }

    /// Creates a new [OrderedItems] list variant.
    pub fn new_list() -> Self {
        Self::List(OrderedList::new())
    }

    /// Creates a [OrderedItems] [Single](Self::Single) variant.
    pub fn single<I: Into<Item>>(val: I) -> Self {
        Self::Single(val.into())
    }

    /// Gets whether the [OrderedItems] contains a [Single](Self::Single) variant.
    pub const fn is_single(&self) -> bool {
        matches!(self, Self::Single(_))
    }

    /// Attempts to get a reference to the [Single](Self::Single) variant.
    pub fn as_single(&self) -> Result<&Item> {
        match self {
            Self::Single(ty) => Ok(ty),
            _ => Err(Error::item("invalid items type")),
        }
    }

    /// Creates a [OrderedItems] [List](Self::List) variant.
    pub fn list<T, I>(val: I) -> Self
    where
        T: Into<Item>,
        I: IntoIterator<Item = T>,
    {
        Self::List(OrderedList::from_items(val))
    }

    /// Gets whether the [OrderedItems] contains a [List](Self::List) variant.
    pub const fn is_list(&self) -> bool {
        matches!(self, Self::List(_))
    }

    /// Attempts to get a reference to the [List](Self::List) variant.
    pub fn as_list(&self) -> Result<&[Item]> {
        match self {
            Self::List(tys) => Ok(tys.as_ref()),
            _ => Err(Error::item("invalid items type")),
        }
    }
}

impl_default!(OrderedItems);
impl_display!(OrderedItems, json);

impl From<Item> for OrderedItems {
    fn from(val: Item) -> Self {
        Self::Single(val)
    }
}

impl From<Vec<Item>> for OrderedItems {
    fn from(val: Vec<Item>) -> Self {
        Self::list(val)
    }
}

impl From<&[Item]> for OrderedItems {
    fn from(val: &[Item]) -> Self {
        Self::list(val.into_iter().cloned())
    }
}

impl<const N: usize> From<&[Item; N]> for OrderedItems {
    fn from(val: &[Item; N]) -> Self {
        Self::list(val.into_iter().cloned())
    }
}

impl<const N: usize> From<[Item; N]> for OrderedItems {
    fn from(val: [Item; N]) -> Self {
        Self::list(val)
    }
}
