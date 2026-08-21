use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub is_superuser: bool,
    pub created_at: String,
}

#[derive(Debug)]
pub enum AuthError {
    NotFound,
    DuplicateUsername,
    DuplicateEmail,
    InvalidPassword,
    Storage(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::NotFound => write!(f, "user not found"),
            AuthError::DuplicateUsername => write!(f, "username already exists"),
            AuthError::DuplicateEmail => write!(f, "email already exists"),
            AuthError::InvalidPassword => write!(f, "invalid password"),
            AuthError::Storage(msg) => write!(f, "storage error: {}", msg),
        }
    }
}

impl std::error::Error for AuthError {}

#[async_trait]
pub trait UserStore: Send + Sync + 'static {
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, AuthError>;

    async fn find_by_email(&self, _email: &str) -> Result<Option<User>, AuthError> {
        // Default: iterate all and filter. Implementations should override for efficiency.
        Err(AuthError::Storage("not implemented".to_string()))
    }

    async fn create_user(
        &self,
        username: &str,
        email: &str,
        name: &str,
        password: &str,
        is_superuser: bool,
    ) -> Result<User, AuthError>;

    async fn delete_user(&self, username: &str) -> Result<(), AuthError>;
}

pub fn hash_password(password: &str) -> Result<String, AuthError> {
    use argon2::password_hash::{rand_core::OsRng, SaltString};
    use argon2::{Argon2, PasswordHasher};

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AuthError::Storage(e.to_string()))?
        .to_string();
    Ok(hash)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AuthError> {
    use argon2::password_hash::PasswordHash;
    use argon2::{Argon2, PasswordVerifier};

    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| AuthError::Storage(e.to_string()))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn find_by_username_or_email<'a>(
    users: &'a [User],
    login: &str,
) -> Option<&'a User> {
    if login.contains('@') {
        users.iter().find(|u| u.email == login)
    } else {
        users.iter().find(|u| u.username == login)
    }
}

pub fn generate_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{:x}{:04x}", nanos, counter & 0xFFFF)
}

pub fn timestamp() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let hash = hash_password("secret123").unwrap();
        assert!(verify_password("secret123", &hash).unwrap());
        assert!(!verify_password("wrong", &hash).unwrap());
    }

    #[test]
    fn test_hash_unique_salts() {
        let h1 = hash_password("same").unwrap();
        let h2 = hash_password("same").unwrap();
        assert_ne!(h1, h2, "each hash should have a unique salt");
    }

    #[test]
    fn test_generate_id_unique() {
        let ids: Vec<String> = (0..10).map(|_| generate_id()).collect();
        let mut sorted = ids.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(ids.len(), sorted.len(), "all generated IDs should be unique");
    }

    #[test]
    fn test_auth_error_display() {
        assert_eq!(AuthError::NotFound.to_string(), "user not found");
        assert_eq!(AuthError::DuplicateUsername.to_string(), "username already exists");
        assert_eq!(AuthError::DuplicateEmail.to_string(), "email already exists");
        assert_eq!(AuthError::InvalidPassword.to_string(), "invalid password");
        assert_eq!(
            AuthError::Storage("disk full".to_string()).to_string(),
            "storage error: disk full"
        );
    }

    #[test]
    fn test_find_by_username_or_email() {
        let users = vec![
            User {
                id: "1".into(),
                username: "alice".into(),
                email: "alice@example.com".into(),
                name: "Alice".into(),
                password_hash: "".into(),
                is_superuser: false,
                created_at: "".into(),
            },
            User {
                id: "2".into(),
                username: "bob".into(),
                email: "bob@test.com".into(),
                name: "Bob".into(),
                password_hash: "".into(),
                is_superuser: false,
                created_at: "".into(),
            },
        ];

        assert_eq!(find_by_username_or_email(&users, "alice").unwrap().id, "1");
        assert_eq!(find_by_username_or_email(&users, "bob@test.com").unwrap().id, "2");
        assert!(find_by_username_or_email(&users, "unknown").is_none());
    }

    #[test]
    fn test_timestamp_format() {
        let ts = timestamp();
        assert!(ts.contains('T'), "expected ISO 8601 format, got: {}", ts);
    }
}
