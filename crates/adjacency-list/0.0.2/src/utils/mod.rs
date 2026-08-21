use graph_types::IndeterminateEdge;

// platform dependent edge
#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ShortEdge {
    pub from: u32,
    pub goto: u32,
}

impl ShortEdge {
    pub const fn new(from: usize, goto: usize) -> ShortEdge {
        Self { from: from as u32, goto: goto as u32 }
    }
    pub const fn as_indeterminate(&self) -> IndeterminateEdge {
        IndeterminateEdge::new(self.from as usize, self.goto as usize)
    }
}
