/// Helper macro to define ActivityStream Actor & Actor-derived types.
#[macro_export]
macro_rules! create_actor {
    (
        $(#[$doc:meta])*
        $ty:ident {
        $(
            $(#[$field_serde:meta])*
            $field:ident: $field_ty:ty $(,)?
        )*
    }) => {
        $crate::create_actor! {
            $(#[$doc])*
            $ty: $crate::ActorType::$ty {
                $(
                    $(#[$field_serde])*
                    $field: $field_ty,
                )*
            }
        }
    };

    (
        $(#[$doc:meta])*
        $ty:ident:
        $actor_ty:ident {
        $(
            $(#[$field_serde:meta])*
            $field:ident: $field_ty:ty $(,)?
        )*
    }) => {
        $crate::create_actor! {
            $(#[$doc])*
            $ty:
            $crate::$actor_ty::$ty {
                $(
                    $(#[$field_serde])*
                    $field: $field_ty,
                )*
            }
        }
    };

    (
        $(#[$doc:meta])*
        $ty:ident:
        $actor_ty:ident :: $actor_var:ident {
        $(
            $(#[$field_serde:meta])*
            $field:ident: $field_ty:ty $(,)?
        )*
    }) => {
        $crate::create_actor! {
            $(#[$doc])*
            $ty:
            $crate::$actor_ty::$actor_var {
                $(
                    $(#[$field_serde])*
                    $field: $field_ty,
                )*
            }
        }
    };

    (
        $(#[$doc:meta])*
        $ty:ident:
        $actor_path:ident :: $actor_ty:ident :: $actor_var:ident {
        $(
            $(#[$field_serde:meta])*
            $field:ident: $field_ty:ty $(,)?
        )*
    }) => {
        $crate::paste! {
            $crate::create_object! {
                $(#[$doc])*
                $ty: $actor_path::$actor_ty::$actor_var {
                    #[serde(skip_serializing_if = "Option::is_none")]
                    inbox: Option<$crate::Item>,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    outbox: Option<$crate::Item>,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    following: Option<$crate::Item>,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    followers: Option<$crate::Item>,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    liked: Option<$crate::Item>,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    streams: Option<$crate::Items>,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    preferred_username: Option<$crate::Name>,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    endpoints: Option<$crate::Endpoints>,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    assertion_method: Option<$crate::MultikeyItems>,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    public_key: Option<$crate::KeyItems>,
                    $(
                        $(#[$field_serde])*
                        $field: $field_ty,
                    )*
                }
            }

            $crate::field_access! {
                $ty<Vocab> {
                    /// A reference to an [ActivityStreams](https://www.w3.org/TR/activitystreams-core/) [OrderedCollection](crate::OrderedCollection) comprised of all the messages received by the actor; see [5.2 Inbox](https://www.w3.org/TR/activitypub/#inbox).
                    inbox: option_ref { $crate::Item },
                    /// An [ActivityStreams](https://www.w3.org/TR/activitystreams-core/) [OrderedCollection](crate::OrderedCollection) comprised of all the messages produced by the actor; see [5.1 Outbox](https://www.w3.org/TR/activitypub/#outbox).
                    outbox: option_ref { $crate::Item },
                    /// A link to an [ActivityStreams](https://www.w3.org/TR/activitystreams-core/) collection of the actors that this actor is following; see [5.4 Following Collection](https://www.w3.org/TR/activitypub/#following).
                    following: option_ref { $crate::Item },
                    /// A link to an [ActivityStreams](https://www.w3.org/TR/activitystreams-core/) collection of the actors that follow this actor; see [5.3 Followers Collection](https://www.w3.org/TR/activitypub/#followers).
                    followers: option_ref { $crate::Item },
                    /// A link to an [ActivityStreams](https://www.w3.org/TR/activitystreams-core/) collection of objects this actor has liked; see [5.5 Liked Collection](https://www.w3.org/TR/activitypub/#liked).
                    liked: option_ref { $crate::Item },
                    /// A list of supplementary Collections which may be of interest.
                    streams: option_ref { $crate::Items },
                    /// A short username which may be used to refer to the actor, with no uniqueness guarantees.
                    preferred_username: option_ref { $crate::Name },
                    /// A json object which maps additional (typically server/domain-wide) endpoints which may be useful either for this actor or someone referencing this actor.
                    ///
                    /// This mapping may be nested inside the actor document as the value or may be a link to a JSON-LD document with these properties.
                    endpoints: option_ref { $crate::Endpoints },
                    /// A list of public key representations following the [FEP-521a](https://codeberg.org/fediverse/fep/src/branch/main/fep/521a/fep-521a.md) specification.
                    assertion_method: option_ref { $crate::MultikeyItems },
                    /// Public key used for HTTP Signatures and Linked Data Signatures.
                    #[deprecated(since = "0.3.0", note = "The `publicKey` vocabulary has been deprecated since Security Vocabulary 2.0. Users should use the `assertionMethod` field instead, where possible.")]
                    public_key: option_ref { $crate::KeyItems },
                }
            }
        }
    };
}
