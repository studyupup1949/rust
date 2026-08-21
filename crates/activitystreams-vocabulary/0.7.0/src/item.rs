use crate::{Error, Iri, Link, Object, Result, VocabularyType, create_item, create_list};

create_item! {
    /// Represents the ActivityStream range of [Iri], [Object], or [Link] types.
    Item
        base
        boxed
        default: Self::Object(Box::new(Object::new_inner())),
    {
        Object(Object),
        Link(Link),
        Iri(Iri),
    }
}

impl Item {
    /// Gets the ID for the [Item].
    pub fn id(&self) -> Result<&Iri> {
        match self {
            Self::Object(object) => object.id().ok_or(Error::item("missing object ID")),
            Self::Iri(iri) => Ok(iri.as_ref()),
            Self::Link(link) => Ok(link.href()),
        }
    }

    /// Gets whether the [Item] contains an [Object] with the provided vocabulary type.
    pub fn contains<T: Into<VocabularyType>>(&self, t: T) -> bool {
        match self {
            Self::Object(object) => object.kind().contains(t),
            _ => false,
        }
    }
}

create_list! {
    /// Represents the ActivityStream
    /// [Items](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-items) type.
    Items: Item,
}

impl Items {
    /// Gets the IDs for the [Items].
    pub fn ids(&self) -> Result<Vec<&Iri>> {
        match self {
            Self::Single(item) => item.id().map(|i| vec![i]),
            Self::List(list) => list.iter().map(|i| i.id()).collect::<Result<Vec<_>>>(),
        }
    }

    /// Gets whether the [Items] contain an [Object] with the provided vocabulary type.
    pub fn contains<T: Into<VocabularyType> + Clone>(&self, t: T) -> bool {
        match self {
            Self::Single(Item::Object(object)) => object.kind().contains(t),
            Self::List(list) => list.iter().any(|i| i.contains(t.clone())),
            _ => false,
        }
    }
}

create_list! {
    /// Represents an ordered list of ActivityStream
    /// [Items](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-items) type.
    OrderedItems: ordered { Item },
}

impl OrderedItems {
    /// Gets the IDs for the [OrderedItems].
    pub fn ids(&self) -> Result<Vec<&Iri>> {
        match self {
            Self::Single(item) => item.id().map(|i| vec![i]),
            Self::List(list) => list.iter().map(|i| i.id()).collect::<Result<Vec<_>>>(),
        }
    }

    /// Gets whether the [OrderedItems] contain an [Object] with the provided vocabulary type.
    pub fn contains<T: Into<VocabularyType> + Clone>(&self, t: T) -> bool {
        match self {
            Self::Single(Item::Object(object)) => object.kind().contains(t),
            Self::List(list) => list.iter().any(|i| i.contains(t.clone())),
            _ => false,
        }
    }
}
