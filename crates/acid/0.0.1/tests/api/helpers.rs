use acid::configuration::{get_configuration, DatabaseSettings};
use acid::startup::{get_connection_pool, Application};
use acid::telemetry::{get_subscriber, init_subscriber};
use argon2::{password_hash::SaltString, Algorithm, Argon2, Params, PasswordHasher, Version};
use chrono::Utc;
use once_cell::sync::Lazy;
use secrecy::Secret;
use serde_json::json;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use wiremock::MockServer;

// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub struct TestUser {
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub status: String,
    pub password: String,
}

impl TestUser {
    pub fn generate() -> Self {
        Self {
            user_id: Uuid::new_v4(),
            username: Uuid::new_v4().to_string(),
            email: Uuid::new_v4().to_string(),
            status: "active".to_string(),
            password: Uuid::new_v4().to_string(),
        }
    }

    async fn store(&self, pool: &PgPool) {
        let salt = SaltString::generate(&mut rand::thread_rng());

        let password_hash = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(15000, 2, 1, None).unwrap(),
        )
        .hash_password(self.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

        sqlx::query!(
            "INSERT INTO users (user_id, username, email, status, password_hash, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)",
            self.user_id,
            self.username,
            self.email,
            self.status,
            password_hash,
            Utc::now()
        )
        .execute(pool)
        .await
        .expect("Failed to store test user.");
    }
}

pub struct TestApp {
    pub port: u16,
    pub address: String,
    pub db_pool: PgPool,
    pub email_server: MockServer,
    pub test_user: TestUser,
    pub api_client: reqwest::Client,
}

impl TestApp {
    pub async fn post_activate_resend(&self, body: String) -> reqwest::Response {
        self.api_client
            .post(&format!(
                "{}/api/v1/auth/signup/activate/resend",
                &self.address
            ))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_reset_password_request(&self, body: String) -> reqwest::Response {
        self.api_client
            .post(&format!(
                "{}/api/v1/auth/change_password/request",
                &self.address
            ))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_change_password(&self, body: String) -> reqwest::Response {
        self.api_client
            .post(&format!("{}/api/v1/auth/change_password", &self.address))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub fn get_activation_link(&self, email_request: &wiremock::Request) -> reqwest::Url {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();

            assert_eq!(links.len(), 1);

            let raw_link = links[0].as_str().to_owned();

            let mut activation_link = reqwest::Url::parse(&raw_link).unwrap();

            // Make sure we don't call random APIs on the web
            assert_eq!(activation_link.host_str().unwrap(), "127.0.0.1");
            activation_link.set_port(Some(self.port)).unwrap();
            activation_link
        };

        let data = json!(body);
        let message = &data["Messages"][0];

        let activation_link = get_link(&message["Variables"]["activation_link"].as_str().unwrap());

        activation_link
    }

    pub fn get_reset_token(&self, email_request: &wiremock::Request) -> String {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

        let data = json!(body);

        let message = &data["Messages"][0];

        let password_reset_link = reqwest::Url::parse(
            &message["Variables"]["password_reset_link"]
                .as_str()
                .unwrap(),
        )
        .unwrap();

        // Make sure we don't call random APIs on the web
        assert_eq!(password_reset_link.host_str().unwrap(), "127.0.0.1");

        let query = password_reset_link.query().unwrap();

        let parts: Vec<&str> = query.split('=').collect();

        parts[1].to_string()
    }

    pub async fn post_login(&self, body: String) -> reqwest::Response {
        self.api_client
            .post(&format!("{}/api/v1/auth/login", &self.address))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_signup(&self, body: String) -> reqwest::Response {
        self.api_client
            .post(&format!("{}/api/v1/auth/signup", &self.address))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let email_server = MockServer::start().await;

    // randomize configuration to ensure test isolation
    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        c.email_client.base_url = email_server.uri();
        c
    };

    configure_database(&configuration.database).await;

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");
    let application_port = application.port();
    let _ = tokio::spawn(application.run_until_stopped());

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .unwrap();

    let test_app = TestApp {
        address: format!("http://localhost:{}", application_port),
        port: application_port,
        db_pool: get_connection_pool(&configuration.database),
        email_server,
        test_user: TestUser::generate(),
        api_client: client,
    };
    test_app.test_user.store(&test_app.db_pool).await;
    test_app
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // create database
    let maintenance_settings = DatabaseSettings {
        database_name: "postgres".to_string(),
        username: "postgres".to_string(),
        password: Secret::new("password".to_string()),
        ..config.clone()
    };
    let mut connection = PgConnection::connect_with(&maintenance_settings.connect_options())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    // migrate database
    let connection_pool = PgPool::connect_with(config.connect_options())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    connection_pool
}
