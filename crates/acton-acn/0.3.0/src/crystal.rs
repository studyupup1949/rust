use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use sha2::{Sha256, Digest};

#[derive(Clone)]
pub struct CrystalConfig {
    pub size: usize,
}

impl Default for CrystalConfig {
    fn default() -> Self {
        CrystalConfig { size: 64 }
    }
}

pub struct Crystal {
    config: CrystalConfig,
    cube: Vec<u8>,
    rng: ChaCha8Rng,
}

impl Crystal {
    pub fn new(seed: &[u8], config: CrystalConfig) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(seed);
        let result = hasher.finalize();
        let bytes = result.as_slice();
        let mut seed_bytes = [0u8; 32];
        let len = bytes.len().min(32);
        seed_bytes[..len].copy_from_slice(&bytes[..len]);
        let rng = ChaCha8Rng::from_seed(seed_bytes);
        let total_bits = config.size * config.size * config.size;
        let total_bytes = (total_bits + 7) / 8;
        Crystal {
            config,
            cube: vec![0; total_bytes],
            rng,
        }
    }

    fn index(&self, x: usize, y: usize, z: usize) -> usize {
        (x * self.config.size * self.config.size) + (y * self.config.size) + z
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> u8 {
        let idx = self.index(x, y, z);
        (self.cube[idx / 8] >> (idx % 8)) & 1
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, value: u8) {
        let idx = self.index(x, y, z);
        let byte = &mut self.cube[idx / 8];
        let bit = idx % 8;
        if value == 1 {
            *byte |= 1 << bit;
        } else {
            *byte &= !(1 << bit);
        }
    }

    // Простой последовательный поиск (для теста)
    fn find_cell_sequential(&mut self, target: u8) -> (usize, usize, usize) {
        for x in 0..self.config.size {
            for y in 0..self.config.size {
                for z in 0..self.config.size {
                    if self.get(x, y, z) == target {
                        return (x, y, z);
                    }
                }
            }
        }
        (0, 0, 0) // fallback
    }

    pub fn encode(&mut self, message: &[u8]) -> Vec<(usize, usize, usize)> {
        let mut coords = Vec::new();
        for &byte in message {
            for bit in 0..8 {
                if (byte >> bit) & 1 == 1 {
                    let coord = self.find_cell_sequential(0);
                    self.set(coord.0, coord.1, coord.2, 1);
                    coords.push(coord);
                }
            }
        }
        coords
    }

    pub fn decode(&mut self, coords: &[(usize, usize, usize)], expected_len: usize) -> Option<Vec<u8>> {
        let total_bits = expected_len * 8;
        let mut bits = vec![0; total_bits];
        
        for (i, &(x, y, z)) in coords.iter().enumerate() {
            if i < total_bits {
                if self.get(x, y, z) == 1 {
                    bits[i] = 1;
                }
            }
        }
        
        for &(x, y, z) in coords {
            self.set(x, y, z, 0);
        }
        
        let mut result = Vec::with_capacity(expected_len);
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
        for byte in &mut self.cube {
            *byte = 0;
        }
    }
}