use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use sha2::{Sha256, Digest};
use std::collections::HashMap;

#[derive(Clone)]
pub struct CrystalConfig {
    pub size: usize,
    pub loss_rate: f64,
}

impl Default for CrystalConfig {
    fn default() -> Self {
        CrystalConfig {
            size: 64,
            loss_rate: 0.3,
        }
    }
}

pub struct MessageCrystal {
    id: u64,
    size: usize,
    cells: Vec<u8>,
    rng: ChaCha8Rng,
}

impl MessageCrystal {
    pub fn new(id: u64, size: usize, seed: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(seed);
        hasher.update(&id.to_le_bytes());
        let result = hasher.finalize();
        let mut seed_bytes = [0u8; 32];
        seed_bytes.copy_from_slice(&result[..32]);
        
        let total_bits = size * size * size;
        let total_bytes = (total_bits + 7) / 8;
        
        MessageCrystal {
            id,
            size,
            cells: vec![0; total_bytes],
            rng: ChaCha8Rng::from_seed(seed_bytes),
        }
    }
    
    fn index(&self, x: usize, y: usize, z: usize) -> usize {
        (x * self.size * self.size) + (y * self.size) + z
    }
    
    pub fn get(&self, idx: usize) -> u8 {
        if idx >= self.cells.len() * 8 {
            return 0;
        }
        (self.cells[idx / 8] >> (idx % 8)) & 1
    }
    
    pub fn set(&mut self, idx: usize, value: u8) {
        if idx >= self.cells.len() * 8 {
            return;
        }
        if value == 1 {
            self.cells[idx / 8] |= 1 << (idx % 8);
        } else {
            self.cells[idx / 8] &= !(1 << (idx % 8));
        }
    }
    
    fn find_cell(&mut self) -> usize {
        loop {
            let idx = self.rng.gen_range(0..self.size * self.size * self.size);
            if self.get(idx) == 0 {
                return idx;
            }
        }
    }
    
    pub fn encode(&mut self, message: &[u8]) -> Vec<usize> {
        let mut coords = Vec::new();
        for &byte in message {
            for bit in 0..8 {
                if (byte >> bit) & 1 == 1 {
                    let idx = self.find_cell();
                    self.set(idx, 1);
                    coords.push(idx);
                }
            }
        }
        coords
    }
    
    pub fn decode(&mut self, coords: &[usize]) -> Option<Vec<u8>> {
        let mut bits = vec![0; coords.len()];
        for (i, &idx) in coords.iter().enumerate() {
            if i < bits.len() {
                bits[i] = self.get(idx);
            }
        }
        
        for &idx in coords {
            self.set(idx, 0);
        }
        
        let mut result = Vec::new();
        for chunk in bits.chunks(8) {
            let mut byte = 0u8;
            for (i, &bit) in chunk.iter().enumerate() {
                if bit == 1 {
                    byte |= 1 << i;
                }
            }
            result.push(byte);
        }
        
        Some(result)
    }
    
    pub fn wipe(&mut self) {
        for byte in &mut self.cells {
            *byte = 0;
        }
    }
    
    pub fn size(&self) -> usize {
        self.size
    }
    
    pub fn id(&self) -> u64 {
        self.id
    }
}

pub struct AdaptiveCrystal {
    messages: HashMap<u64, MessageCrystal>,
    next_id: u64,
    loss_rate: f64,
    base_seed: Vec<u8>,
}

impl AdaptiveCrystal {
    pub fn new(seed: &[u8]) -> Self {
        AdaptiveCrystal {
            messages: HashMap::new(),
            next_id: 1,
            loss_rate: 0.3,
            base_seed: seed.to_vec(),
        }
    }
    
    pub fn with_config(seed: &[u8], loss_rate: f64) -> Self {
        AdaptiveCrystal {
            messages: HashMap::new(),
            next_id: 1,
            loss_rate: loss_rate.clamp(0.0, 0.8),
            base_seed: seed.to_vec(),
        }
    }
    
    fn calculate_size(&self, message_len: usize) -> usize {
        let bits_needed = message_len * 8;
        let cells_needed = (bits_needed as f64 * (1.0 + self.loss_rate)) as usize;
        let size = (cells_needed as f64).cbrt().ceil() as usize;
        size.clamp(8, 256)
    }
    
    pub fn encode(&mut self, message: &[u8]) -> (u64, Vec<usize>, usize) {
        let id = self.next_id;
        self.next_id += 1;
        
        let crystal_size = self.calculate_size(message.len());
        let mut crystal = MessageCrystal::new(id, crystal_size, &self.base_seed);
        let coords = crystal.encode(message);
        
        self.messages.insert(id, crystal);
        (id, coords, crystal_size)
    }
    
    pub fn decode(&mut self, id: u64, coords: &[usize], crystal_size: usize) -> Option<Vec<u8>> {
        if let Some(crystal) = self.messages.get_mut(&id) {
            let result = crystal.decode(coords);
            if result.is_some() {
                self.messages.remove(&id);
            }
            return result;
        }
        
        let mut crystal = MessageCrystal::new(id, crystal_size, &self.base_seed);
        let result = crystal.decode(coords);
        if result.is_some() {
            self.messages.insert(id, crystal);
        }
        result
    }
    
    pub fn confirm(&mut self, id: u64) {
        self.messages.remove(&id);
    }
    
    pub fn set_loss_rate(&mut self, loss_rate: f64) {
        self.loss_rate = loss_rate.clamp(0.0, 0.8);
    }
    
    pub fn pending_count(&self) -> usize {
        self.messages.len()
    }
    
    pub fn cleanup(&mut self) {
        self.messages.clear();
    }
}