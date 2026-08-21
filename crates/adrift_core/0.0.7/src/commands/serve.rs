#![allow(dead_code, unused_variables, unused_must_use)]

use crate::{
    commands::{Arg, Command},
    Routes,
};
use rocket::Rocket;
use rocket_dyn_templates::Template;
use silhouette::facade::Container;
use std::collections::HashMap;

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
        let rocket = Rocket::build();
        let figment = rocket
            .figment()
            .clone()
            .merge(("template_dir", "resources/views"))
            .merge(("static_dir", "public"));

        rocket
            .attach(Template::fairing())
            .configure(figment)
            .mount("/", routes.inner.clone())
            .ignite()
            .await?
            .launch()
            .await?;

        Ok(())
    }
}
