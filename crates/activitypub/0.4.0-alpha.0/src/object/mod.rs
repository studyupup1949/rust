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

//! Object traits and types

pub use activitystreams::{object::kind, Object, PropRefs};
use serde::{Deserialize, Serialize};

pub mod properties;

use self::{kind::*, properties::*};

#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
pub struct Article {
    #[serde(rename = "type")]
    kind: ArticleType,

    #[serde(flatten)]
    #[activitystreams(Object)]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[activitystreams(None)]
    pub ap_object_props: ApObjectProperties,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
pub struct Audio {
    #[serde(rename = "type")]
    kind: AudioType,

    #[serde(flatten)]
    #[activitystreams(Object)]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[activitystreams(None)]
    pub ap_object_props: ApObjectProperties,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    #[serde(rename = "type")]
    kind: DocumentType,

    #[serde(flatten)]
    #[activitystreams(Object)]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[activitystreams(None)]
    pub ap_object_props: ApObjectProperties,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    #[serde(rename = "type")]
    kind: EventType,

    #[serde(flatten)]
    #[activitystreams(Object)]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[activitystreams(None)]
    pub ap_object_props: ApObjectProperties,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    #[serde(rename = "type")]
    kind: ImageType,

    #[serde(flatten)]
    #[activitystreams(Object)]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[activitystreams(None)]
    pub ap_object_props: ApObjectProperties,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    #[serde(rename = "type")]
    kind: NoteType,

    #[serde(flatten)]
    #[activitystreams(Object)]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[activitystreams(None)]
    pub ap_object_props: ApObjectProperties,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    #[serde(rename = "type")]
    kind: PageType,

    #[serde(flatten)]
    #[activitystreams(Object)]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[activitystreams(None)]
    pub ap_object_props: ApObjectProperties,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
pub struct Place {
    #[serde(rename = "type")]
    kind: PlaceType,

    #[serde(flatten)]
    #[activitystreams(Object)]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[activitystreams(None)]
    pub ap_object_props: ApObjectProperties,

    #[serde(flatten)]
    #[activitystreams(None)]
    pub place_props: PlaceProperties,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    #[serde(rename = "type")]
    kind: ProfileType,

    #[serde(flatten)]
    #[activitystreams(Object)]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[activitystreams(None)]
    pub ap_object_props: ApObjectProperties,

    #[serde(flatten)]
    #[activitystreams(None)]
    pub profile_props: ProfileProperties,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
pub struct Relationship {
    #[serde(rename = "type")]
    kind: RelationshipType,

    #[serde(flatten)]
    #[activitystreams(Object)]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[activitystreams(None)]
    pub ap_object_props: ApObjectProperties,

    #[serde(flatten)]
    #[activitystreams(None)]
    pub relationship_props: RelationshipProperties,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
pub struct Tombstone {
    #[serde(rename = "type")]
    kind: TombstoneType,

    #[serde(flatten)]
    #[activitystreams(Object)]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[activitystreams(None)]
    pub ap_object_props: ApObjectProperties,

    #[serde(flatten)]
    #[activitystreams(None)]
    pub tombstone_props: TombstoneProperties,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
pub struct Video {
    #[serde(rename = "type")]
    kind: VideoType,

    #[serde(flatten)]
    #[activitystreams(Object)]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[activitystreams(None)]
    pub ap_object_props: ApObjectProperties,
}
