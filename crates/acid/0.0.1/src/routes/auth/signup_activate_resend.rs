use crate::domain::{UserEmail, UserName};
use crate::email_client::EmailClient;
use crate::routes::auth::get_user_from_email;
use crate::routes::auth::signup::{generate_activation_token, send_activation_email, store_token};
use crate::startup::ApplicationBaseUrl;
use crate::utils::{error_chain_fmt, get_error_response};
use actix_web::{http::StatusCode, web, HttpResponse, ResponseError};
use anyhow::Context;
use sqlx::Executor;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

const TEMPLATE_ID: u64 = 3904091;
const NEW_SIGNUP_ACTIVATION_EMAIL_SUBJECT: &str =
    "Your new activation email - Please activate your AcidArchive.com account";

#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ActivateResendRequest {
    #[schema(example = "user123@gmail.com", required = true)]
    email: String,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct ActivateResendResponse {
    #[schema(example = "success")]
    status: String,
    #[schema(example = "")]
    data: serde_json::Value,
}

#[utoipa::path(
    request_body = ActivateResendRequest,
    post,
    path = "/api/v1/auth/signup/activate/resend",
    responses(
        (status = 202, description = "Activation email resent", body = ActivateResendResponse),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error")
    ),
)]
#[tracing::instrument(
    name = "Resend activation email",
    skip(request, pool, email_client, base_url)
)]
pub async fn signup_activate_resend(
    request: web::Json<ActivateResendRequest>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
) -> Result<HttpResponse, ActivationResendError> {
    if let Some(user) = get_user_from_email(&pool, &request.0.email)
        .await
        .map_err(ActivationResendError::UnexpectedError)?
    {
        let user_email = UserEmail::parse(user.email.as_ref().to_string()).unwrap();
        let user_name = UserName::parse(user.username.as_ref().to_string()).unwrap();

        let mut transaction = pool
            .begin()
            .await
            .context("Failed to acquire a Postgres connection from the pool")?;

        delete_user_activation_token(&mut transaction, user.user_id)
            .await
            .context("Failed deleting user activation tokens")?;

        let activation_token = generate_activation_token();
        store_token(&mut transaction, user.user_id, &activation_token)
            .await
            .context("Failed to store the verification token for a new user.")?;

        transaction
            .commit()
            .await
            .context("Failed to commit SQL transaction to activate account.")?;

        send_activation_email(
            &email_client,
            user_name,
            user_email,
            &base_url.0,
            &activation_token,
            &TEMPLATE_ID,
            NEW_SIGNUP_ACTIVATION_EMAIL_SUBJECT,
        )
        .await
        .context("Failed to send a confirmation email.")?;
    };

    let response = web::Json(ActivateResendResponse {
        status: "success".to_string(),
        data: serde_json::Value::Null,
    });

    Ok(HttpResponse::Accepted().json(response))
}

#[tracing::instrument(name = "Delete activation token", skip(user_id, transaction))]
pub async fn delete_user_activation_token(
    transaction: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    let query = sqlx::query!(
        r#"DELETE FROM activation_tokens WHERE user_id = $1"#,
        user_id,
    );
    transaction.execute(query).await?;
    Ok(())
}

#[derive(thiserror::Error)]
pub enum ActivationResendError {
    #[error("Something went wrong.")]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for ActivationResendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for ActivationResendError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn error_response(&self) -> HttpResponse {
        match self {
            ActivationResendError::UnexpectedError(_) => {
                HttpResponse::build(self.status_code()).json(get_error_response(self.to_string()))
            }
        }
    }
}
