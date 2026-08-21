#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Action {
    pub name: &'static str,
}

impl From<&'static str> for Action {
    fn from(name: &'static str) -> Self {
        Action { name }
    }
}

unsafe impl Send for Action {}
unsafe impl Sync for Action {}
