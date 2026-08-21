use crate::configuration::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use crate::routes::{auth, health_check};

use crate::utils::get_error_response;
use actix_session::{storage::RedisSessionStore, SessionMiddleware};
use actix_web::{
    cookie::Key, dev::Server, error, error::JsonPayloadError, web, web::Data, web::JsonConfig, App,
    HttpResponse, HttpServer,
};
use secrecy::{ExposeSecret, Secret};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, anyhow::Error> {
        let connection_pool = get_connection_pool(&configuration.database);
        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address.");
        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            sender_email,
            configuration.email_client.api_key,
            configuration.email_client.api_token,
        );

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr()?.port();
        let server = run(
            listener,
            connection_pool,
            email_client,
            configuration.application.base_url,
            configuration.application.hmac_secret,
            configuration.redis_uri,
        )
        .await?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new().connect_lazy_with(configuration.connect_options())
}

pub struct ApplicationBaseUrl(pub String);

async fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
    base_url: String,
    hmac_secret: Secret<String>,
    redis_uri: Secret<String>,
) -> Result<Server, anyhow::Error> {
    #[derive(OpenApi)]
    #[openapi(
        paths(
            auth::login,
            auth::request_password_reset,
            auth::signup,
            auth::signup_activate,
            auth::signup_activate_resend,
            auth::change_password,
        ),
        components(schemas(
            auth::ChangePasswordRequest,
            auth::ChangePasswordResponse,
            auth::LoginRequest,
            auth::LoginResponse,
            auth::LoginErrorResponse,
            auth::PasswordResetRequest,
            auth::PasswordResetResponse,
            auth::SignupActivateResponse,
            auth::SignupRequest,
            auth::SignupResponse,
        ))
    )]
    struct ApiDoc;

    let db_pool = Data::new(db_pool);
    let email_client = Data::new(email_client);
    let base_url = Data::new(ApplicationBaseUrl(base_url));

    let secret_key = Key::from(hmac_secret.expose_secret().as_bytes());

    let redis_store = RedisSessionStore::new(redis_uri.expose_secret()).await?;

    let server = HttpServer::new(move || {
        App::new()
            .wrap(SessionMiddleware::new(
                redis_store.clone(),
                secret_key.clone(),
            ))
            .wrap(TracingLogger::default())
            .service(
                web::scope("/api/v1").service(
                    web::scope("/auth")
                        .route("/login", web::post().to(auth::login))
                        .route("/signup", web::post().to(auth::signup))
                        .route("/signup/activate", web::get().to(auth::signup_activate))
                        .route(
                            "/signup/activate/resend",
                            web::post().to(auth::signup_activate_resend),
                        )
                        .route(
                            "/change_password/request",
                            web::post().to(auth::request_password_reset),
                        )
                        .route("/change_password", web::post().to(auth::change_password)),
                ),
            )
            .service(
                RapiDoc::with_openapi("/api-docs/openapi.json", ApiDoc::openapi())
                    .path("/docs/api"),
            )
            .route("/health_check", web::get().to(health_check))
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
            .app_data(ApiError::json_error(JsonConfig::default()))
            .app_data(Data::new(HmacSecret(hmac_secret.expose_secret().clone())))
    })
    .listen(listener)?
    .run();

    Ok(server)
}

#[derive(Clone)]
pub struct HmacSecret(pub String);

pub struct ApiError;

impl ApiError {
    pub fn json_error(cfg: JsonConfig) -> JsonConfig {
        cfg.limit(4096)
            .error_handler(|err: JsonPayloadError, _req| {
                let error = err.to_string();
                let slice = &error[..error.find(" at").unwrap()];

                // create custom error response
                error::InternalError::from_response(
                    err,
                    HttpResponse::BadRequest().json(get_error_response(slice.to_string())),
                )
                .into()
            })
    }
}
