use std::collections::HashSet;
use std::fmt::Debug;

pub trait NodeTrait: Debug + Clone {
    fn new(key: u32) -> Self;
    fn key(&self) -> u32;
    fn fresh_copy(&self) -> Self;
    fn predecessors(&self) -> &HashSet<u32>;
    fn successors(&self) -> &HashSet<u32>;
    fn add_predecessor(&mut self, key: u32);
    fn add_successor(&mut self, key: u32);
    fn remove_predecessor(&mut self, key: u32);
    fn remove_successor(&mut self, key: u32);
}
