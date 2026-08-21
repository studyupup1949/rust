use crate::*;

#[repr(usize)]
pub enum OperationType {
    None,
    Copy,
    Move,
    Mint,
    Demote,
    Revoke,
    Remove,
}
