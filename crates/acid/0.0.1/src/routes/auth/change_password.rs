use crate::domain::UserPassword;
use crate::utils::{error_chain_fmt, get_error_response, get_fail_response, make_password_hash};
use actix_web::{http::StatusCode, web, HttpResponse, ResponseError};
use anyhow::Context;
use sqlx::PgPool;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(serde::Deserialize, ToSchema)]
pub struct ChangePasswordRequest {
    #[schema(example = "2Ig5l6jcH1aZP7Ipc30XHIMEq")]
    reset_token: String,
    #[schema(example = "Password1234!", required = true)]
    password: String,
    #[schema(example = "Password1234!", required = true)]
    password_again: String,
}

#[derive(serde::Serialize, ToSchema)]
pub struct ChangePasswordResponse {
    #[schema(example = "success")]
    status: String,
    #[schema(example = "")]
    data: serde_json::Value,
}

#[utoipa::path(
    request_body = ChangePasswordRequest,
    post,
    path = "/api/v1/auth/change_password",
    responses(
        (status = 200, description = "Password changed", body = ChangePasswordResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
)]
#[tracing::instrument(name = "Change password", skip(request, pool))]
pub async fn change_password(
    request: web::Json<ChangePasswordRequest>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ChangePasswordError> {
    let user_id = get_user_id_from_reset_token(&pool, &request.0.reset_token)
        .await
        .context("Failed to retrieve the user id associated with the provided token.")?
        .ok_or(ChangePasswordError::UnknownToken)?;

    if request.0.password != request.0.password_again {
        return Err(ChangePasswordError::PasswordsWontMatch);
    }

    let password = UserPassword::parse(request.0.password.to_string())
        .map_err(ChangePasswordError::ValidationError)?;

    update_user_password(&pool, password, user_id)
        .await
        .map_err(ChangePasswordError::UnexpectedError)?;

    let response = web::Json(ChangePasswordResponse {
        status: "success".to_string(),
        data: serde_json::Value::Null,
    });

    Ok(HttpResponse::Ok().json(response))
}

#[tracing::instrument(
    name = "Saving new user details in the database",
    skip(password, user_id, pool)
)]
pub async fn update_user_password(
    pool: &PgPool,
    password: UserPassword,
    user_id: Uuid,
) -> Result<Uuid, anyhow::Error> {
    let password_hash = make_password_hash(password.as_ref());

    sqlx::query!(
        r#"UPDATE users SET password_hash = $1 where user_id = $2"#,
        password_hash,
        user_id,
    )
    .execute(pool)
    .await?;
    Ok(user_id)
}

#[tracing::instrument(
    name = "Get user_id from password reset token",
    skip(activation_token, pool)
)]
pub async fn get_user_id_from_reset_token(
    pool: &PgPool,
    activation_token: &str,
) -> Result<Option<Uuid>, anyhow::Error> {
    let result = sqlx::query!(
        r#"SELECT user_id FROM password_reset_tokens WHERE reset_token = $1"#,
        activation_token,
    )
    .fetch_optional(pool)
    .await?;
    Ok(result.map(|r| r.user_id))
}

#[derive(thiserror::Error)]
pub enum ChangePasswordError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error("You are not authorized to make this action")]
    PasswordsWontMatch,
    #[error("There is no user associated with the provided token.")]
    UnknownToken,
    #[error("{0}")]
    ValidationError(String),
}

impl std::fmt::Debug for ChangePasswordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for ChangePasswordError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::UnknownToken => StatusCode::UNAUTHORIZED,
            Self::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::PasswordsWontMatch => StatusCode::UNAUTHORIZED,
            Self::ValidationError(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            Self::UnknownToken => {
                HttpResponse::build(self.status_code()).json(get_fail_response(self.to_string()))
            }
            Self::UnexpectedError(_) => {
                HttpResponse::build(self.status_code()).json(get_error_response(self.to_string()))
            }
            Self::PasswordsWontMatch => {
                HttpResponse::build(self.status_code()).json(get_fail_response(self.to_string()))
            }
            Self::ValidationError(_) => {
                HttpResponse::build(self.status_code()).json(get_error_response(self.to_string()))
            }
        }
    }
}
