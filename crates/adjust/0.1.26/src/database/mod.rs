use std::ops::{Deref, DerefMut};

use diesel::r2d2::{ManageConnection, PooledConnection};

pub mod postgres;
pub mod redis;
pub mod util;

/// Database pool
///
/// Example:
/// ```rs
/// struct AppState {
///   postgres: Pool<Postgres>,
///   redis: Pool<Redis>
/// }
/// ```
#[derive(Clone)]
pub struct Pool<T: ManageConnection> {
  pub inner: diesel::r2d2::Pool<T>
}

impl<T: ManageConnection> From<diesel::r2d2::Pool<T>> for Pool<T> {
  fn from(inner: diesel::r2d2::Pool<T>) -> Self {
    Self { inner }
  }
}

impl<T: ManageConnection> Deref for Pool<T> {
  type Target = diesel::r2d2::Pool<T>;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl<T: ManageConnection> DerefMut for Pool<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.inner
  }
}
/// Database with estabiled connection
///
/// Example:
/// ```rs
/// #[derive(Serialize)]
/// struct Message {
///   message: String
/// }
///
/// async fn handler(
///   State(state): State<AppState>
/// ) -> HttpResult<Message> {
///   let db: Database<Postgres> = state.postgres.get()?;
///
///   // some logic here
///
///   Ok(Json(Message {
///     message: name
///   }))
/// }
/// ```
pub type Database<T> = PooledConnection<T>;

/// ``PoolBuilder`` - Database pool builder
pub trait PoolBuilder<T: ManageConnection> {
  fn create_pool() -> anyhow::Result<Pool<T>>;
}

impl<T: ManageConnection + PoolBuilder<T>> Default for Pool<T> {
  fn default() -> Self {
    <T as PoolBuilder<T>>::create_pool()
      .expect("database pool creation failed")
  }
}