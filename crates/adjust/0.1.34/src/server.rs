use axum::Router;
use crate::controller::{ApplyControllerOnRouter, ControllerList};

const WILDCART: &str = "0.0.0.0";

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
  pub async fn start<S>(name: &'static str, state: S, controllers: ControllerList<S>, port: Option<u32>, dev_port: Option<u32>, ip_override: Option<&str>, emoji: Option<char>) -> anyhow::Result<()>
  where
    S: Clone + Send + Sync + 'static
  {
    let ip_address = ip_override.unwrap_or(WILDCART);
    let host = if ip_address == WILDCART { "localhost" } else { ip_address };

    let port = if cfg!(debug_assertions) {
      dev_port.unwrap_or(8080)
    } else {
      port.unwrap_or(80)
    };

    let ip = format!("{ip_address}:{port}");

    let listener = tokio::net::TcpListener::bind(ip.clone())
      .await?;

    let router = Router::new()
      .use_controllers(controllers)
      .with_state(state);

    #[cfg(feature = "cookie")]
    let router = router.layer(axum_cookie::CookieLayer::default());

    let emoji = emoji.unwrap_or('📡');

    log::info!("{emoji} {name} available at http://{host}:{port}");

    Ok(axum::serve(listener, router)
      .await?)
  }
}