use crate::*;

#[repr(usize)]
pub enum OperationType {
    None,
    Read,
    Write,
    Mint,
}
