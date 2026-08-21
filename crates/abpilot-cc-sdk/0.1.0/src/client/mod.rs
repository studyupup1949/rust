#[cfg(feature = "mp")]
pub mod mp;
#[cfg(feature = "app")]
pub mod app;

#[cfg(feature = "mp")]
pub use mp::MpClient;
#[cfg(feature = "app")]
pub use app::AppClient;

use crate::config::Config;

#[derive(Clone)]
pub struct AbpilotClient {
    #[cfg(feature = "mp")]
    mp_client: MpClient,
    #[cfg(feature = "app")]
    app_client: AppClient,
}

impl AbpilotClient {
    pub fn new() -> Self {
        Self::with_config(Config::default())
    }

    pub fn with_config(config: Config) -> Self {
        Self {
            #[cfg(feature = "mp")]
            mp_client: MpClient::new(config.mp_base_url),
            #[cfg(feature = "app")]
            app_client: AppClient::new(config.app_base_url),
        }
    }

    #[cfg(feature = "mp")]
    pub fn mp(&self) -> &MpClient {
        &self.mp_client
    }

    #[cfg(feature = "mp")]
    pub fn mp_mut(&mut self) -> &mut MpClient {
        &mut self.mp_client
    }

    #[cfg(feature = "app")]
    pub fn app(&self) -> &AppClient {
        &self.app_client
    }

    #[cfg(feature = "app")]
    pub fn app_mut(&mut self) -> &mut AppClient {
        &mut self.app_client
    }
}

impl Default for AbpilotClient {
    fn default() -> Self {
        Self::new()
    }
}
