//^
//^ HEAD
//^

//> HEAD -> SUPER
use super::Report;

//> HEAD -> CORE
use core::ptr::NonNull;

//> HEAD -> ALLOC
use alloc::vec::Vec;


//^
//^ ROOT
//^

//> ROOT -> STRUCT
pub struct Root {
    chain: Vec<&'static str>
}

//> ROOT -> IMPLEMENTATION
impl Root {
    pub const fn new(name: &'static str) -> Self {return Self {
        chain: if name.is_empty() {Vec::new()} else {
            let mut chain = Vec::new();
            chain.push(name);
            chain
        }
    }}
    pub const fn chain(&self) -> &[&'static str] {return self.chain.as_slice()}
    pub const fn to<const NAME: &'static str>(&mut self) -> Report<NAME> {
        if !NAME.is_empty() {self.chain.push(NAME);}
        return Report {
            chain: NonNull::new(&raw mut self.chain).unwrap()
        }
    }
}

//> ROOT -> DEFAULT
const impl Default for Root {
    fn default() -> Self {return Self::new("Main")}
}

//> ROOT -> !SYNC
impl !Sync for Root {}