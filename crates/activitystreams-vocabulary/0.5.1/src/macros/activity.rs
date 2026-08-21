/// Helper macro to define ActivityStream Activity & Activity-derived types.
#[macro_export]
macro_rules! create_activity {
    (
        $(#[$doc:meta])*
        $ty:ident {
        $(
            $(#[$field_serde:meta])*
            $field:ident: $field_ty:ty $(,)?
        )*
    }) => {
        $crate::create_activity! {
            $(#[$doc])*
            $ty: ActivityType {
                $(
                    $(#[$field_serde])*
                    $field: $field_ty,
                )*
            }
        }
    };

    (
        $(#[$doc:meta])*
        $ty:ident: $activity_ty:ident {
        $(
            $(#[$field_serde:meta])*
            $field:ident: $field_ty:ty $(,)?
        )*
    }) => {
        $crate::create_activity! {
            $(#[$doc])*
            $ty: $crate::$activity_ty::$ty {
                $(
                    $(#[$field_serde])*
                    $field: $field_ty,
                )*
            }
        }
    };

    (
        $(#[$doc:meta])*
        $ty:ident: $activity_path:ident :: $activity_ty:ident :: $activity_var:ident {
        $(
            $(#[$field_serde:meta])*
            $field:ident: $field_ty:ty $(,)?
        )*
    }) => {
        $crate::paste! {
            $crate::create_object! {
                $(#[$doc])*
                $ty: $activity_path::$activity_ty::$activity_var {
                    #[serde(skip_serializing_if = "Option::is_none")]
                    actor: Option<$crate::Items>,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    object: Option<$crate::Items>,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    origin: Option<$crate::Items>,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    target: Option<$crate::Items>,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    instrument: Option<$crate::Items>,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    result: Option<$crate::Items>,
                    $(
                        $(#[$field_serde])*
                        $field: $field_ty,
                    )*
                }
            }

            $crate::field_access! {
                $ty<Vocab> {
                    /// Describes one or more entities that either performed or are expected to perform the activity.
                    ///
                    /// Any single activity can have multiple `actor`s.
                    ///
                    /// The `actor` **MAY** be specified using an indirect Link.
                    actor: option_ref { $crate::Items },
                    /// Describes the direct object of the activity.
                    ///
                    /// For instance, in the activity "John added a movie to his wishlist", the object of the activity is the movie added.
                    object: option_ref { $crate::Items },
                    /// Describes an indirect object of the activity from which the activity is directed.
                    ///
                    /// The precise meaning of the origin is the object of the English preposition "from".
                    ///
                    /// For instance, in the activity "John moved an item to List B from List A",
                    /// the origin of the activity is "List A".
                    origin: option_ref { $crate::Items },
                    /// Describes the indirect object, or target, of the activity.
                    ///
                    /// The precise meaning of the target is largely dependent on the type of action being described
                    /// but will often be the object of the English preposition "to".
                    ///
                    /// For instance, in the activity "John added a movie to his wishlist",
                    /// the target of the activity is John's wishlist.
                    ///
                    /// An activity can have more than one target.
                    target: option_ref { $crate::Items },
                    /// Identifies one or more objects used (or to be used) in the completion of an [Activity](crate::Activity).
                    instrument: option_ref { $crate::Items },
                    /// Describes the result of the activity.
                    ///
                    /// For instance, if a particular action results in the creation of a new resource,
                    /// the result property can be used to describe that new resource.
                    result: option_ref { $crate::Items },
                }
            }
        }
    };
}

/// Helper macro to define ActivityStream IntransitiveActivity & IntransitiveActivity-derived types.
#[macro_export]
macro_rules! create_intransitive_activity {
    (
        $(#[$doc:meta])*
        $ty:ident {
        $(
            $(#[$field_serde:meta])*
            $field:ident: $field_ty:ty $(,)?
        )*
    }) => {
        $crate::create_intransitive_activity! {
            $(#[$doc])*
            $ty: ActivityType {
                $(
                    $(#[$field_serde])*
                    $field: $field_ty,
                )*
            }
        }
    };

    (
        $(#[$doc:meta])*
        $ty:ident: $activity_ty:ident {
        $(
            $(#[$field_serde:meta])*
            $field:ident: $field_ty:ty $(,)?
        )*
    }) => {
        $crate::paste! {
            $crate::create_object! {
                $(#[$doc])*
                $ty: $activity_ty {
                    #[serde(skip_serializing_if = "Option::is_none")]
                    actor: Option<$crate::Item>,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    target: Option<$crate::Item>,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    result: Option<$crate::Item>,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    origin: Option<$crate::Item>,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    instrument: Option<$crate::Item>,
                    $(
                        $(#[$field_serde])*
                        $field: $field_ty,
                    )*
                }
            }

            $crate::field_access! {
                $ty<Vocab> {
                    /// Describes one or more entities that either performed or are expected to perform the activity.
                    ///
                    /// Any single activity can have multiple `actor`s.
                    ///
                    /// The `actor` **MAY** be specified using an indirect Link.
                    actor: option_ref { $crate::Item },
                    /// Describes the indirect object, or target, of the activity.
                    ///
                    /// The precise meaning of the target is largely dependent on the type of action being described
                    /// but will often be the object of the English preposition "to".
                    ///
                    /// For instance, in the activity "John added a movie to his wishlist",
                    /// the target of the activity is John's wishlist.
                    ///
                    /// An activity can have more than one target.
                    target: option_ref { $crate::Item },
                    /// Describes the result of the activity.
                    ///
                    /// For instance, if a particular action results in the creation of a new resource,
                    /// the result property can be used to describe that new resource.
                    result: option_ref { $crate::Item },
                    /// Describes an indirect object of the activity from which the activity is directed.
                    ///
                    /// The precise meaning of the origin is the object of the English preposition "from".
                    ///
                    /// For instance, in the activity "John moved an item to List B from List A",
                    /// the origin of the activity is "List A".
                    origin: option_ref { $crate::Item },
                    /// Identifies one or more objects used (or to be used) in the completion of an [Activity](crate::Activity).
                    instrument: option_ref { $crate::Item },
                }
            }
        }
    };
}
