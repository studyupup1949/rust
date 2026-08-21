use crate::*;

#[repr(usize)]
pub enum OperationType {
    None,
    ConfigureAddressSpace,
    ConfigureStateDescriptor,
    ReadState,
    WriteState,
    Enter,
    Exit,
    InjectIrq,
}
