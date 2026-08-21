use super::ValueRef;
use crate::ast::Block;
use std::{hash::Hash, rc::Rc};

/// AbleScript Function
#[derive(Debug, PartialEq, Clone, Hash)]
pub enum Functio {
    /// BF instructions and a length of the type
    ///
    /// Takes input bytes as parameters
    Bf {
        instructions: Vec<u8>,
        tape_len: usize,
    },

    /// Regular AbleScript functio
    ///
    /// Consists of parameter name mapping and AST
    Able { params: Vec<String>, body: Block },

    /// Builtin Rust functio
    Builtin(BuiltinFunctio),

    /// Chained functio pair
    Chain {
        functios: Box<(Functio, Functio)>,
        kind: FunctioChainKind,
    },

    /// Code to be parsed and then executed in current scope
    Eval(String),
}

impl Functio {
    pub fn arity(&self) -> usize {
        match self {
            Functio::Bf {
                instructions: _,
                tape_len: _,
            } => 0,
            Functio::Able { params, body: _ } => params.len(),
            Functio::Builtin(b) => b.arity,
            Functio::Chain { functios, kind: _ } => functios.0.arity() + functios.1.arity(),
            Functio::Eval(_) => 0,
        }
    }
}

/// Built-in Rust functio
#[derive(Clone)]
pub struct BuiltinFunctio {
    pub(super) function: Rc<dyn Fn(&[ValueRef]) -> Result<(), crate::error::ErrorKind>>,
    pub(super) arity: usize,
}

impl BuiltinFunctio {
    /// Wrap a Rust function into AbleScript's built-in functio
    /// 
    /// Arity used for functio chaining, recommend value for variadic
    /// functions is the accepted minimum.
    pub fn new<F>(f: F, arity: usize) -> Self
    where
        F: Fn(&[ValueRef]) -> Result<(), crate::error::ErrorKind> + 'static,
    {
        Self {
            function: Rc::new(f),
            arity,
        }
    }

    pub fn call(&self, args: &[ValueRef]) -> Result<(), crate::error::ErrorKind> {
        (self.function)(args)
    }

    pub fn fn_addr(&self) -> usize {
        Rc::as_ptr(&self.function) as *const () as _
    }
}

impl std::fmt::Debug for BuiltinFunctio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BuiltinFunctio")
            .field("function", &"built-in")
            .field("arity", &self.arity)
            .finish()
    }
}

impl PartialEq for BuiltinFunctio {
    fn eq(&self, other: &Self) -> bool {
        self.fn_addr() == other.fn_addr() && self.arity == other.arity
    }
}

impl Hash for BuiltinFunctio {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.fn_addr().hash(state);
        self.arity.hash(state);
    }
}

/// A method of distributting parameters across functio chain
#[derive(Debug, PartialEq, Copy, Clone, Hash)]
pub enum FunctioChainKind {
    /// All parameters are equally distributed
    Equal,

    /// Parameters are distributed to members of chain
    /// by their arity
    ByArity,
}
