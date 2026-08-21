// ========== src/config.rs ==========
use crate::shuffle::ShuffleScheduler;

#[derive(Clone, Debug)]  // Debugを追加
pub struct A2FConfig {
    pub buffer_timeout_secs: u64,
    pub buffer_max_size: usize,
    pub key_probability: f64,
    pub max_burst: usize,
    pub dummy_probability: f64,
    pub replay_window_size: u64,
}

impl Default for A2FConfig {
    fn default() -> Self {
        Self {
            buffer_timeout_secs: 10,
            buffer_max_size: 10000,
            key_probability: 0.3,
            max_burst: 5,
            dummy_probability: 0.05,
            replay_window_size: 1024,  // 1024に拡大
        }
    }
}

impl A2FConfig {
    pub fn into_scheduler(&self) -> ShuffleScheduler {
        ShuffleScheduler::new(
            self.key_probability,
            self.max_burst,
            self.dummy_probability,
        )
    }
}