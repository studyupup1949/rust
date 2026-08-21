use std::sync::Arc;

use axum::extract::{Path, Request, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};

use activitystreams_vocabulary::{Collection, Items, MimeType};

use crate::app::oauth::{ReadScope, Scope, WriteScope};
use crate::app::{App, AppState};
use crate::db::{Factory, Outbox, TableEntry, Uuid};
use crate::{Activity, ActorType, Role};

impl App {
    /// Gets a [Factory] by UUID in response to an authorized request.
    pub async fn get_factory(
        State(state): State<Arc<AppState>>,
        Path(uuid): Path<String>,
        req: Request,
    ) -> Response {
        let Ok(uuid) = Uuid::try_from(uuid).map_err(|err| {
            log::error!("get_factory: invalid UUID: {err}");
        }) else {
            return StatusCode::BAD_REQUEST.into_response();
        };

        let entry = TableEntry::create(Factory::TABLE, uuid);

        if let Err(res) = state
            .clone()
            .check_authorization(
                req,
                entry,
                |sc| {
                    sc.iter()
                        .any(|s| matches!(s, Scope::Profile | Scope::Read(ReadScope::Read)))
                },
                Role::Visit,
            )
            .await
        {
            return res;
        }

        let Ok(db_factory) = Factory::get(&*state.db().await, &uuid)
            .await
            .map_err(|err| log::error!("get_factory: {err}"))
        else {
            log::error!("get_factory: missing record for ID: {uuid}");
            return StatusCode::NOT_FOUND.into_response();
        };

        let Ok(body) = db_factory
            .try_into_vocab(&*state.db().await)
            .await
            .map(|f| f.to_string())
            .map_err(|err| log::error!("get_factory: error converting to vocabulary: {err}"))
        else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };

        Response::builder()
            .status(StatusCode::OK)
            .header(
                header::CONTENT_TYPE,
                MimeType::ApplicationActivityJson.as_str(),
            )
            .header(header::CONTENT_LENGTH, body.len())
            .body(body.into())
            .unwrap_or_else(|err| {
                log::error!("get_factory: error building response: {err}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            })
    }

    /// Retrieves a [Factory]'s [Outbox] [Activities](Activity).
    pub async fn factory_outbox_read(
        State(state): State<Arc<AppState>>,
        Path(uuid): Path<String>,
        req: Request,
    ) -> Response {
        let Ok(uuid) = Uuid::try_from(uuid).map_err(|err| {
            log::error!("factory: outbox: invalid UUID: {err}");
        }) else {
            return StatusCode::BAD_REQUEST.into_response();
        };

        let factory = TableEntry::create(Factory::TABLE, uuid);

        if let Err(res) = state
            .clone()
            .check_authorization(
                req,
                factory,
                |sc| {
                    sc.iter()
                        .any(|s| matches!(s, Scope::Profile | Scope::Read(ReadScope::Read)))
                },
                Role::Visit,
            )
            .await
        {
            return res;
        }

        let Ok(Some(outbox)) = Outbox::find_by_actor(&*state.db().await, &factory)
            .await
            .map_err(|err| log::error!("factory: outbox: {err}"))
        else {
            log::error!("factory: outbox: missing record for ID: {uuid}");
            return StatusCode::NOT_FOUND.into_response();
        };

        let Ok(activities) = state.outbox_activities(&outbox).await.map_err(|err| {
            log::error!("factory: outbox: error fetching activities: {err}");
        }) else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };

        let body = Collection::new()
            .with_total_items(activities.len() as u64)
            .with_items(Items::list(activities))
            .to_string();

        Response::builder()
            .status(StatusCode::OK)
            .header(
                header::CONTENT_TYPE,
                MimeType::ApplicationActivityJson.as_str(),
            )
            .header(header::CONTENT_LENGTH, body.len())
            .body(body.into())
            .unwrap_or_else(|err| {
                log::error!("factory: outbox: error building response: {err}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            })
    }

    /// Processes an [Activity] `POST`ed to a [Factory]'s [Outbox].
    ///
    /// Extracts the activity and actor information from the request for further handling.
    pub async fn factory_outbox_write(
        State(state): State<Arc<AppState>>,
        Path(uuid): Path<String>,
        req: Request,
    ) -> Response {
        let Ok(uuid) = Uuid::try_from(uuid).map_err(|err| {
            log::error!("factory: outbox: invalid UUID: {err}");
        }) else {
            return StatusCode::BAD_REQUEST.into_response();
        };

        let factory = TableEntry::create(Factory::TABLE, uuid);

        let (req, actor) = match state
            .clone()
            .check_authorization(
                req,
                factory,
                |sc| {
                    sc.iter()
                        .any(|s| matches!(s, Scope::Write(WriteScope::Write)))
                },
                // do not allow servers to write to the outbox
                Role::Deny,
            )
            .await
        {
            Ok(r) => r,
            Err(res) => return res,
        };

        let Ok(Some(mut outbox)) = Outbox::find_by_actor(&*state.db().await, &factory)
            .await
            .map_err(|err| log::error!("factory: outbox: {err}"))
        else {
            log::error!("factory: outbox: missing record for ID: {uuid}");
            return StatusCode::NOT_FOUND.into_response();
        };

        let Ok(body) = axum::body::to_bytes(req.into_body(), state.max_request_length())
            .await
            .map_err(|err| {
                log::error!("factory: outbox: error parsing body: {err}");
            })
        else {
            return StatusCode::BAD_REQUEST.into_response();
        };

        let Ok(body_str) = str::from_utf8(body.as_ref()).map_err(|err| {
            log::error!("factory: outbox: invalid UTF-8 body: {err}");
        }) else {
            return StatusCode::BAD_REQUEST.into_response();
        };

        let Ok(activity) = serde_json::from_str::<Activity>(body_str).map_err(|err| {
            log::error!("factory: outbox: error parsing activity: {err}");
        }) else {
            return StatusCode::BAD_REQUEST.into_response();
        };

        let Some(activity_actor) = activity.actor() else {
            log::error!("factory: outbox: missing activity actor");
            return StatusCode::BAD_REQUEST.into_response();
        };

        if let Err(err) =
            Self::check_activity_actor_id("factory: outbox", actor.id(), activity_actor)
        {
            log::error!("factory: outbox: invalid activity actor: {err}");
            return StatusCode::BAD_REQUEST.into_response();
        }

        let Ok(factory) = Factory::get(&*state.db().await, &uuid)
            .await
            .map_err(|err| {
                log::error!("factory: outbox: unable to fetch Factory: {err}");
            })
        else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };

        let available_actor_types = factory
            .available_actor_types()
            .iter()
            .filter_map(|a| ActorType::try_from(a).ok())
            .collect::<Vec<_>>();

        state
            .handle_factory_outbox_activity(
                &mut outbox,
                &actor,
                &activity,
                available_actor_types.as_slice(),
            )
            .await
            .unwrap_or_else(|err| {
                log::error!("factory: outbox: {err}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            })
    }
}
