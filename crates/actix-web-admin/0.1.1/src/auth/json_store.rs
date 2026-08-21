use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::auth::store::{self, AuthError, User};

pub struct JsonUserStore {
    path: PathBuf,
    users: Mutex<Vec<User>>,
}

impl JsonUserStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let users = if path.exists() {
            let data = std::fs::read_to_string(&path)
                .unwrap_or_else(|_| "[]".to_string());
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Vec::new()
        };
        JsonUserStore {
            path,
            users: Mutex::new(users),
        }
    }

    fn save(&self) -> Result<(), AuthError> {
        let users = self.users.lock().map_err(|e| {
            AuthError::Storage(format!("lock error: {}", e))
        })?;
        let data = serde_json::to_string_pretty(&*users)
            .map_err(|e| AuthError::Storage(e.to_string()))?;
        std::fs::write(&self.path, data)
            .map_err(|e| AuthError::Storage(e.to_string()))?;
        Ok(())
    }

    fn find_by_username_locked(&self, username: &str) -> Result<Option<User>, AuthError> {
        let users = self.users.lock().map_err(|e| {
            AuthError::Storage(format!("lock error: {}", e))
        })?;
        Ok(users.iter().find(|u| u.username == username).cloned())
    }

    fn find_by_email_locked(&self, email: &str) -> Result<Option<User>, AuthError> {
        let users = self.users.lock().map_err(|e| {
            AuthError::Storage(format!("lock error: {}", e))
        })?;
        Ok(users.iter().find(|u| u.email == email).cloned())
    }
}

#[async_trait]
impl store::UserStore for JsonUserStore {
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, AuthError> {
        self.find_by_username_locked(username)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AuthError> {
        self.find_by_email_locked(email)
    }

    async fn create_user(
        &self,
        username: &str,
        email: &str,
        name: &str,
        password: &str,
        is_superuser: bool,
    ) -> Result<User, AuthError> {
        {
            let users = self.users.lock().map_err(|e| {
                AuthError::Storage(format!("lock error: {}", e))
            })?;
            if users.iter().any(|u| u.username == username) {
                return Err(AuthError::DuplicateUsername);
            }
            if users.iter().any(|u| u.email == email) {
                return Err(AuthError::DuplicateEmail);
            }
        }

        let password_hash = store::hash_password(password)?;
        let user = User {
            id: store::generate_id(),
            username: username.to_string(),
            email: email.to_string(),
            name: name.to_string(),
            password_hash,
            is_superuser,
            created_at: store::timestamp(),
        };

        {
            let mut users = self.users.lock().map_err(|e| {
                AuthError::Storage(format!("lock error: {}", e))
            })?;
            users.push(user.clone());
        }
        self.save()?;
        Ok(user)
    }

    async fn delete_user(&self, username: &str) -> Result<(), AuthError> {
        let mut users = self.users.lock().map_err(|e| {
            AuthError::Storage(format!("lock error: {}", e))
        })?;
        let len_before = users.len();
        users.retain(|u| u.username != username);
        if users.len() == len_before {
            return Err(AuthError::NotFound);
        }
        drop(users);
        self.save()?;
        Ok(())
    }
}

impl JsonUserStore {
    pub fn all_users(&self) -> Result<Vec<User>, AuthError> {
        let users = self.users.lock().map_err(|e| {
            AuthError::Storage(format!("lock error: {}", e))
        })?;
        Ok(users.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::store::UserStore;

    #[tokio::test]
    async fn test_create_and_find() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_users3.json");
        let _ = std::fs::remove_file(&path);

        let store = JsonUserStore::new(&path);
        let user = store
            .create_user("alice", "alice@example.com", "Alice", "p4ss", true)
            .await
            .unwrap();
        assert_eq!(user.username, "alice");
        assert_eq!(user.email, "alice@example.com");
        assert_eq!(user.name, "Alice");
        assert!(user.is_superuser);

        let found = store.find_by_username("alice").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().email, "alice@example.com");

        let found = store.find_by_email("alice@example.com").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().username, "alice");

        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn test_duplicate_username() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_users_dup3.json");
        let _ = std::fs::remove_file(&path);

        let store = JsonUserStore::new(&path);
        store
            .create_user("bob", "bob@test.com", "Bob", "pass1", false)
            .await
            .unwrap();
        let result = store
            .create_user("bob", "other@test.com", "Other", "pass2", false)
            .await;
        assert!(matches!(result, Err(AuthError::DuplicateUsername)));

        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn test_duplicate_email() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_users_dup_email.json");
        let _ = std::fs::remove_file(&path);

        let store = JsonUserStore::new(&path);
        store
            .create_user("user1", "same@test.com", "User1", "pass1", false)
            .await
            .unwrap();
        let result = store
            .create_user("user2", "same@test.com", "User2", "pass2", false)
            .await;
        assert!(matches!(result, Err(AuthError::DuplicateEmail)));

        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn test_delete_user() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_users_delete.json");
        let _ = std::fs::remove_file(&path);

        let store = JsonUserStore::new(&path);
        store
            .create_user("dave", "dave@test.com", "Dave", "pass", false)
            .await
            .unwrap();

        let result = store.delete_user("dave").await;
        assert!(result.is_ok(), "should delete existing user");

        let found = store.find_by_username("dave").await.unwrap();
        assert!(found.is_none(), "deleted user should not be found");

        let result = store.delete_user("nonexistent").await;
        assert!(matches!(result, Err(AuthError::NotFound)));

        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn test_persistence() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_users_persist3.json");
        let _ = std::fs::remove_file(&path);

        {
            let store = JsonUserStore::new(&path);
            store
                .create_user("carol", "carol@test.com", "Carol", "secret", false)
                .await
                .unwrap();
        }

        {
            let store = JsonUserStore::new(&path);
            let found = store.find_by_username("carol").await.unwrap();
            assert!(found.is_some(), "user should persist to disk");
            assert_eq!(found.unwrap().email, "carol@test.com");
        }

        let _ = std::fs::remove_file(&path);
    }
}
