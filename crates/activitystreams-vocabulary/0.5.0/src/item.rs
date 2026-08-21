use crate::{Iri, Link, Object, create_item, create_list};

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

create_list! {
    /// Represents the ActivityStream
    /// [Items](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-items) type.
    Items: Item,
}

create_list! {
    /// Represents an ordered list of ActivityStream
    /// [Items](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-items) type.
    OrderedItems: ordered { Item },
}
