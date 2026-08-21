use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Config {
    #[cfg(feature = "mp")]
    pub mp_base_url: String,
    #[cfg(feature = "app")]
    pub app_base_url: String,
    pub timeout: Duration,
    pub max_retries: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            #[cfg(feature = "mp")]
            mp_base_url: "https://wpyi6ctkdvfcxbqtmy6d6tkesi0yzzid.lambda-url.us-east-1.on.aws".to_string(),
            #[cfg(feature = "app")]
            app_base_url: "https://opnqqwytt7sgobosrlk6kxp5de0rolbu.lambda-url.us-east-1.on.aws".to_string(),
            timeout: Duration::from_secs(30),
            max_retries: 3,
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(feature = "mp")]
    pub fn with_mp_base_url(mut self, url: impl Into<String>) -> Self {
        self.mp_base_url = url.into();
        self
    }

    #[cfg(feature = "app")]
    pub fn with_app_base_url(mut self, url: impl Into<String>) -> Self {
        self.app_base_url = url.into();
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }
}
