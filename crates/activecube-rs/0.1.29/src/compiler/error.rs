/// Cube query compilation error (framework-agnostic).
#[derive(Debug, Clone)]
pub struct CompileError {
    pub message: String,
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CompileError {}

impl CompileError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self { message: msg.into() }
    }
}
