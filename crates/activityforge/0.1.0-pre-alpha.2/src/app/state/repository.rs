use activitystreams_vocabulary::{Accept, Iri, VocabularyType};
use axum::response::{IntoResponse, Response};

use http::{Method, StatusCode};

use crate::db::{
    Actor as DbActor, Grant as DbGrant, Inbox, Like as DbLike, Mailbox, MailboxDir, Outbox,
    Repository, Transaction,
};
use crate::{Activity, Error, Like, Result, Role};

use super::AppState;

impl AppState {
    /// Handles an [Activity] `POST`ed to a [Repository]'s [Outbox].
    pub async fn handle_repository_outbox_activity(
        &self,
        outbox: &mut Outbox,
        actor: &DbActor,
        activity: &Activity,
    ) -> Result<Response> {
        match activity {
            Activity::Like(like) => {
                let db = self.db().await;

                let pool = db
                    .pool()
                    .map_err(|err| Error::db(format!("repository: outbox: {err}")))?;

                let mut dbtx = pool
                    .begin()
                    .await
                    .map_err(|err| Error::db(format!("repository: outbox: {err}")))?;

                let db_like = self
                    .handle_repository_like_activity(&mut dbtx, outbox, actor, like)
                    .await?;

                outbox
                    .add_activity_tx(&mut dbtx, db_like.table_entry())
                    .await
                    .map_err(|err| Error::db(format!("repository: outbox: {err}")))?;

                dbtx.commit()
                    .await
                    .map(|_| StatusCode::OK.into_response())
                    .map_err(|err| Error::http(format!("repository: outbox: {err}")))
            }
            _ => Err(Error::http(
                "repository: outbox: unimplemented activity: {activity}",
            )),
        }
    }

    /// Handles an [Activity] `POST`ed to a [Repository]'s [Inbox].
    pub async fn handle_repository_inbox_activity(
        &self,
        inbox: &mut Inbox,
        actor: &DbActor,
        activity: &Activity,
    ) -> Result<Response> {
        match activity {
            Activity::Like(like) => {
                let db = self.db().await;

                let pool = db
                    .pool()
                    .map_err(|err| Error::db(format!("repository: inbox: {err}")))?;

                let mut dbtx = pool
                    .begin()
                    .await
                    .map_err(|err| Error::db(format!("repository: inbox: {err}")))?;

                let db_like = self
                    .handle_repository_like_activity(&mut dbtx, inbox, actor, like)
                    .await?;

                inbox
                    .add_activity_tx(&mut dbtx, db_like.table_entry())
                    .await
                    .map_err(|err| Error::db(format!("repository: inbox: {err}")))?;

                dbtx.commit()
                    .await
                    .map(|_| StatusCode::OK.into_response())
                    .map_err(|err| Error::http(format!("repository: inbox: {err}")))
            }
            _ => Err(Error::http(
                "repository: inbox: unimplemented activity: {activity}",
            )),
        }
    }

    /// Handles a [Like] activity submitted to a [Repository] outbox.
    pub async fn handle_repository_like_activity<T>(
        &self,
        dbtx: &mut Transaction<'_>,
        mailbox: &mut Mailbox<T>,
        actor: &DbActor,
        like: &Like,
    ) -> Result<DbLike>
    where
        T: MailboxDir,
    {
        let mailbox_ty = <T as MailboxDir>::mailbox();

        let actor_items = like.actor().ok_or(Error::http(format!(
            "repository: {mailbox_ty}: missing Like actor"
        )))?;

        let actor_id = actor_items
            .ids()
            .map(|i| i.first().copied())
            .map_err(|err| Error::http(format!("repository: {mailbox_ty}: {err}")))
            .and_then(|i| {
                i.ok_or(Error::http(format!(
                    "repository: {mailbox_ty}: missing Like actor ID"
                )))
            })?;

        if actor_id.as_str() != actor.id().as_str() {
            return Err(Error::http(format!(
                "repository: {mailbox_ty}: invalid Like actor ID: {actor_id}, expected: {}",
                actor.id()
            )));
        }

        let object_items = like.object().ok_or(Error::http(format!(
            "repository: {mailbox_ty}: missing Like object"
        )))?;

        let object_id = object_items
            .ids()
            .map(|i| i.first().copied())
            .map_err(|err| Error::http(format!("repository: {mailbox_ty}: {err}")))
            .and_then(|i| {
                i.ok_or(Error::http(format!(
                    "repository: {mailbox_ty}: missing Like object ID"
                )))
            })?;

        let object_actor = Repository::find_by_id_tx(dbtx, &object_id.into())
            .await
            .and_then(|a| {
                a.ok_or(Error::db(format!(
                    "repository: {mailbox_ty}: no actor exists for ID: {object_id}"
                )))
            })?;

        let mut repo_actor = Repository::get_tx(dbtx, &mailbox.actor().id()).await?;

        if object_actor != repo_actor {
            return Err(Error::http(format!(
                "repository: {mailbox_ty}: Like object does not match, have: {object_id}, expected: {}",
                repo_actor.id()
            )));
        }

        if repo_actor.is_private() || like.capability().is_some() {
            self.check_grants_tx(dbtx, actor, Role::Visit, repo_actor.table_entry())
                .await?;
        }

        let uuid = DbLike::TABLE.uuid_from_ids([&actor_id.into(), &object_id.into()]);
        let id = DbLike::TABLE.id_from_uuid(self.uri(), uuid)?;

        let mut db_like = DbLike::new()
            .with_uuid(uuid)
            .with_id(id)
            .with_actor(actor_id.clone())
            .with_object(object_id.clone());

        db_like.insert_tx(dbtx).await?;

        repo_actor.add_like_tx(dbtx, db_like.uuid()).await?;

        if let Some(cap) = like.capability() {
            let cap_id = cap.id()?;

            if let Err(err) = self
                .send_repository_like_accept(dbtx, like, actor_id, cap_id, &repo_actor)
                .await
            {
                log::warn!("repository: {mailbox_ty}: error sending Like Accept: {err}");
            }
        }

        Ok(db_like)
    }

    /// Sends an [Accept] activity to the [Like] actor's inbox.
    pub(crate) async fn send_repository_like_accept(
        &self,
        dbtx: &mut Transaction<'_>,
        like: &Like,
        actor_id: &Iri,
        grant_id: &Iri,
        repo_actor: &Repository,
    ) -> Result<()> {
        let grant = DbGrant::find_by_id_tx(dbtx, &grant_id.into())
            .await
            .and_then(|g| g.ok_or(Error::db("missing capability Grant")))?;

        if grant.context() != repo_actor.table_entry() {
            return Err(Error::http("capability context mismatch"));
        }

        let accept: Accept<VocabularyType> = Accept::new()
            .with_actor(Iri::from(repo_actor.id()))
            .with_object(like.clone().without_context_property());

        let db_actor = DbActor::find_by_id_tx(dbtx, &actor_id.into())
            .await
            .and_then(|a| a.ok_or(Error::db("missing Like actor")))?;

        let actor_inbox = Inbox::get_tx(dbtx, &db_actor.inbox()).await?;

        let res = self
            .signed_request(Method::POST, actor_inbox.id(), Some(&accept))
            .await?;

        if res.status() == StatusCode::OK {
            Ok(())
        } else {
            Err(Error::http(format!(
                "error sending Accept activity: {res:?}"
            )))
        }
    }
}
