#[derive(Debug)]
pub(crate) struct OperationRecord<F: Sized>(pub [(usize, F); 2]);
