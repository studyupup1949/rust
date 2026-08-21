use crate::{controller::ControllerList, server::WebServer};

/// A ``declarative`` structure for fast web service creation
///
/// # Example
/// ```rs
/// async fn main() -> anyhow::Result<()> {
///   let service = Service {
///     name: "Example",
///     state: AppState {
///       postgres: PostgresConnection::try_connect()?
///     }, // or AppState::default()
///     controllers: [TestController],
///     ..Default::default() // port: None
///   };
///
///   service.run()
///     .await
/// }
/// ```
#[derive(Default)]
pub struct Service<S>
where
  S: Clone + Send + Sync + 'static,
{
  pub name: &'static str,
  pub state: S,
  pub controllers: ControllerList<S>,
  pub port: Option<u32>,
  pub dev_port: Option<u32>,
  pub migrations: bool
}

impl<S> Service<S>
where
  S: Clone + Send + Sync + 'static,
{
  pub async fn run(self) -> anyhow::Result<()> {
    log::info!("starting service {}", self.name);

    WebServer::start(self.state, self.controllers, self.port, self.dev_port)
      .await
  }
}