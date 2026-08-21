use axum::Router;
use crate::controller::{ApplyControllerOnRouter, ControllerList};

/// A structure that raises the ``axum`` server with all the specified controllers.
///
/// This is done as a structure to improve code readability.
pub struct WebServer;

impl WebServer {
  /// Example
  ///
  /// ```rs
  /// pub struct AppState {
  ///   postgres: PostgresPool,
  ///   redis: RedisPool,
  /// }
  ///
  /// #[tokio::main]
  /// async fn main() -> anyhow::Result<()> {
  ///   let state = AppState {
  ///     postgres: postgres::connect()?,
  ///     redis: redis::connect()?
  ///   }
  ///
  ///   WebService::start(state, [UserController], None)
  ///       .await
  /// }
  /// ```
  pub async fn start<S>(state: S, controllers: ControllerList<S>, port: Option<u32>, dev_port: Option<u32>) -> anyhow::Result<()>
  where
    S: Clone + Send + Sync + 'static
  {
    // idk if it will work, like, it's macro, not a runtime check
    let port = if cfg!(debug_assertions) {
      dev_port.unwrap_or(8080)
    } else {
      port.unwrap_or(80)
    };

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
      .await?;

    let router = Router::new()
      .use_controllers(controllers)
      .with_state(state);

    Ok(axum::serve(listener, router)
      .await?)
  }

  pub fn enviroment() {
    if cfg!(debug_assertions) {
      dotenv::dotenv()
        .expect("Unable to find .env file");

      log::debug!("running in debug mode");
    }

    env_logger::init();
  }
}