use crate::utils::{error_chain_fmt, get_error_response, get_fail_response};
use actix_web::{http::StatusCode, web, HttpResponse, ResponseError};
use anyhow::Context;
use sqlx::{Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct Parameters {
    #[schema(example = "2Ig5l6jcH1aZP7Ipc30XHIMEq")]
    activation_token: String,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct SignupActivateResponse {
    #[schema(example = "success")]
    status: String,
    #[schema(example = "")]
    data: serde_json::Value,
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/signup/activate/{activation_token}",
    responses(
        (status = 200, description = "User activated", body = SignupActivateResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("activation_token", description = "The activation token sent to the user's email.")
    )
)]
#[tracing::instrument(name = "Activate a pending user", skip(parameters, pool))]
pub async fn signup_activate(
    parameters: web::Query<Parameters>,
    pool: web::Data<PgPool>,
) -> Result<web::Json<SignupActivateResponse>, SignupActivateError> {
    let user_id = get_user_id_from_token(&pool, &parameters.activation_token)
        .await
        .context("Failed to retrieve the user id associated with the provided token.")?
        .ok_or(SignupActivateError::UnknownToken)?;

    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;
    activate_user(&mut transaction, user_id)
        .await
        .context("Failed to update the user status to `active`.")?;

    delete_activation_token(&mut transaction, &parameters.activation_token)
        .await
        .context("Failed deleting activation token that was already used")?;
    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to activate account.")?;

    Ok(web::Json(SignupActivateResponse {
        status: "success".to_string(),
        data: serde_json::Value::Null,
    }))
}

#[tracing::instrument(name = "Mark user as active", skip(user_id, transaction))]
pub async fn activate_user(
    transaction: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    let query = sqlx::query!(
        r#"UPDATE users SET status = 'active' WHERE user_id = $1"#,
        user_id,
    );
    transaction.execute(query).await?;

    Ok(())
}

#[tracing::instrument(name = "Get user_id from token", skip(activation_token, pool))]
pub async fn get_user_id_from_token(
    pool: &PgPool,
    activation_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT user_id FROM activation_tokens WHERE activation_token = $1"#,
        activation_token,
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|r| r.user_id))
}

#[tracing::instrument(name = "Delete activation token", skip(activation_token, transaction))]
pub async fn delete_activation_token(
    transaction: &mut Transaction<'_, Postgres>,
    activation_token: &str,
) -> Result<(), sqlx::Error> {
    let query = sqlx::query!(
        r#"DELETE FROM activation_tokens WHERE activation_token = $1"#,
        activation_token,
    );
    transaction.execute(query).await?;

    Ok(())
}

#[derive(thiserror::Error)]
pub enum SignupActivateError {
    #[error("Something went wrong.")]
    UnexpectedError(#[from] anyhow::Error),
    #[error("There is no user associated with the provided token.")]
    UnknownToken,
}

impl std::fmt::Debug for SignupActivateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for SignupActivateError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::UnknownToken => StatusCode::UNAUTHORIZED,
            Self::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            Self::UnexpectedError(_) => {
                HttpResponse::build(self.status_code()).json(get_error_response(self.to_string()))
            }
            Self::UnknownToken => {
                HttpResponse::build(self.status_code()).json(get_fail_response(self.to_string()))
            }
        }
    }
}
