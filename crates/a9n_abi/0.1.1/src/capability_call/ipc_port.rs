pub use a9n_types::MessageInfo;

#[repr(usize)]
pub enum OperationType {
    None,
    Send,
    Receive,
    Call,
    Reply,
    ReplyReceive,
    Identify,
}
