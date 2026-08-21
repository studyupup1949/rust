use rand::prelude::*;

pub struct ShuffleScheduler {
    rng: ThreadRng,
}

impl ShuffleScheduler {
    pub fn new() -> Self {
        Self {
            rng: thread_rng(),
        }
    }

    /// パケットの順序をランダムに並べ替える
    pub fn shuffle_packets<T>(&mut self, mut packets: Vec<T>) -> Vec<T> {
        packets.shuffle(&mut self.rng);
        packets
    }
}

impl Default for ShuffleScheduler {
    fn default() -> Self {
        Self::new()
    }
}