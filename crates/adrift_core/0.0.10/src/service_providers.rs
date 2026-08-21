mod vite_provider;

#[crate::async_trait]
pub trait ServiceProvider: Send + Sync {
    /// Register any application services.
    async fn register(&self) -> anyhow::Result<()> {
        Ok(())
    }

    /// Bootstrap any application services.
    async fn boot(&self) -> anyhow::Result<()> {
        Ok(())
    }
}


pub struct ServiceProviders {
    pub items: Vec<Box<dyn ServiceProvider>>,
}

pub fn get_providers() -> Vec<Box<dyn ServiceProvider>> {
    vec![
        Box::new(vite_provider::ViteProvider),
    ]
}