use crate::*;

#[repr(usize)]
pub enum OperationType {
    None,
    Map,
    Unmap,
    GetUnsetDepth,
}
