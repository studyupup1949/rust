use serde::{Deserialize, Serialize};

use crate::{Iri, field_access};

/// Represents an optional set of endpoints for an [Actor](crate::ActorType).
///
/// A JSON object which maps additional (typically server/domain-wide) endpoints which may be useful either for this actor or someone referencing this actor.
///
/// This mapping may be nested inside the actor document as the value or may be a link to a JSON-LD document with these properties.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Endpoints {
    #[serde(skip_serializing_if = "Option::is_none")]
    proxy_url: Option<Iri>,
    #[serde(skip_serializing_if = "Option::is_none")]
    oauth_authorized_endpoint: Option<Iri>,
    #[serde(skip_serializing_if = "Option::is_none")]
    oauth_token_endpoint: Option<Iri>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provide_client_key: Option<Iri>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sign_client_key: Option<Iri>,
    #[serde(skip_serializing_if = "Option::is_none")]
    shared_inbox: Option<Iri>,
}

field_access! {
    Endpoints {
        /// Endpoint URI so this actor's clients may access remote ActivityStreams objects which require authentication to access.
        ///
        /// To use this endpoint, the client posts an `x-www-form-urlencoded` `id` parameter with the value being the `id` of the requested ActivityStreams object.
        proxy_url: option_ref { Iri },
        /// If OAuth 2.0 bearer tokens [RFC6749](https://tools.ietf.org/html/rfc6749) [RFC6750](https://tools.ietf.org/html/rfc6750) are being used for authenticating [client to server interactions](https://www.w3.org/TR/activitypub/#client-to-server-interactions), this endpoint specifies a URI at which a browser-authenticated user may obtain a new authorization grant.
        oauth_authorized_endpoint: option_ref { Iri },
        /// If OAuth 2.0 bearer tokens [RFC6749](https://tools.ietf.org/html/rfc6749) [RFC6750](https://tools.ietf.org/html/rfc6750) are being used for authenticating [client to server interactions](https://www.w3.org/TR/activitypub/#client-to-server-interactions), this endpoint specifies a URI at which a client may acquire an access token.
        oauth_token_endpoint: option_ref { Iri },
        /// If Linked Data Signatures and HTTP Signatures are being used for authentication and authorization, this endpoint specifies a URI at which browser-authenticated users may authorize a client's public key for [client to server interactions](https://www.w3.org/TR/activitypub/#client-to-server-interactions).
        provide_client_key: option_ref { Iri },
        /// If Linked Data Signatures and HTTP Signatures are being used for authentication and authorization, this endpoint specifies a URI at which a client key may be signed by the actor's key for a time window to act on behalf of the actor in interacting with foreign servers.
        sign_client_key: option_ref { Iri },
        /// An optional endpoint used for wide delivery of publicly addressed activities and activities sent to followers.
        ///
        /// `sharedInbox` endpoints *SHOULD* also be publicly readable [OrderedCollection](crate::OrderedCollection) objects containing objects addressed to the Public special collection.
        ///
        /// Reading from the sharedInbox endpoint *MUST NOT* present objects which are not addressed to the `Public` endpoint.
        shared_inbox: option_ref { Iri },
    }
}
