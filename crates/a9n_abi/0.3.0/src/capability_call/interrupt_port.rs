use crate::*;

#[repr(usize)]
pub enum OperationType {
    None,
    Bind,
    Unbind,
    Ack,
    GetIrqNumber,
}
