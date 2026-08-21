use std::collections::HashSet;
use acute_window::event::VirtualKeyCode;

pub struct Keyboard {
    pub just_pressed: HashSet<VirtualKeyCode>,
    pub pressed: HashSet<VirtualKeyCode>,
    pub just_released: HashSet<VirtualKeyCode>,
}

impl Keyboard {
    pub fn just_pressed(&self, key: VirtualKeyCode) -> bool {
        self.just_pressed.contains(&key)
    }

    pub fn pressed(&self, key: VirtualKeyCode) -> bool {
        self.pressed.contains(&key)
    }

    pub fn just_released(&self, key: VirtualKeyCode) -> bool {
        self.just_released.contains(&key)
    }

    pub(crate) fn press(&mut self, key: VirtualKeyCode) {
        if !self.pressed(key) {
            self.just_pressed.insert(key);
        }
        self.pressed.insert(key);
    }

    pub(crate) fn release(&mut self, key: VirtualKeyCode) {
        self.pressed.remove(&key);
        self.just_released.insert(key);
    }

    pub(crate) fn clear(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    pub(crate) fn new() -> Self {
        Self {
            just_pressed: Default::default(),
            pressed: Default::default(),
            just_released: Default::default(),
        }
    }
}