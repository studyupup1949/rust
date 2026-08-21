//^
//^ HEAD
//^

//> HEAD -> DOCS
#![doc = include_str!("README.md")]

//> HEAD -> LINTS
#![allow(incomplete_features)]

//> HEAD -> FEATURES
#![feature(const_trait_impl)]
#![feature(unsized_const_params)]
#![feature(adt_const_params)]
#![feature(default_field_values)]
#![feature(const_heap)]
#![feature(const_default)]
#![feature(never_type)]
#![feature(generic_const_exprs)]
#![feature(test)]

//> HEAD -> CRATES
extern crate test;

//> HEAD -> MODULES
#[cfg(test)]
mod benches;
mod state;
#[cfg(test)]
mod tests;

//> HEAD -> PUBLIC STATE
pub use state::{
    Same,
    Name
};

//> HEAD -> STATE
use state::{
    State,
    Main,
    DerivedState
};

//> HEAD -> ISSUING
use issuing::Issue;

//> HEAD -> TERMINAL
use libutils_terminal::Terminal;

//> HEAD -> CONSOLE
use libutils_console::{
    Console, 
    Update
};


//^ 
//^ REPORT
//^ 

//> REPORT -> STRUCT
pub struct Report<Current: State> {
    data: Current
}

//> REPORT -> MAIN IMPLEMENTATION
impl Report<Main> {
    pub const fn new(name: &'static str) -> Self {
        let mut chain = Vec::new();
        chain.push(name);
        return Self {
            data: Main {
                chain: chain
            }
        };
    }
}

//> REPORT -> IMPLEMENTATION
impl<Current: const State> Report<Current> {
    pub const fn to<'valid, Following: const DerivedState<'valid>>(&'valid mut self) -> Report<Following> {
        return Report {
            data: Following::convert(&mut self.data)
        }
    }
    pub fn issue(&self, object: impl Into<Issue>) -> Option<!> {
        Terminal::problem(object.into(), self.data.get()).sync();
        return None;
    }
    pub fn eat<Type>(&self, result: Result<Type, Issue>) -> Option<Type> {return match result {
        Ok(value) => Some(value),
        Err(issue) => self.issue(issue)?
    }}
}

//> REPORT -> DEFAULT
const impl Default for Report<Main> {
    fn default() -> Self {return Self::new("Main")}
}