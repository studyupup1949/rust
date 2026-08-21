use sha2::{Sha256, Digest};

#[derive(Clone)]
pub struct CrystalConfig {
    pub size: usize,
}

impl Default for CrystalConfig {
    fn default() -> Self {
        CrystalConfig { size: 256 } // 256 байт на сообщение
    }
}

pub struct Crystal {
    cells: Vec<Option<u8>>,
}

impl Crystal {
    pub fn new(seed: &[u8], config: CrystalConfig) -> Self {
        let total_cells = config.size;
        Crystal {
            cells: vec![None; total_cells],
        }
    }

    pub fn encode(&mut self, message: &[u8]) -> Vec<usize> {
        let mut indices = Vec::new();
        
        for (i, &byte) in message.iter().enumerate() {
            if i < self.cells.len() {
                self.cells[i] = Some(byte);
                indices.push(i);
            }
        }
        
        indices
    }

    pub fn decode(&mut self, indices: &[usize], expected_len: usize) -> Option<Vec<u8>> {
        let mut result = Vec::with_capacity(expected_len);
        
        for &idx in indices {
            if let Some(Some(byte)) = self.cells.get(idx) {
                result.push(*byte);
            }
        }
        
        for &idx in indices {
            self.cells[idx] = None;
        }
        
        Some(result)
    }

    pub fn wipe(&mut self) {
        for cell in &mut self.cells {
            *cell = None;
        }
    }
}