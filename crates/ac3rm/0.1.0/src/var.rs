use crate::ConstraintId;
use std::{
    collections::{HashMap, HashSet},
    fmt,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VarId(pub usize);

impl fmt::Display for VarId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "e{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub(super) struct ValueState {
    pub(super) value: i32,
    pub(super) killers: HashSet<ConstraintId>,
}

#[derive(Debug, Clone)]
pub(super) struct Variable {
    pub(super) domain: Vec<ValueState>,
    pub(super) index_by_value: HashMap<i32, usize>,
}
