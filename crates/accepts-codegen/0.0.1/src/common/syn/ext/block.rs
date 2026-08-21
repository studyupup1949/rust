use syn::{Block, Stmt, token::Brace};

pub trait BlockConstructExt {
    fn from_stmts(stmts: Vec<Stmt>) -> Block;
}

impl BlockConstructExt for Block {
    fn from_stmts(stmts: Vec<Stmt>) -> Block {
        Block {
            brace_token: Brace::default(),
            stmts,
        }
    }
}
