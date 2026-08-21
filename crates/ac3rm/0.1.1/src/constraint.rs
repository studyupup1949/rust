use crate::VarId;
use std::{fmt, ops::Deref};

#[derive(Debug, Clone)]
pub enum Constraint {
    Equality(VarId, VarId),   // Represents an equality constraint between two variables (e.g., x_i == x_j).
    Inequality(VarId, VarId), // Represents an inequality constraint between two variables (e.g., x_i != x_j).
    Set(VarId, i32),          // Represents a constraint that a variable must take a specific value (e.g., x_i == 5).
    Forbid(VarId, i32),       // Represents a constraint that a variable cannot take a specific value (e.g., x_i != 5).
}

impl fmt::Display for Constraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Constraint::Equality(var1, var2) => write!(f, "{} == {}", var1, var2),
            Constraint::Inequality(var1, var2) => write!(f, "{} != {}", var1, var2),
            Constraint::Set(var, value) => write!(f, "{} == {}", var, value),
            Constraint::Forbid(var, value) => write!(f, "{} != {}", var, value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstraintId(usize);

impl ConstraintId {
    pub(crate) fn new(index: usize) -> Self {
        ConstraintId(index)
    }
}

impl Deref for ConstraintId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for ConstraintId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "c{}", self.0)
    }
}
