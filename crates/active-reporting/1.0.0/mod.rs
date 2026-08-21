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
#![feature(negative_impls)]
#![feature(const_heap)]
#![feature(const_default)]
#![feature(generic_const_exprs)]

//> HEAD -> MODULES
mod root;

//> HEAD -> ROOT
pub use root::Root;


//^ 
//^ REPORT
//^ 

//> REPORT -> STRUCT
pub struct Report<'valid, const NAME: &'static str> {
    chain: &'valid mut Vec<&'static str>
}

//> REPORT -> IMPLEMENTATION
impl<'valid, const NAME: &'static str> Report<'valid, NAME> {
    pub const fn chain(&'valid self) -> &'valid [&'static str] {return self.chain.as_slice()}
    pub const fn to<const OTHER: &'static str>(&'valid mut self) -> Report<'valid, OTHER> {
        if !OTHER.is_empty() {self.chain.push(OTHER);}
        return Report {
            chain: self.chain
        }
    }
}

//> REPORT -> DROP
impl<'valid, const NAME: &'static str> Drop for Report<'valid, NAME> {
    fn drop(&mut self) {if !NAME.is_empty() {self.chain.pop();}}
}

//> REPORT -> !SEND
impl<'valid, const NAME: &'static str> !Send for Report<'valid, NAME> {}

//> REPORT -> !SYNC
impl<'valid, const NAME: &'static str> !Sync for Report<'valid, NAME> {}