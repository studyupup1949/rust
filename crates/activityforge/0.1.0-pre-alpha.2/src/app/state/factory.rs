use axum::response::Response;

use http::StatusCode;

use activitystreams_vocabulary::{Item, Items, Object};

use crate::crypto::KeyType;
use crate::db::{
    Activity as DbActivity, Actor as DbActor, Create as DbCreate, Inbox, Outbox,
    Repository as DbRepository, TableEntry,
};
use crate::{Activity, Actor, ActorType, Error, Repository, Result, Role, util};

use super::AppState;

impl AppState {
    /// Handles an [Activity] `POST`ed to a [Factory](crate::db::Factory)'s [Outbox].
    pub async fn handle_factory_outbox_activity(
        &self,
        outbox: &mut Outbox,
        actor: &DbActor,
        activity: &Activity,
        available_actor_types: &[ActorType],
    ) -> Result<Response> {
        match activity {
            Activity::Create(create) => {
                let create_actor = create
                    .actor()
                    .ok_or(Error::http("factory_outbox: missing Create actor"))
                    .and_then(|i| Actor::from_items(actor.table(), i))?;

                let items = create
                    .object()
                    .ok_or(Error::http("factory_outbox: missing activity object"))?;

                match items {
                    Items::Single(Item::Object(object))
                        if items.contains(ActorType::Repository) =>
                    {
                        let repo_type = ActorType::Repository;

                        if available_actor_types.contains(&repo_type) {
                            let repo = self
                                .handle_create_repository_activity(
                                    outbox,
                                    actor,
                                    &create_actor,
                                    object,
                                )
                                .await?;

                            Response::builder()
                                .status(StatusCode::OK)
                                .body(repo.to_string().into())
                                .map_err(|err| {
                                    Error::http(format!(
                                        "factory_outbox: error building response: {err}"
                                    ))
                                })
                        } else {
                            Err(Error::http(format!(
                                "factory_outbox: {repo_type} is unavailable"
                            )))
                        }
                    }
                    Items::List(list) if items.contains(ActorType::Repository) => {
                        let repo_type = ActorType::Repository;

                        if available_actor_types.contains(&repo_type) {
                            let object = list
                                .iter()
                                .find(|i| i.contains(repo_type))
                                .ok_or(Error::http("factory_outbox: missing Create object"))
                                .and_then(|i| i.as_object().map_err(Error::from))
                                .map_err(|err| {
                                    Error::http(format!("factory_outbox: invalid object: {err}"))
                                })?;

                            let repo = self
                                .handle_create_factory_activity(
                                    outbox,
                                    actor,
                                    &create_actor,
                                    object,
                                )
                                .await?;

                            Response::builder()
                                .status(StatusCode::OK)
                                .body(repo.to_string().into())
                                .map_err(|err| {
                                    Error::http(format!(
                                        "factory_outbox: error building response: {err}"
                                    ))
                                })
                        } else {
                            Err(Error::http(format!(
                                "factory_outbox: {repo_type} is unavailable"
                            )))
                        }
                    }
                    _ => Err(Error::http("factory_outbox: unsupported Activity type")),
                }
            }
            _ => Err(Error::http(
                "factory_outbox: unimplemented activity: {activity}",
            )),
        }
    }

    /// Handles a [Create] activity to create a [Repository].
    ///
    /// This function **MUST** only be called:
    ///
    /// - after the request signature has been verified
    /// - the actor has been authorized
    #[allow(deprecated)]
    pub(crate) async fn handle_create_repository_activity(
        &self,
        outbox: &mut Outbox,
        actor: &DbActor,
        create_actor: &Actor,
        object: &Object,
    ) -> Result<Repository> {
        let create_actor_id = create_actor.id()?;
        let actor_id = actor.id();

        if actor_id.as_str() != create_actor_id.as_str() {
            return Err(Error::http(format!(
                "create_repository: mismatch of `Create` actor ID: {create_actor_id}, expected: {actor_id}"
            )));
        }

        let repository_uuid = util::rand_uuid();

        let mut repository = Repository::try_from(object)
            .map_err(|err| {
                Error::http(format!(
                    "create_repository: error parsing Repository object: {err}"
                ))
            })
            .and_then(|f| {
                DbRepository::TABLE
                    .id_from_uuid(self.uri(), repository_uuid)
                    .map(|id| f.with_id(id))
            })?;

        if repository.assertion_method().is_none() && repository.public_key().is_none() {
            log::debug!("create_repository: creating new repository keys");

            let (multikeys, pemkey) = self
                .create_keys(
                    &repository.id().into(),
                    [KeyType::Ed25519, KeyType::Rsa2048],
                )
                .await
                .map_err(|err| {
                    Error::http(format!(
                        "create_repository: error creating key entries: {err}"
                    ))
                })?;

            if !multikeys.is_empty() {
                repository.set_assertion_method(multikeys);
            }

            if let Some(pemkey) = pemkey {
                repository.set_public_key(pemkey);
            }
        }

        let repository =
            DbRepository::try_from_vocab_with_uuid(&*self.db().await, &repository, repository_uuid)
                .await
                .map_err(|err| {
                    Error::http(format!(
                        "create_repository: error storing repository record: {err}"
                    ))
                })?;

        log::debug!("create_repository: successfully created repository: {repository}");

        let create_uuid = util::rand_uuid();
        let create_id = DbCreate::TABLE.id_from_uuid(self.uri(), create_uuid)?;

        let mut create_activity = DbActivity::create(
            DbCreate::new()
                .with_uuid(create_uuid)
                .with_id(create_id)
                .with_actor(actor.table_entry())
                .with_object(repository.table_entry()),
        );

        self.add_outbox_activity(outbox, &mut create_activity)
            .await?;

        self.create_grant(
            actor,
            &[Role::Visit, Role::Write, Role::Maintain, Role::Admin],
            repository.table_entry(),
        )
        .await
        .map_err(|err| Error::http(format!("create_repository: error creating grant: {err}")))?;

        self.create_grant(
            actor,
            &[Role::Visit, Role::Write, Role::Maintain, Role::Admin],
            TableEntry::create(Inbox::TABLE, repository.inbox()),
        )
        .await
        .map_err(|err| {
            Error::http(format!(
                "create_repository: error creating inbox grant: {err}"
            ))
        })?;

        self.create_grant(
            actor,
            &[Role::Visit, Role::Write, Role::Maintain, Role::Admin],
            TableEntry::create(Outbox::TABLE, repository.outbox()),
        )
        .await
        .map_err(|err| {
            Error::http(format!(
                "create_repository: error creating outbox grant: {err}"
            ))
        })?;

        Ok(Repository::new().with_id(repository.id()))
    }
}
