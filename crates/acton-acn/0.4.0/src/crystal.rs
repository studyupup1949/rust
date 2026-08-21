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
    cells: Vec<u8>,
}

impl Crystal {
    pub fn new(_seed: &[u8], config: CrystalConfig) -> Self {
        let total_cells = config.size * config.size * config.size;
        Crystal {
            config,
            cells: vec![0; total_cells],
        }
    }

    pub fn get(&self, index: usize) -> u8 {
        self.cells.get(index).copied().unwrap_or(0)
    }

    pub fn set(&mut self, index: usize, value: u8) {
        if let Some(cell) = self.cells.get_mut(index) {
            *cell = value;
        }
    }

    pub fn encode(&mut self, message: &[u8]) -> Vec<usize> {
        let mut indices = Vec::new();
        let mut next_index = 0;
        
        for &byte in message {
            for bit in 0..8 {
                if ((byte >> bit) & 1) == 1 {
                    indices.push(next_index);
                    self.set(next_index, 1);
                    next_index += 1;
                }
            }
        }
        
        indices
    }

    pub fn decode(&mut self, indices: &[usize], expected_len: usize) -> Option<Vec<u8>> {
        let total_bits = expected_len * 8;
        let mut bits = vec![0; total_bits];
        
        for (i, &idx) in indices.iter().enumerate() {
            if i < total_bits {
                if self.get(idx) == 1 {
                    bits[i] = 1;
                }
            }
        }
        
        for &idx in indices {
            self.set(idx, 0);
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
        for cell in &mut self.cells {
            *cell = 0;
        }
    }
}