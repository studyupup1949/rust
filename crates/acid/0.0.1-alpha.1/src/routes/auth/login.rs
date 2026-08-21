use crate::authentication::{validate_credentials, AuthError, Credentials};
use crate::session_state::TypedSession;
use crate::utils::{error_chain_fmt, get_error_response, get_fail_response};
use actix_web::{error::InternalError, http::StatusCode, web, HttpResponse, ResponseError};
use sqlx::PgPool;

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct LoginRequest {
    #[schema(example = "user123", required = true)]
    username: String,
    #[schema(example = "Password123", required = true)]
    password: String,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct LoginResponse {
    #[schema(example = "success")]
    status: String,
    #[schema(example = "")]
    data: serde_json::Value,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct LoginErrorResponse {
    #[schema(example = "fail")]
    status: String,
    #[schema(example = "Invalid input")]
    message: String,
}

#[utoipa::path(
    request_body = LoginRequest,
    post,
    path = "/api/v1/auth/login",
    responses(
        (status = 200, description = "User logged in", body = LoginResponse),
        (status = 400, description = "Invalid input", body = LoginErrorResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
)]
#[tracing::instrument(
    skip(request, pool, session),
    fields(
        username=tracing::field::Empty,
        user_id=tracing::field::Empty
    )
)]
pub async fn login(
    request: web::Json<LoginRequest>,
    pool: web::Data<PgPool>,
    session: TypedSession,
) -> Result<web::Json<LoginResponse>, InternalError<LoginError>> {
    let credentials = Credentials {
        username: request.0.username,
        password: request.0.password,
    };

    tracing::Span::current().record("username", tracing::field::display(&credentials.username));

    match validate_credentials(credentials, &pool).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", tracing::field::display(&user_id));

            session.renew();
            session.insert_user_id(user_id).map_err(|e| {
                InternalError::from_response(
                    LoginError::UnexpectedError(e.into()),
                    HttpResponse::InternalServerError().finish(),
                )
            })?;

            Ok(web::Json(LoginResponse {
                status: "success".to_string(),
                data: serde_json::Value::Null,
            }))
        }
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
                AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
                AuthError::InactiveAccount(_) => LoginError::ForbiddenError(e.into()),
            };

            match e {
                LoginError::AuthError(_) => Err(InternalError::from_response(
                    e,
                    HttpResponse::Ok()
                        .status(StatusCode::UNAUTHORIZED)
                        .json(get_fail_response("Authentication failed.".to_string())),
                )),
                LoginError::UnexpectedError(_) => Err(InternalError::from_response(
                    e,
                    HttpResponse::Ok()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .json(get_error_response("Something went wrong.".to_string())),
                )),
                LoginError::ForbiddenError(_) => Err(InternalError::from_response(
                    e,
                    HttpResponse::Ok()
                        .status(StatusCode::FORBIDDEN)
                        .json(get_fail_response("Account is not activated.".to_string())),
                )),
            }
        }
    }
}

#[derive(thiserror::Error)]
pub enum LoginError {
    #[error("Authentication failed.")]
    AuthError(#[source] anyhow::Error),
    #[error("Account is inactive.")]
    ForbiddenError(#[source] anyhow::Error),
    #[error("Something went wrong.")]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for LoginError {
    fn status_code(&self) -> StatusCode {
        match self {
            LoginError::AuthError(_) => StatusCode::BAD_REQUEST,
            LoginError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            LoginError::ForbiddenError(_) => StatusCode::FORBIDDEN,
        }
    }
}
