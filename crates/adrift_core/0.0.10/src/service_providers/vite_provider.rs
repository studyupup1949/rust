use crate::serde::json::Value;
use crate::templates::tera::{Error, Result};
use crate::{
    anyhow, async_trait, service_providers::ServiceProvider, templates::tera::Function, Container,
    TemplateConfig,
};
use std::{collections::HashMap, sync::Arc};
use std::{env, fs};
pub struct ViteProvider;

#[async_trait]
impl ServiceProvider for ViteProvider {
    async fn register(&self) -> anyhow::Result<()> {
        Container::singleton(&|container| {
            let mut config = container.resolve::<TemplateConfig>().unwrap();

            config.functions.insert("vite".to_string(), Arc::new(Vite));

            config
        })
        .expect("failed to bind state");
        Ok(())
    }

    async fn boot(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

struct Vite;

impl Function for Vite {
    fn is_safe(&self) -> bool {
        true
    }

    fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
        let mut entrypoints: Vec<String> = args
            .get("entrypoints")
            .ok_or_else(|| Error::msg("The `vite` function requires an `entrypoints` argument."))?
            .as_array()
            .ok_or_else(|| Error::msg("The `vite` function requires an `entrypoints` argument."))?
            .iter()
            .map(|v| {
                v.as_str()
                    .ok_or_else(|| {
                        Error::msg("The `vite` function requires an `entrypoints` argument.")
                    })
                    .map(|s| s.to_string())
            })
            .collect::<Result<Vec<String>>>()?;

        entrypoints.insert(0, "@vite/client".to_string());

        let base_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

        let base_url = fs::read_to_string(format!("{}/public/hot", base_dir)).unwrap_or("".to_string());

        dbg!(&base_url);

        if !base_url.is_empty() {
            let html = entrypoints
                .iter()
                .map(|entrypoint| {
                    if entrypoint.ends_with(".css") {
                        return format!(
                            r#"<link rel="stylesheet" href="{}/{}" />"#,
                            base_url, entrypoint
                        );
                    }

                    format!(
                        r#"<script type="module" src="{}/{}"></script>"#,
                        base_url, entrypoint
                    )
                })
                .collect::<Vec<String>>()
                .join("\n");

            Ok(Value::String(html))
        } else {
            Ok(Value::String("TODO".to_string()))
        }
    }
}
