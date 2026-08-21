use std::fmt::Debug;

pub trait EdgeTrait: Debug + Clone {
    fn new(source: u32, target: u32) -> Self;
    fn source(&self) -> u32;
    fn target(&self) -> u32;
}
