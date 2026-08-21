use axum::response::Response;

use http::{Method, StatusCode};

use activitystreams_vocabulary::{Accept, Follow, Iri, Item, Items, Object};

use crate::crypto::{KeyType, SymmetricKey};
use crate::db::{
    Activity as DbActivity, Actor as DbActor, Create as DbCreate, Factory as DbFactory,
    Follow as DbFollow, Follower as DbFollower, Inbox, Mailbox, MailboxDir, Outbox,
    Person as DbPerson, TableEntry, Transaction,
};
use crate::{Activity, Actor, ActorType, Error, Factory, Result, Role, util};

use super::AppState;

impl AppState {
    /// Handles an [Activity] `POST`ed to a [Person](DbPerson)'s [Inbox].
    pub async fn handle_person_inbox_activity(
        &self,
        inbox: &mut Inbox,
        actor: &DbActor,
        activity: &Activity,
    ) -> Result<Response> {
        match activity {
            Activity::Follow(follow) => {
                let db = self.db().await;
                let pool = db.pool()?;
                let db_key = db.key()?;
                let mut dbtx = pool.begin().await?;

                let (db_follow, accept) = self
                    .handle_person_follow_activity(&mut dbtx, &db_key, actor, inbox, follow)
                    .await?;

                inbox
                    .add_activity_tx(&mut dbtx, db_follow.table_entry())
                    .await?;

                dbtx.commit()
                    .await
                    .map_err(|err| Error::db(format!("person: inbox: {err}")))?;

                Response::builder()
                    .status(StatusCode::OK)
                    .body(accept.to_string().into())
                    .map_err(|err| {
                        Error::http(format!("person: inbox: error building response: {err}"))
                    })
            }
            _ => Err(Error::http(format!(
                "person: inbox: unsupported activity: {activity}"
            ))),
        }
    }

    /// Handles an [Activity] `POST`ed to a [Person](DbPerson)'s [Outbox].
    pub async fn handle_person_outbox_activity(
        &self,
        outbox: &mut Outbox,
        actor: &DbActor,
        activity: &Activity,
    ) -> Result<Response> {
        match activity {
            Activity::Create(create) => {
                let create_actor = create
                    .actor()
                    .ok_or(Error::http("person: outbox: missing Create actor"))
                    .and_then(|i| Actor::from_items(actor.table(), i))?;

                let items = create
                    .object()
                    .ok_or(Error::http("person: outbox: missing activity object"))?;

                match items {
                    Items::Single(Item::Object(object)) if items.contains(ActorType::Factory) => {
                        let factory = self
                            .handle_create_factory_activity(outbox, actor, &create_actor, object)
                            .await?;

                        Response::builder()
                            .status(StatusCode::OK)
                            .body(factory.to_string().into())
                            .map_err(|err| {
                                Error::http(format!(
                                    "person: outbox: error building response: {err}"
                                ))
                            })
                    }
                    Items::List(list) if items.contains(ActorType::Factory) => {
                        let object = list
                            .iter()
                            .find(|i| i.contains(ActorType::Factory))
                            .ok_or(Error::http("person: outbox: missing Create object"))
                            .and_then(|i| i.as_object().map_err(Error::from))
                            .map_err(|err| {
                                Error::http(format!("person: outbox: invalid object: {err}"))
                            })?;

                        let factory = self
                            .handle_create_factory_activity(outbox, actor, &create_actor, object)
                            .await?;

                        Response::builder()
                            .status(StatusCode::OK)
                            .body(factory.to_string().into())
                            .map_err(|err| {
                                Error::http(format!(
                                    "person: outbox: error building response: {err}"
                                ))
                            })
                    }
                    _ => Err(Error::http("person: outbox: unsupported Activity type")),
                }
            }
            _ => Err(Error::http(format!(
                "person: outbox: unsupported Activity: {activity}"
            ))),
        }
    }

    /// Handles a [Create] activity to create a [Factory].
    ///
    /// This function **MUST** only be called:
    ///
    /// - after the request signature has been verified
    /// - the actor has been authorized
    #[allow(deprecated)]
    pub(crate) async fn handle_create_factory_activity(
        &self,
        outbox: &mut Outbox,
        actor: &DbActor,
        create_actor: &Actor,
        object: &Object,
    ) -> Result<Factory> {
        let create_actor_id = create_actor.id()?;
        let actor_id = actor.id();

        if actor_id.as_str() != create_actor_id.as_str() {
            return Err(Error::http(format!(
                "create_factory: mismatch of `Create` actor ID: {create_actor_id}, expected: {actor_id}"
            )));
        }

        let factory_uuid = util::rand_uuid();

        let mut factory = Factory::try_from(object)
            .map_err(|err| {
                Error::http(format!(
                    "create_factory: error parsing Factory object: {err}"
                ))
            })
            .and_then(|f| {
                DbFactory::TABLE
                    .id_from_uuid(self.uri(), factory_uuid)
                    .map(|id| f.with_id(id))
            })?;

        if factory.assertion_method().is_none() && factory.public_key().is_none() {
            log::debug!("create_factory: creating new factory keys");

            let (multikeys, pemkey) = self
                .create_keys(&factory.id().into(), [KeyType::Ed25519, KeyType::Rsa2048])
                .await
                .map_err(|err| {
                    Error::http(format!("create_factory: error creating key entries: {err}"))
                })?;

            if !multikeys.is_empty() {
                factory.set_assertion_method(multikeys);
            }

            if let Some(pemkey) = pemkey {
                factory.set_public_key(pemkey);
            }
        }

        let factory =
            DbFactory::try_from_vocab_with_uuid(&*self.db().await, &factory, factory_uuid)
                .await
                .map_err(|err| {
                    Error::http(format!(
                        "create_factory: error storing factory record: {err}"
                    ))
                })?;

        log::debug!("create_factory: successfully created factory: {factory}");

        let create_uuid = util::rand_uuid();
        let create_id = DbCreate::TABLE.id_from_uuid(self.uri(), create_uuid)?;

        let mut create_activity = DbActivity::create(
            DbCreate::new()
                .with_uuid(create_uuid)
                .with_id(create_id)
                .with_actor(actor.table_entry())
                .with_object(factory.table_entry()),
        );

        self.add_outbox_activity(outbox, &mut create_activity)
            .await?;

        let factory_actor = DbActor::factory(factory.clone());
        for grant_actor in [actor, &factory_actor].into_iter() {
            self.create_grant(
                grant_actor,
                &[Role::Visit, Role::Write, Role::Maintain, Role::Admin],
                factory.table_entry(),
            )
            .await
            .map_err(|err| Error::http(format!("create_factory: error creating grant: {err}")))?;

            self.create_grant(
                grant_actor,
                &[Role::Visit, Role::Write, Role::Maintain, Role::Admin],
                TableEntry::create(Inbox::TABLE, factory.inbox()),
            )
            .await
            .map_err(|err| {
                Error::http(format!("create_factory: error creating inbox grant: {err}"))
            })?;

            self.create_grant(
                grant_actor,
                &[Role::Visit, Role::Write, Role::Maintain, Role::Admin],
                TableEntry::create(Outbox::TABLE, factory.outbox()),
            )
            .await
            .map_err(|err| {
                Error::http(format!(
                    "create_factory: error creating outbox grant: {err}"
                ))
            })?;
        }

        Ok(Factory::new().with_id(factory.id()))
    }

    /// Handles a [Follow] activity for a [Person](DbPerson).
    pub(crate) async fn handle_person_follow_activity<T: MailboxDir>(
        &self,
        dbtx: &mut Transaction<'_>,
        db_key: &SymmetricKey,
        actor: &DbActor,
        mailbox: &Mailbox<T>,
        follow: &Follow,
    ) -> Result<(DbFollow, Accept)> {
        let mailbox_ty = <T as MailboxDir>::mailbox();
        let mut mailbox_actor = DbPerson::get_tx(dbtx, &mailbox.actor().id()).await?;

        let follow_actor = follow
            .actor()
            .ok_or(Error::http(format!(
                "person: {mailbox_ty}: missing Follow actor"
            )))
            .and_then(|i| Actor::from_items(DbPerson::TABLE, i))?;

        let object_actor = follow
            .object()
            .ok_or(Error::http(format!(
                "person: {mailbox_ty}: missing Follow object"
            )))
            .and_then(|i| Actor::from_items(DbPerson::TABLE, i))?;

        let object_id = object_actor
            .id()
            .map_err(|err| Error::http(format!("person: {mailbox_ty}: {err}")))?;

        let mailbox_actor_id = mailbox_actor.id();

        if object_id.as_str() != mailbox_actor_id.as_str() {
            return Err(Error::http(format!(
                "person: {mailbox_ty}: mismatch Follow object ID: {object_id}, expected: {mailbox_actor_id}"
            )));
        }

        let db_follow_actor = DbActor::try_from_vocab_tx(dbtx, db_key, &follow_actor).await?;
        let follow_actor_id = db_follow_actor.id();

        self.check_grants_tx(
            dbtx,
            &db_follow_actor,
            Role::Visit,
            mailbox_actor.table_entry(),
        )
        .await?;

        if actor.id() != db_follow_actor.id() {
            self.check_grants_tx(dbtx, actor, Role::Visit, mailbox_actor.table_entry())
                .await?;
        }

        let follow_uuid = DbFollow::TABLE.uuid_from_ids([follow_actor_id, mailbox_actor_id]);
        let follow_id = DbFollow::TABLE.id_from_uuid(self.uri(), follow_uuid)?;
        let mut db_follow = DbFollow::new()
            .with_uuid(follow_uuid)
            .with_id(follow_id)
            .with_actor(db_follow_actor.table_entry())
            .with_object(mailbox_actor.table_entry());

        db_follow.insert_tx(dbtx).await?;

        if let Some(mut follower) =
            DbFollower::find_by_actor_tx(dbtx, db_follow_actor.table_entry()).await?
        {
            follower
                .add_follow_tx(dbtx, mailbox_actor.table_entry())
                .await?;
        } else {
            let follower_uuid = DbFollower::TABLE.uuid_from_id(db_follow_actor.id());
            let follower_id = DbFollower::TABLE.id_from_uuid(self.uri(), follower_uuid)?;

            let mut db_follower = DbFollower::new()
                .with_uuid(follower_uuid)
                .with_id(follower_id)
                .with_actor(db_follow_actor.table_entry())
                .with_following([mailbox_actor.table_entry()])?;

            db_follower.insert_tx(dbtx).await?;
        }

        mailbox_actor
            .add_follower_tx(dbtx, db_follow_actor.uuid())
            .await?;

        // TODO: add support for manually approved Follow activities

        let accept: Accept = Accept::new()
            .with_actor(Iri::from(mailbox_actor.id()))
            .with_object(Iri::from(db_follow.id()));

        let actor_inbox = Inbox::get_tx(dbtx, &db_follow_actor.inbox()).await?;

        match self
            .signed_request_tx(dbtx, db_key, Method::POST, actor_inbox.id(), Some(&accept))
            .await
        {
            Ok(res) if res.status() != StatusCode::OK => {
                log::warn!("person: {mailbox_ty}: error sending Follow Accept: {res:?}")
            }
            Ok(_) => log::debug!(
                "person: {mailbox_ty}: successfully sent Accept to {}",
                actor_inbox.id()
            ),
            Err(err) => log::warn!("person: {mailbox_ty}: error sending Accept request: {err}"),
        }

        Ok((db_follow, accept))
    }
}
