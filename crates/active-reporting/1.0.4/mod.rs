//^
//^ HEAD
//^

//> HEAD -> NO STD
#![no_std]

//> HEAD -> DOCS
#![doc = include_str!("README.md")]

//> HEAD -> LINTS
#![allow(incomplete_features)]

//> HEAD -> FEATURES
#![feature(const_trait_impl)]
#![feature(unsized_const_params)]
#![feature(adt_const_params)]
#![feature(negative_impls)]
#![feature(const_heap)]
#![feature(const_default)]
#![feature(generic_const_exprs)]

//> HEAD -> CRATES
extern crate alloc;

//> HEAD -> MODULES
mod root;

//> HEAD -> ROOT
pub use root::Root;

//> HEAD -> ALLOC
use alloc::vec::Vec;

//> HEAD -> CORE
use core::ptr::NonNull;


//^ 
//^ REPORT
//^ 

//> REPORT -> STRUCT
pub struct Report<const NAME: &'static str> {
    chain: NonNull<Vec<&'static str>>,
}

//> REPORT -> IMPLEMENTATION
impl<const NAME: &'static str> Report<NAME> {
    pub const fn chain(&self) -> &[&'static str] {
        return unsafe {self.chain.as_ref().as_slice()};
    }
    pub const fn to<const OTHER: &'static str>(&mut self) -> Report<OTHER> {
        if !OTHER.is_empty() {unsafe {self.chain.as_mut().push(OTHER)};}
        return Report {
            chain: self.chain
        }
    }
}

//> REPORT -> DROP
impl<const NAME: &'static str> Drop for Report<NAME> {
    fn drop(&mut self) {if !NAME.is_empty() {unsafe {self.chain.as_mut().pop()};}}
}

//> REPORT -> !SEND
impl<const NAME: &'static str> !Send for Report<NAME> {}

//> REPORT -> !SYNC
impl<const NAME: &'static str> !Sync for Report<NAME> {}