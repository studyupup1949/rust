use crate::db::{Actor, Grant, TableEntry, TableType, Transaction, Uuid};
use crate::{Error, Result, Role, util};

use super::AppState;

impl AppState {
    /// Checks the database [Grant]s for an `actor` requesting `role` access to a `context` resource.
    pub async fn check_grants(&self, actor: &Actor, role: Role, context: TableEntry) -> Result<()> {
        let db = self.db().await;
        let pool = db.pool()?;
        let mut dbtx = pool.begin().await?;

        self.check_grants_tx(&mut dbtx, actor, role, context)
            .await?;

        dbtx.commit()
            .await
            .map(|_| ())
            .map_err(|err| Error::db(format!("app: {err}")))
    }

    /// Checks the database [Grant]s for an `actor` requesting `role` access to a `context` resource using a transaction.
    pub async fn check_grants_tx(
        &self,
        dbtx: &mut Transaction<'_>,
        actor: &Actor,
        role: Role,
        context: TableEntry,
    ) -> Result<()> {
        if role == Role::Public {
            return Ok(());
        } else if role == Role::Deny {
            return Err(Error::http(format!(
                "check_grants: access to {context} is denied"
            )));
        }

        let actor_entry = actor.table_entry();

        let grants = Grant::find_by_actor_tx(dbtx, &actor_entry)
            .await
            .map_err(|err| {
                Error::http(format!(
                    "check_grants: error retrieving grants for actor: {actor_entry}, error: {err}",
                ))
            })?;

        if grants.iter().any(|grant| {
            log::debug!("check_grants: checking grant: {grant}");
            grant.permits(actor.table_entry(), role, context)
        }) {
            Ok(())
        } else {
            Err(Error::http(format!(
                "check_grants: no `{role}` grant for {actor_entry} to {context}",
            )))
        }
    }

    /// Creates a new [Grant] record in the database.
    ///
    /// Grants an `actor` record `role` access to a `context` resource.
    pub async fn create_grant(
        &self,
        actor: &Actor,
        roles: &[Role],
        context: TableEntry,
    ) -> Result<Grant> {
        let db = self.db().await;
        let pool = db.pool()?;
        let mut dbtx = pool.begin().await?;

        let grant = self
            .create_grant_tx(&mut dbtx, actor, roles, context)
            .await?;

        dbtx.commit()
            .await
            .map(|_| grant)
            .map_err(|err| Error::db(format!("app: {err}")))
    }

    /// Creates a new [Grant] record in the database using a transaction.
    ///
    /// Grants an `actor` record `role` access to a `context` resource.
    pub async fn create_grant_tx(
        &self,
        dbtx: &mut Transaction<'_>,
        actor: &Actor,
        roles: &[Role],
        context: TableEntry,
    ) -> Result<Grant> {
        self.create_grant_with_uuid_tx(dbtx, util::rand_uuid(), actor, roles, context)
            .await
    }

    /// Creates a new [Grant] record in the database using a transaction.
    ///
    /// Grants an `actor` record `role` access to a `context` resource.
    pub async fn create_grant_with_uuid(
        &self,
        uuid: Uuid,
        actor: &Actor,
        roles: &[Role],
        context: TableEntry,
    ) -> Result<Grant> {
        let db = self.db().await;
        let pool = db.pool()?;
        let mut dbtx = pool.begin().await?;

        let grant = self
            .create_grant_with_uuid_tx(&mut dbtx, uuid, actor, roles, context)
            .await?;

        dbtx.commit()
            .await
            .map(|_| grant)
            .map_err(|err| Error::db(format!("app: {err}")))
    }

    /// Creates a new [Grant] record in the database using a transaction.
    ///
    /// Grants an `actor` record `role` access to a `context` resource.
    pub async fn create_grant_with_uuid_tx(
        &self,
        dbtx: &mut Transaction<'_>,
        uuid: Uuid,
        actor: &Actor,
        roles: &[Role],
        context: TableEntry,
    ) -> Result<Grant> {
        let id = TableType::Grant.id_from_uuid(self.uri(), uuid)?;

        let mut roles = roles.to_vec();
        roles.sort();
        roles.dedup();

        let mut grant = Grant::new()
            .with_uuid(uuid)
            .with_id(id)
            .with_actor(actor.table_entry())
            .with_context(context)
            .with_objects(roles)?;

        // add a Grant giving admin permissions to the person actor over the application.
        grant.find_or_create_tx(dbtx).await.map(|_| grant)
    }
}
