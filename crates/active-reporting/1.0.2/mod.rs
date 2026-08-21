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
#![feature(phantom_variance_markers)]
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
use core::{
    marker::PhantomCovariantLifetime,
    ptr::NonNull
};


//^ 
//^ REPORT
//^ 

//> REPORT -> STRUCT
pub struct Report<'valid, const NAME: &'static str> {
    chain: NonNull<Vec<&'static str>>,
    _lifetime: PhantomCovariantLifetime<'valid>
}

//> REPORT -> IMPLEMENTATION
impl<'valid, const NAME: &'static str> Report<'valid, NAME> {
    pub const fn chain(&'valid self) -> &'valid [&'static str] {
        return unsafe {self.chain.as_ref().as_slice()};
    }
    pub const fn to<const OTHER: &'static str>(&'valid mut self) -> Report<'valid, OTHER> {
        if !OTHER.is_empty() {unsafe {self.chain.as_mut().push(OTHER)};}
        return Report {
            chain: self.chain,
            _lifetime: PhantomCovariantLifetime::new()
        }
    }
}

//> REPORT -> DROP
impl<'valid, const NAME: &'static str> Drop for Report<'valid, NAME> {
    fn drop(&mut self) {if !NAME.is_empty() {unsafe {self.chain.as_mut().pop()};}}
}

//> REPORT -> !SEND
impl<'valid, const NAME: &'static str> !Send for Report<'valid, NAME> {}

//> REPORT -> !SYNC
impl<'valid, const NAME: &'static str> !Sync for Report<'valid, NAME> {}