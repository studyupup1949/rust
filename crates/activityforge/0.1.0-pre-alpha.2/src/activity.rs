use activitystreams_vocabulary::{
    Accept, Add, Announce, Create, Delete, Dislike, Flag, Follow, Ignore, Invite, Item, Items,
    Join, Leave, Listen, Move, Offer, Read, Reject, Remove, TentativeAccept, TentativeReject, Undo,
    Update, View, create_item,
};

use crate::{Error, Result};

mod apply;
mod assign;
mod edit;
mod grant;
mod like;
mod push;
mod resolve;
mod revoke;

pub use apply::Apply;
pub use assign::Assign;
pub use edit::Edit;
pub use grant::{Grant, GrantItem};
pub use like::Like;
pub use push::Push;
pub use resolve::Resolve;
pub use revoke::Revoke;

create_item! {
    /// Represents a activity of any kind.
    ///
    /// An [Activity] is a subtype of [Object](activitystreams_vocabulary::Object) that describes some form of action that may happen, is currently happening, or has already happened.
    ///
    /// It is important to note that the [Activity] type itself does not carry any specific semantics about the kind of action being taken.
    Activity
        boxed
        default: Self::Apply(Box::default()),
    {
        Accept(Accept),
        TentativeAccept(TentativeAccept),
        Add(Add),
        Announce(Announce),
        Create(Create),
        Delete(Delete),
        Dislike(Dislike),
        Flag(Flag),
        Follow(Follow),
        Ignore(Ignore),
        Invite(Invite),
        Join(Join),
        Leave(Leave),
        Like(Like),
        Listen(Listen),
        Moving(Move),
        Offer(Offer),
        Read(Read),
        Reject(Reject),
        TentativeReject(TentativeReject),
        Remove(Remove),
        Undo(Undo),
        Update(Update),
        View(View),
        Apply(Apply),
        Assign(Assign),
        Edit(Edit),
        Grant(Grant),
        Push(Push),
        Resolve(Resolve),
        Revoke(Revoke),
    }
}

impl Activity {
    /// Gets an optional reference to the [Activity] actor.
    pub fn actor(&self) -> Option<&Items> {
        match self {
            Self::Accept(activity) => activity.actor(),
            Self::TentativeAccept(activity) => activity.actor(),
            Self::Add(activity) => activity.actor(),
            Self::Announce(activity) => activity.actor(),
            Self::Create(activity) => activity.actor(),
            Self::Delete(activity) => activity.actor(),
            Self::Dislike(activity) => activity.actor(),
            Self::Flag(activity) => activity.actor(),
            Self::Follow(activity) => activity.actor(),
            Self::Ignore(activity) => activity.actor(),
            Self::Invite(activity) => activity.actor(),
            Self::Join(activity) => activity.actor(),
            Self::Leave(activity) => activity.actor(),
            Self::Like(activity) => activity.actor(),
            Self::Listen(activity) => activity.actor(),
            Self::Moving(activity) => activity.actor(),
            Self::Offer(activity) => activity.actor(),
            Self::Read(activity) => activity.actor(),
            Self::Reject(activity) => activity.actor(),
            Self::TentativeReject(activity) => activity.actor(),
            Self::Remove(activity) => activity.actor(),
            Self::Undo(activity) => activity.actor(),
            Self::Update(activity) => activity.actor(),
            Self::View(activity) => activity.actor(),
            Self::Apply(activity) => activity.actor(),
            Self::Assign(activity) => activity.actor(),
            Self::Edit(activity) => activity.actor(),
            Self::Grant(activity) => activity.actor(),
            Self::Push(activity) => activity.actor(),
            Self::Resolve(activity) => activity.actor(),
            Self::Revoke(activity) => activity.actor(),
        }
    }
}

impl From<&Activity> for Item {
    fn from(val: &Activity) -> Self {
        match val {
            Activity::Accept(activity) => activity.as_ref().clone().into(),
            Activity::TentativeAccept(activity) => activity.as_ref().clone().into(),
            Activity::Add(activity) => activity.as_ref().clone().into(),
            Activity::Announce(activity) => activity.as_ref().clone().into(),
            Activity::Create(activity) => activity.as_ref().clone().into(),
            Activity::Delete(activity) => activity.as_ref().clone().into(),
            Activity::Dislike(activity) => activity.as_ref().clone().into(),
            Activity::Flag(activity) => activity.as_ref().clone().into(),
            Activity::Follow(activity) => activity.as_ref().clone().into(),
            Activity::Ignore(activity) => activity.as_ref().clone().into(),
            Activity::Invite(activity) => activity.as_ref().clone().into(),
            Activity::Join(activity) => activity.as_ref().clone().into(),
            Activity::Leave(activity) => activity.as_ref().clone().into(),
            Activity::Like(activity) => activity.as_ref().clone().into(),
            Activity::Listen(activity) => activity.as_ref().clone().into(),
            Activity::Moving(activity) => activity.as_ref().clone().into(),
            Activity::Offer(activity) => activity.as_ref().clone().into(),
            Activity::Read(activity) => activity.as_ref().clone().into(),
            Activity::Reject(activity) => activity.as_ref().clone().into(),
            Activity::TentativeReject(activity) => activity.as_ref().clone().into(),
            Activity::Remove(activity) => activity.as_ref().clone().into(),
            Activity::Undo(activity) => activity.as_ref().clone().into(),
            Activity::Update(activity) => activity.as_ref().clone().into(),
            Activity::View(activity) => activity.as_ref().clone().into(),
            Activity::Apply(activity) => activity.as_ref().clone().into(),
            Activity::Assign(activity) => activity.as_ref().clone().into(),
            Activity::Edit(activity) => activity.as_ref().clone().into(),
            Activity::Grant(activity) => activity.as_ref().clone().into(),
            Activity::Push(activity) => activity.as_ref().clone().into(),
            Activity::Resolve(activity) => activity.as_ref().clone().into(),
            Activity::Revoke(activity) => activity.as_ref().clone().into(),
        }
    }
}

impl From<Activity> for Item {
    fn from(val: Activity) -> Self {
        match val {
            Activity::Accept(activity) => (*activity).into(),
            Activity::TentativeAccept(activity) => (*activity).into(),
            Activity::Add(activity) => (*activity).into(),
            Activity::Announce(activity) => (*activity).into(),
            Activity::Create(activity) => (*activity).into(),
            Activity::Delete(activity) => (*activity).into(),
            Activity::Dislike(activity) => (*activity).into(),
            Activity::Flag(activity) => (*activity).into(),
            Activity::Follow(activity) => (*activity).into(),
            Activity::Ignore(activity) => (*activity).into(),
            Activity::Invite(activity) => (*activity).into(),
            Activity::Join(activity) => (*activity).into(),
            Activity::Leave(activity) => (*activity).into(),
            Activity::Like(activity) => (*activity).into(),
            Activity::Listen(activity) => (*activity).into(),
            Activity::Moving(activity) => (*activity).into(),
            Activity::Offer(activity) => (*activity).into(),
            Activity::Read(activity) => (*activity).into(),
            Activity::Reject(activity) => (*activity).into(),
            Activity::TentativeReject(activity) => (*activity).into(),
            Activity::Remove(activity) => (*activity).into(),
            Activity::Undo(activity) => (*activity).into(),
            Activity::Update(activity) => (*activity).into(),
            Activity::View(activity) => (*activity).into(),
            Activity::Apply(activity) => (*activity).into(),
            Activity::Assign(activity) => (*activity).into(),
            Activity::Edit(activity) => (*activity).into(),
            Activity::Grant(activity) => (*activity).into(),
            Activity::Push(activity) => (*activity).into(),
            Activity::Resolve(activity) => (*activity).into(),
            Activity::Revoke(activity) => (*activity).into(),
        }
    }
}

impl TryFrom<Item> for Activity {
    type Error = Error;

    fn try_from(val: Item) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&Item> for Activity {
    type Error = Error;

    fn try_from(val: &Item) -> Result<Self> {
        Accept::try_from(val)
            .map(Self::accept)
            .or_else(|_| TentativeAccept::try_from(val).map(Self::tentativeaccept))
            .or_else(|_| Add::try_from(val).map(Self::add))
            .or_else(|_| Announce::try_from(val).map(Self::announce))
            .or_else(|_| Create::try_from(val).map(Self::create))
            .or_else(|_| Delete::try_from(val).map(Self::delete))
            .or_else(|_| Dislike::try_from(val).map(Self::dislike))
            .or_else(|_| Flag::try_from(val).map(Self::flag))
            .or_else(|_| Follow::try_from(val).map(Self::follow))
            .or_else(|_| Ignore::try_from(val).map(Self::ignore))
            .or_else(|_| Invite::try_from(val).map(Self::invite))
            .or_else(|_| Join::try_from(val).map(Self::join))
            .or_else(|_| Leave::try_from(val).map(Self::leave))
            .or_else(|_| Like::try_from(val).map(Self::like))
            .or_else(|_| Listen::try_from(val).map(Self::listen))
            .or_else(|_| Move::try_from(val).map(Self::moving))
            .or_else(|_| Offer::try_from(val).map(Self::offer))
            .or_else(|_| Read::try_from(val).map(Self::read))
            .or_else(|_| Reject::try_from(val).map(Self::reject))
            .or_else(|_| TentativeReject::try_from(val).map(Self::tentativereject))
            .or_else(|_| Remove::try_from(val).map(Self::remove))
            .or_else(|_| Undo::try_from(val).map(Self::undo))
            .or_else(|_| Update::try_from(val).map(Self::update))
            .or_else(|_| View::try_from(val).map(Self::view))
            .or_else(|_| Apply::try_from(val).map(Self::apply))
            .or_else(|_| Assign::try_from(val).map(Self::assign))
            .or_else(|_| Edit::try_from(val).map(Self::edit))
            .or_else(|_| Grant::try_from(val).map(Self::grant))
            .or_else(|_| Push::try_from(val).map(Self::push))
            .or_else(|_| Resolve::try_from(val).map(Self::resolve))
            .or_else(|_| Revoke::try_from(val).map(Self::revoke))
            .map_err(|err| Error::activity(err.to_string()))
    }
}

impl TryFrom<Items> for Activity {
    type Error = Error;

    fn try_from(val: Items) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&Items> for Activity {
    type Error = Error;

    fn try_from(val: &Items) -> Result<Self> {
        match val {
            Items::Single(item) => item.try_into(),
            Items::List(list) => list
                .iter()
                .filter_map(|i| Self::try_from(i).ok())
                .next()
                .ok_or(Error::activity(format!("no valid item found: {val}"))),
        }
    }
}
