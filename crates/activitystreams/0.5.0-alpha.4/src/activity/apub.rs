/*
 * This file is part of ActivityStreams.
 *
 * Copyright © 2020 Riley Trautman
 *
 * ActivityStreams is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * ActivityStreams is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with ActivityStreams.  If not, see <http://www.gnu.org/licenses/>.
 */

//! Activity traits and types

use crate::{
    activity::{
        kind::*, properties::*, Activity, ActivityBox, IntransitiveActivity,
        IntransitiveActivityBox,
    },
    object::{
        properties::{ApObjectProperties, ObjectProperties},
        Object, ObjectBox,
    },
    PropRefs,
};
use serde::{Deserialize, Serialize};

/// Indicates that the actor accepts the object.
///
/// The target property can be used in certain circumstances to indicate the context into which the
/// object has been accepted.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
#[prop_refs(Object)]
#[prop_refs(Activity)]
pub struct Accept {
    #[serde(rename = "type")]
    kind: AcceptType,

    #[serde(flatten)]
    #[prop_refs]
    pub accept_props: AcceptProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub ap_object_props: ApObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub activity_props: ActivityProperties,
}

/// Indicates that the actor has added the object to the target.
///
/// If the target property is not explicitly specified, the target would need to be determined
/// implicitly by context. The origin can be used to identify the context from which the object
/// originated.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
#[prop_refs(Object)]
#[prop_refs(Activity)]
pub struct Add {
    #[serde(rename = "type")]
    kind: AddType,

    #[serde(flatten)]
    #[prop_refs]
    pub add_props: AddProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub ap_object_props: ApObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub activity_props: ActivityProperties,
}

/// Indicates that the actor has moved object from origin to target.
///
/// If the origin or target are not specified, either can be determined by context.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
#[prop_refs(Object)]
#[prop_refs(Activity)]
pub struct AMove {
    #[serde(rename = "type")]
    kind: MoveType,

    #[serde(flatten)]
    #[prop_refs]
    pub move_props: MoveProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub ap_object_props: ApObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub activity_props: ActivityProperties,
}

/// Indicates that the actor is calling the target's attention the object.
///
/// The origin typically has no defined meaning.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
#[prop_refs(Object)]
#[prop_refs(Activity)]
pub struct Announce {
    #[serde(rename = "type")]
    kind: AnnounceType,

    #[serde(flatten)]
    #[prop_refs]
    pub announce_props: AnnounceProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub ap_object_props: ApObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub activity_props: ActivityProperties,
}

/// An IntransitiveActivity that indicates that the actor has arrived at the location.
///
/// The origin can be used to identify the context from which the actor originated. The target
/// typically has no defined meaning.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
#[prop_refs(Object)]
#[prop_refs(Activity)]
#[prop_refs(IntransitiveActivity)]
pub struct Arrive {
    #[serde(rename = "type")]
    kind: ArriveType,

    #[serde(flatten)]
    #[prop_refs]
    pub arrive_props: ArriveProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub ap_object_props: ApObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub activity_props: ActivityProperties,
}

/// Indicates that the actor is blocking the object.
///
/// Blocking is a stronger form of Ignore. The typical use is to support social systems that allow
/// one user to block activities or content of other users. The target and origin typically have no
/// defined meaning.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
#[prop_refs(Object)]
#[prop_refs(Activity)]
pub struct Block {
    #[serde(rename = "type")]
    kind: BlockType,

    #[serde(flatten)]
    #[prop_refs]
    pub block_props: BlockProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub ap_object_props: ApObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub activity_props: ActivityProperties,
}

/// Indicates that the actor has created the object.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
#[prop_refs(Object)]
#[prop_refs(Activity)]
pub struct Create {
    #[serde(rename = "type")]
    kind: CreateType,

    #[serde(flatten)]
    #[prop_refs]
    pub create_props: CreateProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub ap_object_props: ApObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub activity_props: ActivityProperties,
}

/// Indicates that the actor has deleted the object.
///
/// If specified, the origin indicates the context from which the object was deleted.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
#[prop_refs(Object)]
#[prop_refs(Activity)]
pub struct Delete {
    #[serde(rename = "type")]
    kind: DeleteType,

    #[serde(flatten)]
    #[prop_refs]
    pub delete_props: DeleteProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub ap_object_props: ApObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub activity_props: ActivityProperties,
}

/// Indicates that the actor dislikes the object.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
#[prop_refs(Object)]
#[prop_refs(Activity)]
pub struct Dislike {
    #[serde(rename = "type")]
    kind: DislikeType,

    #[serde(flatten)]
    #[prop_refs]
    pub dislike_props: DislikeProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub ap_object_props: ApObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub activity_props: ActivityProperties,
}

/// Indicates that the actor is "flagging" the object.
///
/// Flagging is defined in the sense common to many social platforms as reporting content as being
/// inappropriate for any number of reasons.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
#[prop_refs(Object)]
#[prop_refs(Activity)]
pub struct Flag {
    #[serde(rename = "type")]
    kind: FlagType,

    #[serde(flatten)]
    #[prop_refs]
    pub flag_props: FlagProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub ap_object_props: ApObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub activity_props: ActivityProperties,
}

/// Indicates that the actor is "following" the object.
///
/// Following is defined in the sense typically used within Social systems in which the actor is
/// interested in any activity performed by or on the object. The target and origin typically have
/// no defined meaning.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
#[prop_refs(Object)]
#[prop_refs(Activity)]
pub struct Follow {
    #[serde(rename = "type")]
    kind: FollowType,

    #[serde(flatten)]
    #[prop_refs]
    pub follow_props: FollowProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub ap_object_props: ApObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub activity_props: ActivityProperties,
}

/// Indicates that the actor is ignoring the object.
///
/// The target and origin typically have no defined meaning.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
#[prop_refs(Object)]
#[prop_refs(Activity)]
pub struct Ignore {
    #[serde(rename = "type")]
    kind: IgnoreType,

    #[serde(flatten)]
    #[prop_refs]
    pub ignore_props: IgnoreProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub ap_object_props: ApObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub activity_props: ActivityProperties,
}

/// A specialization of Offer in which the actor is extending an invitation for the object to the
/// target.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
#[prop_refs(Object)]
#[prop_refs(Activity)]
pub struct Invite {
    #[serde(rename = "type")]
    kind: InviteType,

    #[serde(flatten)]
    #[prop_refs]
    pub invite_props: InviteProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub ap_object_props: ApObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub activity_props: ActivityProperties,
}

/// Indicates that the actor has joined the object.
///
/// The target and origin typically have no defined meaning
#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
#[prop_refs(Object)]
#[prop_refs(Activity)]
pub struct Join {
    #[serde(rename = "type")]
    kind: JoinType,

    #[serde(flatten)]
    #[prop_refs]
    pub join_props: JoinProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub ap_object_props: ApObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub activity_props: ActivityProperties,
}

/// Indicates that the actor has left the object.
///
/// The target and origin typically have no meaning.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
#[prop_refs(Object)]
#[prop_refs(Activity)]
pub struct Leave {
    #[serde(rename = "type")]
    kind: LeaveType,

    #[serde(flatten)]
    #[prop_refs]
    pub leave_props: LeaveProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub ap_object_props: ApObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub activity_props: ActivityProperties,
}

/// Indicates that the actor likes, recommends or endorses the object.
///
/// The target and origin typically have no defined meaning.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
#[prop_refs(Object)]
#[prop_refs(Activity)]
pub struct Like {
    #[serde(rename = "type")]
    kind: LikeType,

    #[serde(flatten)]
    #[prop_refs]
    pub like_props: LikeProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub ap_object_props: ApObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub activity_props: ActivityProperties,
}

/// Indicates that the actor has listened to the object.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
#[prop_refs(Object)]
#[prop_refs(Activity)]
pub struct Listen {
    #[serde(rename = "type")]
    kind: ListenType,

    #[serde(flatten)]
    #[prop_refs]
    pub listen_props: ListenProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub ap_object_props: ApObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub activity_props: ActivityProperties,
}

/// Indicates that the actor is offering the object.
///
/// If specified, the target indicates the entity to which the object is being offered.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
#[prop_refs(Object)]
#[prop_refs(Activity)]
pub struct Offer {
    #[serde(rename = "type")]
    kind: OfferType,

    #[serde(flatten)]
    #[prop_refs]
    pub offer_props: OfferProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub ap_object_props: ApObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub activity_props: ActivityProperties,
}

/// Represents a question being asked.
///
/// Question objects are an extension of IntransitiveActivity. That is, the Question object is an
/// Activity, but the direct object is the question itself and therefore it would not contain an
/// object property.
///
/// Either of the anyOf and oneOf properties MAY be used to express possible answers, but a
/// Question object MUST NOT have both properties.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
#[prop_refs(Object)]
#[prop_refs(Activity)]
#[prop_refs(IntransitiveActivity)]
pub struct Question {
    #[serde(rename = "type")]
    kind: QuestionType,

    #[serde(flatten)]
    #[prop_refs]
    pub question_props: QuestionProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub ap_object_props: ApObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub activity_props: ActivityProperties,
}

/// Indicates that the actor has read the object.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
#[prop_refs(Object)]
#[prop_refs(Activity)]
pub struct Read {
    #[serde(rename = "type")]
    kind: ReadType,

    #[serde(flatten)]
    #[prop_refs]
    pub read_props: ReadProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub ap_object_props: ApObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub activity_props: ActivityProperties,
}

/// Indicates that the actor is rejecting the object.
///
/// The target and origin typically have no defined meaning.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
#[prop_refs(Object)]
#[prop_refs(Activity)]
pub struct Reject {
    #[serde(rename = "type")]
    kind: RejectType,

    #[serde(flatten)]
    #[prop_refs]
    pub reject_props: RejectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub ap_object_props: ApObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub activity_props: ActivityProperties,
}

/// Indicates that the actor is removing the object.
///
/// If specified, the origin indicates the context from which the object is being removed.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
#[prop_refs(Object)]
#[prop_refs(Activity)]
pub struct Remove {
    #[serde(rename = "type")]
    kind: RemoveType,

    #[serde(flatten)]
    #[prop_refs]
    pub remove_props: RemoveProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub ap_object_props: ApObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub activity_props: ActivityProperties,
}

/// A specialization of Accept indicating that the acceptance is tentative.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
#[prop_refs(Object)]
#[prop_refs(Activity)]
pub struct TentativeAccept {
    #[serde(rename = "type")]
    kind: TentativeAcceptType,

    #[serde(flatten)]
    #[prop_refs]
    pub tentative_accept_props: TentativeAcceptProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub ap_object_props: ApObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub activity_props: ActivityProperties,
}

/// A specialization of Reject in which the rejection is considered tentative.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
#[prop_refs(Object)]
#[prop_refs(Activity)]
pub struct TentativeReject {
    #[serde(rename = "type")]
    kind: TentativeRejectType,

    #[serde(flatten)]
    #[prop_refs]
    pub tentative_reject_props: TentativeRejectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub ap_object_props: ApObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub activity_props: ActivityProperties,
}

/// Indicates that the actor is traveling to target from origin.
///
/// Travel is an IntransitiveObject whose actor specifies the direct object. If the target or
/// origin are not specified, either can be determined by context.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
#[prop_refs(Object)]
#[prop_refs(Activity)]
#[prop_refs(IntransitiveActivity)]
pub struct Travel {
    #[serde(rename = "type")]
    kind: TravelType,

    #[serde(flatten)]
    #[prop_refs]
    pub travel_props: TravelProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub ap_object_props: ApObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub activity_props: ActivityProperties,
}

/// Indicates that the actor is undoing the object.
///
/// In most cases, the object will be an Activity describing some previously performed action (for
/// instance, a person may have previously "liked" an article but, for whatever reason, might
/// choose to undo that like at some later point in time).
///
/// The target and origin typically have no defined meaning.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
#[prop_refs(Object)]
#[prop_refs(Activity)]
pub struct Undo {
    #[serde(rename = "type")]
    kind: UndoType,

    #[serde(flatten)]
    #[prop_refs]
    pub undo_props: UndoProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub ap_object_props: ApObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub activity_props: ActivityProperties,
}

/// Indicates that the actor has updated the object.
///
/// Note, however, that this vocabulary does not define a mechanism for describing the actual set
/// of modifications made to object.
///
/// The target and origin typically have no defined meaning.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
#[prop_refs(Object)]
#[prop_refs(Activity)]
pub struct Update {
    #[serde(rename = "type")]
    kind: UpdateType,

    #[serde(flatten)]
    #[prop_refs]
    pub update_props: UpdateProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub ap_object_props: ApObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub activity_props: ActivityProperties,
}

/// Indicates that the actor has viewed the object.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
#[prop_refs(Object)]
#[prop_refs(Activity)]
pub struct View {
    #[serde(rename = "type")]
    kind: ViewType,

    #[serde(flatten)]
    #[prop_refs]
    pub view_props: ViewProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub object_props: ObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub ap_object_props: ApObjectProperties,

    #[serde(flatten)]
    #[prop_refs]
    pub activity_props: ActivityProperties,
}
