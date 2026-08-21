use crate::{brian::InterpretError, lexer::Token};
use std::{fmt::Display, io, ops::Range};

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub span: Range<usize>,
}

#[derive(Debug)]
pub enum ErrorKind {
    /// Parser expected token, but none was available
    UnexpectedEoi,

    /// Parser encountered unknown token
    InvalidToken,

    /// Parser expected certain token, but other one appeared
    UnexpectedToken(Token),

    /// Attempted to assign to undefined variable
    UnknownVariable(String),

    /// Attempted to access banned variable
    MeloVariable(String),

    /// Breaking / re-starting loop outside loop
    LoopOpOutsideLoop,

    /// Rlyeh was executed but host interface's exit
    /// doesn't exit the program
    NonExitingRlyeh(i32),

    /// Missing left-hand side expression in binary expression
    MissingLhs,

    /// Error when executing BF code
    Brian(InterpretError),

    /// IO Error
    Io(io::Error),
}

impl Error {
    pub fn new(kind: ErrorKind, span: Range<usize>) -> Self {
        Self { kind, span }
    }

    /// Create an UnexpectedEoi error, where the EOI occurs at the
    /// given index in the input.
    pub fn unexpected_eoi(index: usize) -> Self {
        Self::new(ErrorKind::UnexpectedEoi, index..index)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error at range {}-{}: {}",
            self.span.start, self.span.end, self.kind
        )
    }
}
impl std::error::Error for Error {}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::UnexpectedEoi => write!(f, "unexpected end of input"),
            ErrorKind::InvalidToken => write!(f, "invalid token"),
            ErrorKind::UnexpectedToken(Token::Melo) => write!(f, "unexpected marten"),
            ErrorKind::UnexpectedToken(token) => write!(f, "unexpected token {:?}", token),
            ErrorKind::UnknownVariable(name) => write!(f, "unknown identifier \"{}\"", name),
            ErrorKind::MeloVariable(name) => write!(f, "banned variable \"{}\"", name),
            ErrorKind::LoopOpOutsideLoop => write!(
                f,
                "unable to perform loop operation (enough or and enough) outside a loop"
            ),
            &ErrorKind::NonExitingRlyeh(code) => write!(f, "program exited with code {code}"),
            ErrorKind::Brian(err) => write!(f, "brainfuck error: {}", err),
            // TODO: give concrete numbers here.
            ErrorKind::MissingLhs => write!(f, "missing expression before binary operation"),
            ErrorKind::Io(err) => write!(f, "I/O error: {}", err),
        }
    }
}

impl From<io::Error> for ErrorKind {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
