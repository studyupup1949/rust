#![allow(dead_code, unused_variables, unused_must_use)]

use crate::{
    commands::{Arg, Command},
    RoutesConfig, TemplateConfig,
};
use clap::arg;
use rocket::{fs::FileServer, Rocket};
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
        vec![arg!(-d --dev ... "Run with frontend development server")]
    }

    async fn handle(&self, args: HashMap<String, String>) -> anyhow::Result<()> {
        let dev = args.get("dev").is_some();

        if dev {
            let mut cmd = std::process::Command::new("pnpm");
            cmd.arg("run").arg("dev");
            cmd.spawn()?;
        }
        let routes: RoutesConfig = Container::resolve()?;
        let tpl_config: TemplateConfig = Container::resolve()?;

        let rocket = Rocket::build();
        let figment = rocket
            .figment()
            .clone()
            .merge(("template_dir", "resources/views"))
            .merge(("static_dir", "public"));

        let tpl = Template::custom(move |engines| {
            for (name, fun) in tpl_config.functions.clone() {
                engines.tera.functions.insert(name, fun);
            }
        });

        rocket
            .attach(tpl)
            .configure(figment)
            .mount("/", routes.inner.clone())
            .mount("/", FileServer::from("public/"))
            .ignite()
            .await?
            .launch()
            .await?;

        Ok(())
    }
}
