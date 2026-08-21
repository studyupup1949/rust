use activitystreams_vocabulary::{field_access, impl_default, impl_display};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

/// Represents configuration settings for a DB connection.
///
/// ```rust
/// use activityforge::db::DbConfig;
///
/// let username = "db_user";
/// let password = "db_password123";
/// let db_name = "test_db";
/// let host = "localhost";
/// let port = 5432u16;
///
/// let db_conn = format!("postgres://{username}:{password}@{host}:{port}/{db_name}");
///
/// let db_config = DbConfig::new()
///     .with_username(username)
///     .with_password(password)
///     .with_db_name(db_name)
///     .with_host(host)
///     .with_port(port);
///
/// assert_eq!(db_config.postgres_connection(), db_conn);
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DbConfig {
    username: String,
    #[serde(skip)]
    password: Zeroizing<String>,
    db_name: String,
    host: String,
    port: u16,
    max_connections: u32,
}

impl DbConfig {
    /// Represents the default PostgreSQL connection port.
    pub const POSTGRES_PORT: u16 = 5432;

    /// Creates a new [DbConfig].
    pub fn new() -> Self {
        Self {
            username: String::new(),
            password: Zeroizing::new(String::new()),
            db_name: String::new(),
            host: String::new(),
            port: 0,
            max_connections: 5,
        }
    }

    /// Gets a reference to the password.
    pub fn password(&self) -> &str {
        &self.password
    }

    /// Sets the password.
    pub fn set_password<I: Into<String>>(&mut self, val: I) {
        self.password = Zeroizing::new(val.into());
    }

    /// Builder function that sets the password.
    pub fn with_password<I: Into<String>>(self, val: I) -> Self {
        Self {
            password: Zeroizing::new(val.into()),
            ..self
        }
    }

    /// Gets the PostgreSQL connection string for the [DbConfig].
    pub fn postgres_connection(&self) -> String {
        let username = self.username();
        let password = self.password();
        let host = self.host();
        let db_name = self.db_name();
        let port = if self.port == 0 {
            Self::POSTGRES_PORT
        } else {
            self.port
        };

        format!("postgres://{username}:{password}@{host}:{port}/{db_name}")
    }
}

field_access! {
    DbConfig {
        /// The username used to connect to the database.
        username: as_ref { &str, String },
        /// The database name.
        db_name: as_ref { &str, String },
        /// The database host.
        host: as_ref { &str, String },
    }
}

field_access! {
    DbConfig {
        /// The database port.
        port: u16,
        /// The maximum number of connections to the database.
        max_connections: u32,
    }
}

impl_default!(DbConfig);
impl_display!(DbConfig, json);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_config() {
        let username = "db_user";
        let password = "db_password123";
        let db_name = "test_db";
        let host = "localhost";
        let port = 5432u16;

        let db_conn = format!("postgres://{username}:{password}@{host}:{port}/{db_name}");

        let db_config = DbConfig::new()
            .with_username(username)
            .with_password(password)
            .with_db_name(db_name)
            .with_host(host)
            .with_port(port);

        assert_eq!(db_config.username(), username);
        assert_eq!(db_config.password(), password);
        assert_eq!(db_config.db_name(), db_name);
        assert_eq!(db_config.host(), host);
        assert_eq!(db_config.port(), port);

        assert_eq!(db_config.postgres_connection(), db_conn);
    }
}
