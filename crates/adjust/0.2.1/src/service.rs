use anyhow::Result;
use std::future::Future;
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
  name: &'static str,
  state: S,
  controllers: ControllerList<S>,
  port: Option<u32>,
  dev_port: Option<u32>,
  ip: Option<&'static str>,
  emoji: Option<char>,
  // todo
  migrations: bool
}

impl<S> Service<S>
where
  S: Clone + Send + Sync + 'static + Default
{
  pub fn new(name: &'static str) -> Self {
    Self::enviroment();

    Self {
      name,
      ..Default::default()
    }
  }

  pub fn state(&mut self, state: S) -> &mut Self {
    self.state = state;
    self
  }

  pub fn controllers(&mut self, controllers: ControllerList<S>) -> &mut Self {
    self.controllers = controllers;
    self
  }

  pub fn port(&mut self, port: u32) -> &mut Self {
    self.port = Some(port);
    self
  }

  pub fn dev_port(&mut self, port: u32) -> &mut Self {
    self.dev_port = Some(port);
    self
  }

  pub fn ip(&mut self, ip: &'static str) -> &mut Self {
    self.ip = Some(ip);
    self
  }

  pub fn emoji(&mut self, emoji: char) -> &mut Self {
    self.emoji = Some(emoji);
    self
  }

  pub fn migrations(&mut self, migrations: bool) -> &mut Self {
    self.migrations = migrations;
    self
  }

  pub fn enviroment() {
    if cfg!(debug_assertions) {
      dotenv::dotenv()
        .expect("unable to find .env file");
    }

    env_logger::init();
  }

  pub fn run(self) -> impl Future<Output = Result<()>> {
    WebServer::start(self.name, self.state, self.controllers, self.port, self.dev_port, self.ip, self.emoji)
  }
}