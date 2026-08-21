//! # The Addon Library
//! The addon library is a library meant to extend the Rust standard library with types. Sometimes when you use the standard library,
//! certain, seemingly 'obvious' things are missing, leaving you with either importing a new crate or implementing it yourself.
//! This repeats, and one day you'll end up with either 10 dependencies or a mini-std library in your project.
//!
//! The addon library fixes that, by implementing your 'obvious' types, isolated as one dependency. It's focused on types and traits, less
//! towards what the standard library implements (networking, io, os stuff, etc.) It's `no_std + alloc` by default,
//! which means you don't even need the foundational Rust standard library to have the `addon` library.

#![warn(missing_docs)]
#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod prelude;

pub mod ord;
#[cfg(feature = "alloc")]
pub mod vec;
