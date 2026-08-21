/*
 * This file is part of ActivityPub.
 *
 * Copyright © 2018 Riley Trautman
 *
 * ActivityPub is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * ActivityPub is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with ActivityPub.  If not, see <http://www.gnu.org/licenses/>.
 */

//! Namespace for properties of standard Object types
//!
//! To use these properties in your own types, you can flatten them into your struct with serde:
//!
//! ```rust
//! use activitypub::{
//!     object::properties::{
//!         ApObjectProperties,
//!         ObjectProperties,
//!     },
//!     Object,
//!     PropRefs,
//! };
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Clone, Debug, Serialize, Deserialize, PropRefs)]
//! #[serde(rename_all = "camelCase")]
//! pub struct MyObject {
//!     #[serde(rename = "type")]
//!     pub kind: String,
//!
//!     /// Define a require property for the MyObject type
//!     pub my_property: String,
//!
//!     #[serde(flatten)]
//!     #[activitystreams(Object)]
//!     pub object_props: ObjectProperties,
//!
//!     #[serde(flatten)]
//!     #[activitystreams(None)]
//!     pub ap_object_props: ApObjectProperties,
//! }
//! #
//! # fn main() {}
//! ```

use activitystreams::properties;

pub use activitystreams::{
    object::{
        properties::{
            ObjectProperties, PlaceProperties, ProfileProperties, RelationshipProperties,
            TombstoneProperties,
        },
        ObjectBox,
    },
    primitives::XsdAnyUri,
};

properties! {
    ApObject {
        docs [
            "Define activitypub properties for the Object type as described by the Activity Pub vocabulary.",
        ],

        shares {
            docs [
                "This is a list of all Announce activities with this object as the object property, added as",
                "a side effect.",
                "",
                "The shares collection MUST be either an OrderedCollection or a Collection and MAY be",
                "filtered on privileges of an authenticated user or as appropriate when no authentication is",
                "given.",
                "",
                "- Range: `anyUri`",
                "- Functional: true",
            ],
            types [ XsdAnyUri ],
            functional,
        },

        likes {
            docs [
                "This is a list of all Like activities with this object as the object property, added as a",
                "side effect.",
                "",
                "The likes collection MUST be either an OrderedCollection or a Collection and MAY be",
                "filtered on privileges of an authenticated user or as appropriate when no authentication is",
                "given.",
                "",
                "- Range: `anyUri`",
                "- Functional: true",
            ],
            types [ XsdAnyUri ],
            functional,
        },

        source {
            docs [
                "The source property is intended to convey some sort of source from which the content markup",
                "was derived, as a form of provenance, or to support future editing by clients.",
                "",
                "In general, clients do the conversion from source to content, not the other way around.",
                "",
                "The value of source is itself an object which uses its own content and mediaType fields to",
                "supply source information.",
                "",
                "- Range: `Object`",
                "- Functional: true",
            ],
            types [
                XsdAnyUri,
                ObjectBox,
            ],
            functional,
        },

        upload_media {
            docs [
                "Servers MAY support uploading document types to be referenced in activites, such as images,",
                "video or other binary data, but the precise mechanism is out of scope for this version of",
                "`ActivityPub`.",
                "",
                "The Social Web Community Group is refining the protocol in the",
                "[`ActivityPub` Media Upload report](https://www.w3.org/wiki/SocialCG/ActivityPub/MediaUpload).",
                "",
                "- Range: `anyUri`",
                "- Functional: false",
            ],
            types [ XsdAnyUri ],
        },
    }
}
