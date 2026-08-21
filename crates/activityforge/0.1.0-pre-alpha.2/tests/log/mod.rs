/// Initializes the test logger.
pub fn init_logger() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
        .try_init()
        .ok();
}
