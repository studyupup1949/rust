#![allow(dead_code, unused_variables, unused_must_use)]

use std::collections::HashMap;
use silhouette::facade::Container;
use crate::{
    commands::{Arg, Command},
    Routes,
};

pub struct Serve;

#[async_trait::async_trait]
impl Command for Serve {
    fn name(&self) -> &'static str {
        "serve"
    }

    fn description(&self) -> &'static str {
        "Serve the application"
    }

    fn args(&self) -> Vec<Arg> {
        vec![]
    }

    async fn handle(&self, _args: HashMap<String, String>) -> anyhow::Result<()> {
        let routes: Routes = Container::resolve()?;

        rocket::build()
            .mount("/", routes.inner.clone())
            .ignite()
            .await?
            .launch()
            .await?;

        Ok(())
    }
}
