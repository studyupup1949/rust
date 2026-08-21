//! AbleScript's Abstract Syntax tree
//!
//! Statements are the type which is AST made of, as they
//! express an effect.
//!
//! Expressions are just operations and they cannot be
//! used as statements. Functions in AbleScript are in fact
//! just plain subroutines and they do not return any value,
//! so their calls are statements.

use crate::{base_55::char2num, value::Value};
use std::{fmt::Debug, hash::Hash};

type Span = std::ops::Range<usize>;

#[derive(Clone)]
pub struct Spanned<T> {
    pub item: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(item: T, span: Span) -> Self {
        Self { item, span }
    }
}

impl<T: Debug> Debug for Spanned<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "{:#?} @ {:?}", self.item, self.span)
        } else {
            write!(f, "{:?} @ {:?}", self.item, self.span)
        }
    }
}

impl<T: PartialEq> PartialEq for Spanned<T> {
    fn eq(&self, other: &Self) -> bool {
        self.item == other.item
    }
}

impl<T: Hash> Hash for Spanned<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.item.hash(state);
    }
}

#[derive(Debug, PartialEq, Clone, Hash)]
pub struct Assignable {
    pub ident: Spanned<String>,
    pub kind: AssignableKind,
}

#[derive(Debug, PartialEq, Clone, Hash)]
pub enum AssignableKind {
    Variable,
    Index { indices: Vec<Spanned<Expr>> },
}

pub struct InvalidAssignable;

impl Assignable {
    pub fn from_expr(expr: Spanned<Expr>) -> Result<Assignable, InvalidAssignable> {
        match expr.item {
            Expr::Variable(ident) => Ok(Assignable {
                ident: Spanned::new(ident, expr.span),
                kind: AssignableKind::Variable,
            }),
            Expr::Index { expr, index } => Self::from_index(*expr, *index),
            _ => Err(InvalidAssignable),
        }
    }

    fn from_index(
        mut buf: Spanned<Expr>,
        index: Spanned<Expr>,
    ) -> Result<Assignable, InvalidAssignable> {
        let mut indices = vec![index];
        let ident = loop {
            match buf.item {
                Expr::Variable(ident) => break ident,
                Expr::Index { expr, index } => {
                    indices.push(*index);
                    buf = *expr;
                }
                _ => return Err(InvalidAssignable),
            }
        };

        indices.reverse();
        Ok(Assignable {
            ident: Spanned::new(ident, buf.span),
            kind: AssignableKind::Index { indices },
        })
    }
}

pub type Block = Vec<Spanned<Stmt>>;

/// A syntactic unit expressing an effect.
#[derive(Debug, PartialEq, Clone, Hash)]
pub enum Stmt {
    // Control flow
    Unless {
        cond: Spanned<Expr>,
        body: Block,
    },
    Loop {
        body: Block,
    },
    Enough,
    AndAgain,

    Dim {
        ident: Spanned<String>,
        init: Option<Spanned<Expr>>,
    },
    Assign {
        assignable: Assignable,
        value: Spanned<Expr>,
    },

    Functio {
        ident: Spanned<String>,
        params: Vec<Spanned<String>>,
        body: Block,
    },
    BfFunctio {
        ident: Spanned<String>,
        tape_len: Option<Spanned<Expr>>,
        code: Vec<u8>,
    },
    Call {
        expr: Spanned<Expr>,
        args: Vec<Spanned<Expr>>,
    },
    Print {
        expr: Spanned<Expr>,
        newline: bool,
    },
    Read(Assignable),
    Melo(Spanned<String>),
    Finally(Block),
    Rlyeh,
    Rickroll,
}

/// Expression is parse unit which do not cause any effect,
/// like math and logical operations or values.
#[derive(Debug, PartialEq, Clone, Hash)]
pub enum Expr {
    BinOp {
        lhs: Box<Spanned<Expr>>,
        rhs: Box<Spanned<Expr>>,
        kind: BinOpKind,
    },
    Aint(Box<Spanned<Expr>>),
    Literal(Literal),
    Cart(Vec<(Spanned<Expr>, Spanned<Expr>)>),
    Index {
        expr: Box<Spanned<Expr>>,
        index: Box<Spanned<Expr>>,
    },
    Len(Box<Spanned<Expr>>),
    Keys(Box<Spanned<Expr>>),
    Variable(String),
}

#[derive(Debug, PartialEq, Clone, Hash)]
pub enum Literal {
    Char(char),
    Int(isize),
    Str(String),
}

impl From<Literal> for Value {
    fn from(lit: Literal) -> Self {
        match lit {
            Literal::Char(c) => Self::Int(char2num(c)),
            Literal::Int(i) => Self::Int(i),
            Literal::Str(s) => Self::Str(s),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Hash)]
pub enum BinOpKind {
    Add,
    Subtract,
    Multiply,
    Divide,
    Greater,
    Less,
    Equal,
    NotEqual,
}

impl BinOpKind {
    pub fn from_token(t: crate::lexer::Token) -> Result<Self, crate::error::ErrorKind> {
        use crate::lexer::Token;

        match t {
            Token::Plus => Ok(Self::Add),
            Token::Minus => Ok(Self::Subtract),
            Token::Star => Ok(Self::Multiply),
            Token::FwdSlash => Ok(Self::Divide),
            Token::GreaterThan => Ok(Self::Greater),
            Token::LessThan => Ok(Self::Less),
            Token::Equals => Ok(Self::Equal),
            Token::Aint => Ok(Self::NotEqual),
            t => Err(crate::error::ErrorKind::UnexpectedToken(t)),
        }
    }
}
