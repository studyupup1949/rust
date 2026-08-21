use std::marker::PhantomData;

use activitystreams_vocabulary::{impl_default, impl_display};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::db::{Iri, TableEntry, TableEntryList, Uuid};
use crate::{Error, Result, impl_sql_list_field, impl_sql_mailbox, util};

/// Represents the mailbox variants.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailboxType {
    Inbox,
    Outbox,
}

impl MailboxType {
    pub const INBOX: &str = "inbox";
    pub const OUTBOX: &str = "outbox";

    /// Creates a new [MailboxType]
    #[inline]
    pub const fn new() -> Self {
        Self::Inbox
    }

    /// Gets the [MailboxType] string representation.
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Inbox => Self::INBOX,
            Self::Outbox => Self::OUTBOX,
        }
    }
}

impl_default!(MailboxType);
impl_display!(MailboxType, str);

/// Marker trait for mailbox types.
///
/// Helpful for deduplication of database code.
pub trait MailboxDir {
    fn mailbox() -> MailboxType;
}

/// Marker type for an ActivityPub inbox.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InboxType;

/// Marker type for an ActivityPub outbox.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OutboxType;

impl MailboxDir for InboxType {
    fn mailbox() -> MailboxType {
        MailboxType::Inbox
    }
}

impl MailboxDir for OutboxType {
    fn mailbox() -> MailboxType {
        MailboxType::Outbox
    }
}

/// Represents an ActivityPub mailbox
///
/// - [inbox](https://www.w3.org/TR/activitypub/#inbox)
/// - [outbox](https://www.w3.org/TR/activitypub/#inbox)
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Mailbox<Dir: MailboxDir> {
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    uuid: Uuid,
    id: Iri,
    actor: TableEntry,
    activities: Vec<TableEntry>,
    #[sqlx(skip)]
    _phantom: PhantomData<Dir>,
}

impl<Dir: MailboxDir> Mailbox<Dir> {
    /// Creates a new [Mailbox].
    pub fn new() -> Self {
        Self {
            uuid: Uuid::nil(),
            id: Iri::new(),
            actor: TableEntry::new(),
            activities: Vec::new(),
            _phantom: PhantomData,
        }
    }

    /// Performs checks on record invariants.
    pub fn check_db(&self) -> Result<()> {
        if self.id.is_empty() {
            Err(Error::sql("mailbox: empty ID"))
        } else if self.actor.is_empty() {
            Err(Error::sql("mailbox: empty actor"))
        } else {
            Ok(())
        }
    }

    /// Gets the [Uuid].
    ///
    /// Represents the [Uuid] primary key of the table entry.
    pub const fn uuid(&self) -> Uuid {
        self.uuid
    }

    /// Sets the [Uuid].
    ///
    /// Represents the [Uuid] primary key of the table entry.
    pub fn set_uuid<I: Into<Uuid>>(&mut self, uuid: I) {
        self.uuid = uuid.into();
    }

    /// Builder function that sets the [Uuid].
    ///
    /// Represents the [Uuid] primary key of the table entry.
    pub fn with_uuid<I: Into<Uuid>>(self, uuid: I) -> Self {
        Self {
            uuid: uuid.into(),
            ..self
        }
    }

    /// Gets the `id`.
    ///
    /// Represents the IRI used to fetch the [Mailbox] record.
    pub const fn id(&self) -> &Iri {
        &self.id
    }

    /// Sets the `id`.
    ///
    /// Represents the IRI used to fetch the [Mailbox] record.
    pub fn set_id<I: Into<Iri>>(&mut self, id: I) {
        self.id = id.into();
    }

    /// Builder function that sets the `id`.
    ///
    /// Represents the IRI used to fetch the [Mailbox] record.
    pub fn with_id<I: Into<Iri>>(self, id: I) -> Self {
        Self {
            id: id.into(),
            ..self
        }
    }

    /// Gets the `actor`.
    ///
    /// References the actor receiving activities in this [Mailbox].
    pub const fn actor(&self) -> TableEntry {
        self.actor
    }

    /// Sets the `actor`.
    ///
    /// References the actor receiving activities in this [Mailbox].
    pub fn set_actor<I: Into<TableEntry>>(&mut self, actor: I) {
        self.actor = actor.into();
    }

    /// Builder function that sets the `actor`.
    ///
    /// References the actor receiving activities in this [Mailbox].
    pub fn with_actor<I: Into<TableEntry>>(self, actor: I) -> Self {
        Self {
            actor: actor.into(),
            ..self
        }
    }
}

impl<Dir: MailboxDir> Default for Mailbox<Dir> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Dir: MailboxDir> core::fmt::Display for Mailbox<Dir> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        serde_json::to_string(self)
            .map_err(|_| core::fmt::Error)
            .and_then(|s| write!(f, "{s}"))
    }
}

/// Convenience alias for an ActivityPub [Inbox](https://www.w3.org/TR/activitypub/#inbox).
pub type Inbox = Mailbox<InboxType>;

impl_sql_list_field! {
    Inbox {
        /// Represents the list of mailbox activities.
        activity, activities: { "activities" TableEntry },
    }
}

impl_sql_mailbox! {
    Inbox {
        id: { "id" Iri },
        actor: { "actor" TableEntry },
        activities: { "activities" TableEntryList },
    }
}

/// Convenience alias for an ActivityPub [Outbox](https://www.w3.org/TR/activitypub/#outbox).
pub type Outbox = Mailbox<OutboxType>;

impl_sql_list_field! {
    Outbox {
        /// Represents the list of mailbox activities.
        activity, activities: { "activities" TableEntry },
    }
}

impl_sql_mailbox! {
    Outbox {
        id: { "id" Iri },
        actor: { "actor" TableEntry },
        activities: { "activities" TableEntryList },
    }
}
