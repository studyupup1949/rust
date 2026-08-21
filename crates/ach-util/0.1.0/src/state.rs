#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[atomic_macro::atomic(8)]
pub enum MemoryState {
    Uninitialized = 0,
    Initializing = 1,
    Initialized = 2,
    Erasing = 3,
    Peeking = 4,
}
impl MemoryState {
    pub fn is_uninitialized(&self) -> bool {
        self == &Self::Uninitialized
    }
    pub fn is_initializing(&self) -> bool {
        self == &Self::Initializing
    }
    pub fn is_initialized(&self) -> bool {
        self == &Self::Initialized
    }
    pub fn is_erasing(&self) -> bool {
        self == &Self::Erasing
    }
    pub fn is_peeking(&self) -> bool {
        self == &Self::Peeking
    }
}
impl From<u8> for MemoryState {
    fn from(s: u8) -> Self {
        match s {
            1 => MemoryState::Initializing,
            2 => MemoryState::Initialized,
            3 => MemoryState::Erasing,
            4 => MemoryState::Peeking,
            _ => MemoryState::Uninitialized,
        }
    }
}
impl From<MemoryState> for u8 {
    fn from(s: MemoryState) -> Self {
        s as u8
    }
}
