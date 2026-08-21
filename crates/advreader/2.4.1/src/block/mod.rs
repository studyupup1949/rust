pub mod a2l;

use std::iter::Enumerate;

pub use a2l::A2LBlock;

use crate::{AdvReturnValue, FnBlockReturnType};

pub trait Block {
    fn new(line_end: u8, skip_comments: bool) -> Box<Self>
    where
        Self: Sized + Send + Sync;

    fn is_block_mode(&self) -> bool;

    fn start_block(&mut self);

    fn build_result(&mut self) -> Vec<AdvReturnValue>;

    fn read_block(
        &mut self,
        item: &[u8],
        buffer: &[u8],
        itr: &mut Enumerate<std::slice::Iter<'_, u8>>,
        i: usize,
        c: u8,
        line_num: &mut usize,
    ) -> FnBlockReturnType;
}
