use crate::*;
use bitflags::bitflags;

#[repr(usize)]
pub enum OperationType {
    None,
    Map,
    Unmap,
    GetUnsetDepth,
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Attribute: usize
    {
        const READ = 1 << 0;
        const WRITE = 1 << 1;
        const EXECUTE = 1 << 2;
        const ALL = (1 << 0) | (1 << 1) | (1 << 2);
    }
}
