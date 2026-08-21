// ========== scheduler.rs ==========
use rand::prelude::*;
use rand::distributions::Uniform;

pub struct ShuffleScheduler {
    rng: ThreadRng,
    key_probability: f64,
    burst_range: Uniform<usize>,
    dummy_probability: f64,
}

impl ShuffleScheduler {
    pub fn new(key_probability: f64, max_burst: usize, dummy_probability: f64) -> Self {
        Self {
            rng: thread_rng(),
            key_probability,
            burst_range: Uniform::new(1, max_burst + 1),
            dummy_probability,
        }
    }
    
    pub fn should_send_key(&mut self) -> bool {
        self.rng.gen_bool(self.key_probability)
    }
    
    pub fn should_send_dummy(&mut self) -> bool {
        self.rng.gen_bool(self.dummy_probability)
    }
    
    pub fn next_burst(&mut self) -> usize {
        self.burst_range.sample(&mut self.rng)
    }
    
    pub fn shuffle_packets<T>(&mut self, packets: Vec<T>) -> Vec<T> {
        let mut shuffled = packets;
        shuffled.shuffle(&mut self.rng);
        shuffled
    }
}

impl Default for ShuffleScheduler {
    fn default() -> Self {
        Self::new(0.3, 5, 0.05)
    }
}