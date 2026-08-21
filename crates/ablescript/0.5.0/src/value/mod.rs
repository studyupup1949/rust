pub mod functio;

mod coercions;
mod ops;

pub use functio::Functio;

use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    fmt::Display,
    hash::Hash,
    io::Write,
    mem::discriminant,
    rc::Rc,
};

pub type Cart = HashMap<Value, ValueRef>;

/// AbleScript Value
#[derive(Debug, Default, Clone)]
pub enum Value {
    #[default]
    Nul,
    Undefined,
    Str(String),
    Int(isize),
    Abool(Abool),
    Functio(Functio),
    Cart(Cart),
}

impl Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        discriminant(self).hash(state);
        match self {
            Value::Nul | Value::Undefined => (),
            Value::Str(v) => v.hash(state),
            Value::Int(v) => v.hash(state),
            Value::Abool(v) => v.to_string().hash(state),
            Value::Functio(statements) => statements.hash(state),
            Value::Cart(_) => self.to_string().hash(state),
        }
    }
}

impl Value {
    /// Write an AbleScript value to a Brainfuck input stream by
    /// coercing the value to an integer, then truncating that integer
    /// to a single byte, then writing that byte. This should only be
    /// called on `Write`rs that cannot fail, e.g., `Vec<u8>`, because
    /// any IO errors will cause a panic.
    pub fn bf_write(&self, stream: &mut impl Write) {
        stream
            .write_all(&[self.clone().into_isize() as u8])
            .expect("Failed to write to Brainfuck input");
    }
}

/// Three-state logic value
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Abool {
    Never = -1,
    Sometimes = 0,
    Always = 1,
}

impl Abool {
    pub fn to_bool(&self) -> bool {
        match self {
            Self::Always => true,
            Self::Sometimes if rand::random() => true,
            _ => false,
        }
    }
}

impl Display for Abool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Abool::Never => write!(f, "never"),
            Abool::Sometimes => write!(f, "sometimes"),
            Abool::Always => write!(f, "always"),
        }
    }
}

impl From<bool> for Abool {
    fn from(b: bool) -> Self {
        if b {
            Abool::Always
        } else {
            Abool::Never
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nul => write!(f, "nul"),
            Value::Undefined => write!(f, "undefined"),
            Value::Str(v) => write!(f, "{}", v),
            Value::Int(v) => write!(f, "{}", v),
            Value::Abool(v) => write!(f, "{}", v),
            Value::Functio(v) => match v {
                Functio::Bf {
                    instructions,
                    tape_len,
                } => {
                    write!(
                        f,
                        "({}) {}",
                        tape_len,
                        String::from_utf8(instructions.to_owned())
                            .expect("Brainfuck functio source should be UTF-8")
                    )
                }
                Functio::Able { params, body } => {
                    write!(
                        f,
                        "({}) -> {:?}",
                        params.join(", "),
                        // Maybe we should have a pretty-printer for
                        // statement blocks at some point?
                        body,
                    )
                }
                Functio::Builtin(b) => write!(f, "builtin @ {}", b.fn_addr()),
                Functio::Chain { functios, kind } => {
                    let (a, b) = *functios.clone();
                    write!(
                        f,
                        "{} {} {} ",
                        Value::Functio(a),
                        match kind {
                            functio::FunctioChainKind::Equal => '+',
                            functio::FunctioChainKind::ByArity => '*',
                        },
                        Value::Functio(b)
                    )
                }
                Functio::Eval(s) => write!(f, "{}", s),
            },
            Value::Cart(c) => {
                write!(f, "[")?;
                let mut cart_vec = c.iter().collect::<Vec<_>>();
                cart_vec.sort_by(|x, y| x.0.partial_cmp(y.0).unwrap_or(std::cmp::Ordering::Less));

                for (idx, (key, value)) in cart_vec.into_iter().enumerate() {
                    write!(f, "{}", if idx != 0 { ", " } else { "" },)?;
                    match &*value.borrow() {
                        x if std::ptr::eq(x, self) => write!(f, "<cycle>"),
                        x => write!(f, "{x}"),
                    }?;
                    write!(f, " <= {key}")?;
                }

                write!(f, "]")
            }
        }
    }
}

/// Runtime borrow-checked, counted reference to a [Value]
#[derive(Debug, Clone, PartialEq)]
pub struct ValueRef(Rc<RefCell<Value>>);

impl ValueRef {
    pub fn new(v: Value) -> Self {
        Self(Rc::new(RefCell::new(v)))
    }

    pub fn borrow(&self) -> Ref<Value> {
        self.0.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<Value> {
        self.0.borrow_mut()
    }

    pub fn replace(&self, v: Value) -> Value {
        self.0.replace(v)
    }
}

/// AbleScript variable either holding a reference
/// or being banned
#[derive(Debug)]
pub enum Variable {
    /// Reference to a value
    Ref(ValueRef),

    /// Banned variable
    Melo,
}

impl Variable {
    pub fn from_value(value: Value) -> Self {
        Self::Ref(ValueRef::new(value))
    }
}
