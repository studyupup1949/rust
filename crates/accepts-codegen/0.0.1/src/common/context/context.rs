#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodegenContext {
    pub internal: bool,
}
impl CodegenContext {
    pub fn new(internal: bool) -> Self {
        Self { internal }
    }

    pub const fn new_const(internal: bool) -> Self {
        Self { internal }
    }
}
