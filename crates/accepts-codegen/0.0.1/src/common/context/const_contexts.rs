use super::CodegenContext;

pub const INTERNAL_CONTEXT: CodegenContext = CodegenContext::new_const(true);
pub const PUBLIC_CONTEXT: CodegenContext = CodegenContext::new_const(false);
