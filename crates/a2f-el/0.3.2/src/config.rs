use crate::shuffle::ShuffleScheduler;

#[derive(Clone, Debug)]
pub struct A2FConfig {
    pub buffer_timeout_secs: u64,
    pub buffer_max_size: usize,
    pub key_probability: f64,
    pub max_burst: usize,
    pub replay_window_size: u64,
    pub session_timeout_secs: u64,  // 追加
}

impl Default for A2FConfig {
    fn default() -> Self {
        Self {
            buffer_timeout_secs: 10,
            buffer_max_size: 10000,
            key_probability: 0.3,
            max_burst: 5,
            replay_window_size: 1024,
            session_timeout_secs: 3600,  // 追加（デフォルト1時間）
        }
    }
}

impl A2FConfig {
    pub fn into_scheduler(&self) -> ShuffleScheduler {
        ShuffleScheduler::new() 
    }
}