use crate::domain::{NewUser, UserEmail, UserName, UserPassword};
use crate::email_client::EmailClient;
use crate::startup::ApplicationBaseUrl;
use crate::utils::{error_chain_fmt, get_error_response, get_fail_response, make_password_hash};
use actix_web::{http::StatusCode, web, HttpResponse, ResponseError};
use anyhow::Context;
use chrono::Utc;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use secrecy::Secret;
use sqlx::{Executor, PgPool, Postgres, Transaction};
use std::collections::HashMap;
use uuid::Uuid;

const TEMPLATE_ID: u64 = 3865334;
const SIGNUP_ACTIVATION_EMAIL_SUBJECT: &str =
    "Welcome to AcidArchive.com - Please activate your account";

#[derive(serde::Deserialize, Debug, utoipa::ToSchema)]
pub struct SignupRequest {
    #[schema(example = "user123", required = true)]
    username: String,
    #[schema(example = "user123@mail.com", required = true)]
    email: String,
    #[schema(example = "Password123!", required = true)]
    password: String,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct SignupResponse {
    #[schema(example = "success")]
    status: String,
    #[schema(example = "")]
    data: serde_json::Value,
}

#[derive(serde::Serialize, Debug)]
pub struct User {
    username: String,
    email: String,
}

pub fn parse_user(data: SignupRequest) -> Result<NewUser, String> {
    let username = UserName::parse(data.username)?;
    let email = UserEmail::parse(data.email)?;
    let password = UserPassword::parse(data.password)?;

    Ok(NewUser {
        username,
        email,
        password,
    })
}

#[utoipa::path(
    request_body = SignupRequest,
    post,
    path = "/api/v1/auth/signup",
    responses(
        (status = 201, description = "Signup successfully", body = SignupResponse),
        (status = 400, description = "Invalid input"),
        (status = 409, description = "Conflict"),
        (status = 500, description = "Internal server error")
    ),
)]
#[tracing::instrument(
    name = "Adding new user",
    skip(request, pool, email_client, base_url),
    fields(
        username = %request.username,
    )
)]
pub async fn signup(
    request: web::Json<SignupRequest>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
) -> Result<HttpResponse, SignupError> {
    let new_user = parse_user(request.0).map_err(SignupError::ValidationError)?;

    // check if username is available
    if let Some((_user_id, _username)) = get_stored_credentials(new_user.username.as_ref(), &pool)
        .await
        .map_err(SignupError::UnexpectedError)?
    {
        return Err(SignupError::ConflictError(
            "Username not available.".to_string(),
        ));
    }

    // check if email is available
    if let Some(_user_id) = get_user_id_by_email(new_user.email.as_ref(), &pool)
        .await
        .map_err(SignupError::UnexpectedError)?
    {
        return Err(SignupError::ConflictError(
            "Email not available.".to_string(),
        ));
    }

    // username & email available
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;

    let user_id = insert_user(&mut transaction, &new_user)
        .await
        .context("Failed to insert new user in the database.")?;

    let verify_token = generate_activation_token();
    store_token(&mut transaction, user_id, &verify_token)
        .await
        .context("Failed to store the verification token for a new user.")?;
    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a new user.")?;

    send_activation_email(
        &email_client,
        new_user.username,
        new_user.email,
        &base_url.0,
        &verify_token,
        &TEMPLATE_ID,
        SIGNUP_ACTIVATION_EMAIL_SUBJECT,
    )
    .await
    .context("Failed to send a confirmation email.")?;

    let response = web::Json(SignupResponse {
        status: "success".to_string(),
        data: serde_json::Value::Null,
    });

    Ok(HttpResponse::Created().json(response))
}

pub fn generate_activation_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

#[tracing::instrument(
    name = "Send an activation email to a new user",
    skip(email_client, username, email, base_url, activation_token)
)]
pub async fn send_activation_email(
    email_client: &EmailClient,
    username: UserName,
    email: UserEmail,
    base_url: &str,
    activation_token: &str,
    template_id: &u64,
    subject: &str,
) -> Result<(), reqwest::Error> {
    let activation_link = format!(
        "{}/api/v1/auth/signup/activate?activation_token={}",
        base_url, activation_token
    );

    let mut variables = HashMap::new();
    variables.insert(String::from("activation_link"), activation_link);
    variables.insert(String::from("username"), username.as_ref().to_string());

    email_client
        .send_email(&email, subject, template_id, variables)
        .await
}

#[tracing::instrument(name = "Get stored credentials", skip(username, pool))]
async fn get_stored_credentials(
    username: &str,
    pool: &PgPool,
) -> Result<Option<(uuid::Uuid, Secret<String>)>, anyhow::Error> {
    let row = sqlx::query!(
        r#"SELECT user_id, username, email, password_hash FROM users WHERE username = $1"#,
        username
    )
    .fetch_optional(pool)
    .await
    .context("Failed to perform a query to retrieve stored credentials.")?
    .map(|row| (row.user_id, Secret::new(row.password_hash)));

    Ok(row)
}

#[tracing::instrument(name = "Check if email is taken", skip(email, pool))]
pub async fn get_user_id_by_email(
    email: &str,
    pool: &PgPool,
) -> Result<Option<uuid::Uuid>, anyhow::Error> {
    let row = sqlx::query!(r#"SELECT user_id FROM users WHERE email = $1"#, email)
        .fetch_optional(pool)
        .await
        .context("Failed to perform a query to retrieve stored credentials.")?
        .map(|row| (row.user_id));

    Ok(row)
}

#[tracing::instrument(
    name = "Saving new user details in the database",
    skip(new_user, transaction)
)]
pub async fn insert_user(
    transaction: &mut Transaction<'_, Postgres>,
    new_user: &NewUser,
) -> Result<Uuid, sqlx::Error> {
    let user_id = Uuid::new_v4();

    let password_hash = make_password_hash(new_user.password.as_ref());

    let query = sqlx::query!(
        r#"
    INSERT INTO users (user_id, username, email, password_hash, created_at)
    VALUES ($1, $2, $3, $4, $5)
            "#,
        user_id,
        new_user.username.as_ref(),
        new_user.email.as_ref(),
        password_hash,
        Utc::now()
    );

    transaction.execute(query).await?;
    Ok(user_id)
}

#[derive(thiserror::Error)]
pub enum SignupError {
    #[error("{0}")]
    ValidationError(String),
    #[error("{0}")]
    ConflictError(String),
    #[error("Something went wrong.")]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for SignupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for SignupError {
    fn status_code(&self) -> StatusCode {
        match self {
            SignupError::ValidationError(_) => StatusCode::BAD_REQUEST,
            SignupError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            SignupError::ConflictError(_) => StatusCode::CONFLICT,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            SignupError::ValidationError(_) => {
                HttpResponse::build(self.status_code()).json(get_fail_response(self.to_string()))
            }
            SignupError::UnexpectedError(_) => {
                HttpResponse::build(self.status_code()).json(get_error_response(self.to_string()))
            }
            SignupError::ConflictError(_) => {
                HttpResponse::build(self.status_code()).json(get_fail_response(self.to_string()))
            }
        }
    }
}

#[tracing::instrument(
    name = "Store activation token in the database",
    skip(activation_token, transaction)
)]
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    activation_token: &str,
) -> Result<(), StoreSignupTokenError> {
    let query = sqlx::query!(
        r#"
    INSERT INTO activation_tokens (activation_token, user_id, created_at)
    VALUES ($1, $2, $3)
        "#,
        activation_token,
        user_id,
        Utc::now()
    );
    transaction
        .execute(query)
        .await
        .map_err(StoreSignupTokenError)?;
    Ok(())
}

pub struct StoreSignupTokenError(sqlx::Error);

impl std::error::Error for StoreSignupTokenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

impl std::fmt::Debug for StoreSignupTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl std::fmt::Display for StoreSignupTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A database error was encountered while trying to store a signup activation token."
        )
    }
}
