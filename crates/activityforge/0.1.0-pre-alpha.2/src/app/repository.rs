use std::sync::Arc;

use axum::extract::{Path, Request, State};
use axum::response::{IntoResponse, Response};

use http::{StatusCode, header};

use activitystreams_vocabulary::{Collection, Items, MimeType};

use crate::app::oauth::{ReadScope, Scope, WriteScope};
use crate::app::{App, AppState};
use crate::db::{Inbox, Outbox, Repository, TableEntry, Uuid};
use crate::{Activity, Role};

impl App {
    /// Gets a [Repository] by UUID in response to an authorized request.
    pub async fn get_repository(
        State(state): State<Arc<AppState>>,
        Path(uuid): Path<String>,
        req: Request,
    ) -> Response {
        let Ok(uuid) = Uuid::try_from(uuid).map_err(|err| {
            log::error!("get_repository: invalid UUID: {err}");
        }) else {
            return StatusCode::BAD_REQUEST.into_response();
        };

        let Ok(db_repository) = Repository::get(&*state.db().await, &uuid)
            .await
            .map_err(|err| log::error!("get_repository: {err}"))
        else {
            log::error!("get_repository: missing record for ID: {uuid}");
            return StatusCode::NOT_FOUND.into_response();
        };

        if db_repository.is_private()
            && let Err(res) = state
                .clone()
                .check_authorization(
                    req,
                    db_repository.table_entry(),
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

        let Ok(body) = db_repository
            .try_into_vocab(&*state.db().await)
            .await
            .map(|r| r.to_string())
            .map_err(|err| log::error!("get_repository: error converting to vocabulary: {err}"))
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
                log::error!("get_repository: error building response: {err}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            })
    }

    /// Retrieves a [Repository]'s [Inbox] [Activities](Activity).
    pub async fn repository_inbox_read(
        State(state): State<Arc<AppState>>,
        Path(uuid): Path<String>,
        req: Request,
    ) -> Response {
        let Ok(uuid) = Uuid::try_from(uuid).map_err(|err| {
            log::error!("repository: inbox: invalid UUID: {err}");
        }) else {
            return StatusCode::BAD_REQUEST.into_response();
        };

        let repository = TableEntry::create(Repository::TABLE, uuid);

        let Ok(db_repository) = Repository::get(&*state.db().await, &uuid)
            .await
            .map_err(|err| log::error!("get_repository: {err}"))
        else {
            log::error!("get_repository: missing record for ID: {uuid}");
            return StatusCode::NOT_FOUND.into_response();
        };

        if db_repository.is_private()
            && let Err(res) = state
                .clone()
                .check_authorization(
                    req,
                    repository,
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

        let Ok(Some(inbox)) = Inbox::find_by_actor(&*state.db().await, &repository)
            .await
            .map_err(|err| log::error!("repository: inbox: {err}"))
        else {
            log::error!("repository: inbox: missing record for ID: {uuid}");
            return StatusCode::NOT_FOUND.into_response();
        };

        let Ok(activities) = state.inbox_activities(&inbox).await.map_err(|err| {
            log::error!("repository: inbox: error fetching activities: {err}");
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
                log::error!("repository: inbox: error building response: {err}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            })
    }

    /// Processes an [Activity] `POST`ed to a [Repository]'s [Inbox].
    ///
    /// Extracts the activity and actor information from the request for further handling.
    pub async fn repository_inbox_write(
        State(state): State<Arc<AppState>>,
        Path(uuid): Path<String>,
        req: Request,
    ) -> Response {
        let Ok(uuid) = Uuid::try_from(uuid).map_err(|err| {
            log::error!("repository_inbox: invalid UUID: {err}");
        }) else {
            return StatusCode::BAD_REQUEST.into_response();
        };

        let repository = TableEntry::create(Repository::TABLE, uuid);

        let Ok(db_repository) = Repository::get(&*state.db().await, &uuid)
            .await
            .map_err(|err| log::error!("get_repository: {err}"))
        else {
            log::error!("get_repository: missing record for ID: {uuid}");
            return StatusCode::NOT_FOUND.into_response();
        };

        let role = if db_repository.is_private() {
            Role::Visit
        } else {
            Role::Public
        };

        let (req, actor) = match state
            .clone()
            .check_authorization(
                req,
                repository,
                // do not allow clients to write to the inbox
                |_| false,
                role,
            )
            .await
        {
            Ok(r) => r,
            Err(res) => return res,
        };

        let Ok(Some(mut inbox)) = Inbox::find_by_actor(&*state.db().await, &repository)
            .await
            .map_err(|err| log::error!("repository_inbox: {err}"))
        else {
            log::error!("repository_inbox: missing record for ID: {uuid}");
            return StatusCode::NOT_FOUND.into_response();
        };

        let Ok(body) = axum::body::to_bytes(req.into_body(), state.max_request_length())
            .await
            .map_err(|err| {
                log::error!("repository_inbox: error parsing body: {err}");
            })
        else {
            return StatusCode::BAD_REQUEST.into_response();
        };

        let Ok(body_str) = str::from_utf8(body.as_ref()).map_err(|err| {
            log::error!("repository_inbox: invalid UTF-8 body: {err}");
        }) else {
            return StatusCode::BAD_REQUEST.into_response();
        };

        let Ok(activity) = serde_json::from_str::<Activity>(body_str).map_err(|err| {
            log::error!("repository_inbox: error parsing activity: {err}");
        }) else {
            return StatusCode::BAD_REQUEST.into_response();
        };

        let Some(activity_actor) = activity.actor() else {
            log::error!("repository_inbox: missing activity actor");
            return StatusCode::BAD_REQUEST.into_response();
        };

        if let Err(err) =
            Self::check_activity_actor_id("repository_inbox", actor.id(), activity_actor)
        {
            log::error!("repository_inbox: invalid activity actor: {err}");
            return StatusCode::BAD_REQUEST.into_response();
        }

        state
            .handle_repository_inbox_activity(&mut inbox, &actor, &activity)
            .await
            .unwrap_or_else(|err| {
                log::error!("repository_inbox: {err}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            })
    }

    /// Retrieves a [Repository]'s [Outbox] [Activities](Activity).
    pub async fn repository_outbox_read(
        State(state): State<Arc<AppState>>,
        Path(uuid): Path<String>,
        req: Request,
    ) -> Response {
        let Ok(uuid) = Uuid::try_from(uuid).map_err(|err| {
            log::error!("repository_outbox: invalid UUID: {err}");
        }) else {
            return StatusCode::BAD_REQUEST.into_response();
        };

        let repository = TableEntry::create(Repository::TABLE, uuid);

        let Ok(db_repository) = Repository::get(&*state.db().await, &uuid)
            .await
            .map_err(|err| log::error!("get_repository: {err}"))
        else {
            log::error!("get_repository: missing record for ID: {uuid}");
            return StatusCode::NOT_FOUND.into_response();
        };

        if db_repository.is_private()
            && let Err(res) = state
                .clone()
                .check_authorization(
                    req,
                    repository,
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

        let Ok(Some(outbox)) = Outbox::find_by_actor(&*state.db().await, &repository)
            .await
            .map_err(|err| log::error!("repository_outbox: {err}"))
        else {
            log::error!("repository_outbox: missing record for ID: {uuid}");
            return StatusCode::NOT_FOUND.into_response();
        };

        let Ok(activities) = state.outbox_activities(&outbox).await.map_err(|err| {
            log::error!("repository_outbox: error fetching activities: {err}");
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
                log::error!("repository_outbox: error building response: {err}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            })
    }

    /// Processes an [Activity] `POST`ed to a [Repository]'s [Outbox].
    ///
    /// Extracts the activity and actor information from the request for further handling.
    pub async fn repository_outbox_write(
        State(state): State<Arc<AppState>>,
        Path(uuid): Path<String>,
        req: Request,
    ) -> Response {
        let Ok(uuid) = Uuid::try_from(uuid).map_err(|err| {
            log::error!("repository_outbox: invalid UUID: {err}");
        }) else {
            return StatusCode::BAD_REQUEST.into_response();
        };

        let repository = TableEntry::create(Repository::TABLE, uuid);

        let (req, actor) = match state
            .clone()
            .check_authorization(
                req,
                repository,
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

        let Ok(Some(mut outbox)) = Outbox::find_by_actor(&*state.db().await, &repository)
            .await
            .map_err(|err| log::error!("repository_outbox: {err}"))
        else {
            log::error!("repository_outbox: missing record for ID: {uuid}");
            return StatusCode::NOT_FOUND.into_response();
        };

        let Ok(body) = axum::body::to_bytes(req.into_body(), state.max_request_length())
            .await
            .map_err(|err| {
                log::error!("repository_outbox: error parsing body: {err}");
            })
        else {
            return StatusCode::BAD_REQUEST.into_response();
        };

        let Ok(body_str) = str::from_utf8(body.as_ref()).map_err(|err| {
            log::error!("repository_outbox: invalid UTF-8 body: {err}");
        }) else {
            return StatusCode::BAD_REQUEST.into_response();
        };

        let Ok(activity) = serde_json::from_str::<Activity>(body_str).map_err(|err| {
            log::error!("repository_outbox: error parsing activity: {err}");
        }) else {
            return StatusCode::BAD_REQUEST.into_response();
        };

        let Some(activity_actor) = activity.actor() else {
            log::error!("repository_outbox: missing activity actor");
            return StatusCode::BAD_REQUEST.into_response();
        };

        if let Err(err) =
            Self::check_activity_actor_id("repository_outbox", actor.id(), activity_actor)
        {
            log::error!("repository_outbox: invalid activity actor: {err}");
            return StatusCode::BAD_REQUEST.into_response();
        }

        state
            .handle_repository_outbox_activity(&mut outbox, &actor, &activity)
            .await
            .unwrap_or_else(|err| {
                log::error!("repository_outbox: {err}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            })
    }
}
